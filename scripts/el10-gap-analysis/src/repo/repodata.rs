use crate::repo::types::*;
use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn fetch_repodata(config: &RepoConfig, cache_dir: &Path) -> Result<Vec<RepoPackage>> {
    fs::create_dir_all(cache_dir)?;
    let cache_file = cache_dir.join(format!("{}-primary.xml.gz", config.name));

    if !cache_file.exists() || file_age_hours(&cache_file) > 24 {
        download_primary(config, &cache_file)?;
    }

    parse_primary_xml(&cache_file, &config.name)
}

fn file_age_hours(path: &Path) -> u64 {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .map(|d| d.as_secs() / 3600)
        .unwrap_or(u64::MAX)
}

fn download_primary(config: &RepoConfig, dest: &Path) -> Result<()> {
    let repomd_url = format!("{}/repodata/repomd.xml", config.base_url);
    eprintln!("  Fetching repomd.xml from {} ...", config.name);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let repomd_body = client
        .get(&repomd_url)
        .send()
        .with_context(|| format!("Failed to fetch {repomd_url}"))?
        .text()
        .with_context(|| format!("Failed to read {repomd_url}"))?;

    let primary_href = extract_primary_href(&repomd_body)
        .with_context(|| format!("No primary data found in repomd.xml for {}", config.name))?;

    let primary_url = format!("{}/{}", config.base_url, primary_href);
    eprintln!("  Downloading primary metadata: {primary_href} ...");

    let mut response = client
        .get(&primary_url)
        .send()
        .with_context(|| format!("Failed to fetch {primary_url}"))?;

    let mut file = fs::File::create(dest)?;
    std::io::copy(&mut response, &mut file)?;
    eprintln!(
        "  Cached to {}",
        dest.file_name().unwrap().to_string_lossy()
    );
    Ok(())
}

fn extract_primary_href(repomd_xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(repomd_xml);
    let mut in_primary = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"data" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"type"
                        && attr.unescape_value().ok().as_deref() == Some("primary")
                    {
                        in_primary = true;
                    }
                }
            }
            Ok(Event::Empty(ref e)) if in_primary && e.name().as_ref() == b"location" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"href" {
                        return attr.unescape_value().ok().map(|v| v.into_owned());
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"data" => {
                in_primary = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

pub fn parse_primary_xml(path: &Path, repo_name: &str) -> Result<Vec<RepoPackage>> {
    let file = fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;

    let reader: Box<dyn Read> = if path
        .extension()
        .is_some_and(|ext| ext == "gz")
    {
        Box::new(GzDecoder::new(file))
    } else {
        Box::new(file)
    };

    let buf_reader = BufReader::with_capacity(256 * 1024, reader);
    let mut xml_reader = Reader::from_reader(buf_reader);
    xml_reader.config_mut().trim_text(true);

    let mut packages = Vec::new();
    let mut buf = Vec::new();
    let mut current: Option<RepoPackageBuilder> = None;
    let mut in_provides = false;
    let mut in_requires = false;
    let mut text_buf = String::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name().into_inner().to_vec();
                let local = local_name(&name_bytes);
                match local {
                    b"package" => {
                        let is_rpm = e.attributes().flatten().any(|a| {
                            a.key.as_ref() == b"type"
                                && a.unescape_value().ok().as_deref() == Some("rpm")
                        });
                        if is_rpm {
                            current = Some(RepoPackageBuilder::new());
                        }
                    }
                    b"name" | b"file" => {
                        text_buf.clear();
                    }
                    b"provides" => in_provides = true,
                    b"requires" => in_requires = true,
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name().into_inner().to_vec();
                let local = local_name(&name_bytes);
                if let Some(ref mut pkg) = current {
                    match local {
                        b"version" => {
                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"epoch" => {
                                        pkg.epoch = attr
                                            .unescape_value()
                                            .ok()
                                            .and_then(|v| v.parse().ok())
                                            .unwrap_or(0);
                                    }
                                    b"ver" => {
                                        pkg.version = attr
                                            .unescape_value()
                                            .map(|v| v.into_owned())
                                            .unwrap_or_default();
                                    }
                                    b"rel" => {
                                        pkg.release = attr
                                            .unescape_value()
                                            .map(|v| v.into_owned())
                                            .unwrap_or_default();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"entry" if in_provides => {
                            pkg.provides.push(parse_rpm_entry(e));
                        }
                        b"entry" if in_requires => {
                            // We don't need requires from repodata for gap analysis
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(t) = e.unescape() {
                    text_buf.push_str(&t);
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().into_inner().to_vec();
                let local = local_name(&name_bytes);
                match local {
                    b"package" => {
                        if let Some(pkg) = current.take() {
                            if !pkg.name.is_empty() {
                                packages.push(pkg.build());
                            }
                        }
                    }
                    b"name" => {
                        if let Some(ref mut pkg) = current {
                            if pkg.name.is_empty() {
                                pkg.name = text_buf.clone();
                            }
                        }
                        text_buf.clear();
                    }
                    b"arch" => {
                        if let Some(ref mut pkg) = current {
                            pkg.arch = text_buf.clone();
                        }
                        text_buf.clear();
                    }
                    b"file" => {
                        if let Some(ref mut pkg) = current {
                            pkg.files.push(text_buf.clone());
                        }
                        text_buf.clear();
                    }
                    b"provides" => in_provides = false,
                    b"requires" => in_requires = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("  Warning: XML parse error in {repo_name}: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    // Deduplicate: keep highest version per package name
    let mut best: HashMap<String, RepoPackage> = HashMap::new();
    for pkg in packages {
        let key = format!("{}.{}", pkg.name, pkg.arch);
        let dominated = best.get(&key).map_or(false, |existing| {
            crate::analysis::version::compare_evr(
                pkg.epoch,
                &pkg.version,
                &pkg.release,
                existing.epoch,
                &existing.version,
                &existing.release,
            ) != std::cmp::Ordering::Greater
        });
        if !dominated {
            best.insert(key, pkg);
        }
    }

    Ok(best.into_values().collect())
}

fn parse_rpm_entry(e: &quick_xml::events::BytesStart) -> RepoProvide {
    let mut provide = RepoProvide {
        name: String::new(),
        flags: None,
        epoch: None,
        version: None,
        release: None,
    };
    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"name" => {
                provide.name = attr
                    .unescape_value()
                    .map(|v| v.into_owned())
                    .unwrap_or_default();
            }
            b"flags" => {
                provide.flags = attr.unescape_value().ok().map(|v| v.into_owned());
            }
            b"epoch" => {
                provide.epoch = attr.unescape_value().ok().and_then(|v| v.parse().ok());
            }
            b"ver" => {
                provide.version = attr.unescape_value().ok().map(|v| v.into_owned());
            }
            b"rel" => {
                provide.release = attr.unescape_value().ok().map(|v| v.into_owned());
            }
            _ => {}
        }
    }
    provide
}

fn local_name(full: &[u8]) -> &[u8] {
    full.iter()
        .rposition(|&b| b == b':')
        .map(|pos| &full[pos + 1..])
        .unwrap_or(full)
}

struct RepoPackageBuilder {
    name: String,
    arch: String,
    epoch: u32,
    version: String,
    release: String,
    provides: Vec<RepoProvide>,
    files: Vec<String>,
}

impl RepoPackageBuilder {
    fn new() -> Self {
        Self {
            name: String::new(),
            arch: String::new(),
            epoch: 0,
            version: String::new(),
            release: String::new(),
            provides: Vec::new(),
            files: Vec::new(),
        }
    }

    fn build(self) -> RepoPackage {
        RepoPackage {
            name: self.name,
            arch: self.arch,
            epoch: self.epoch,
            version: self.version,
            release: self.release,
            provides: self.provides,
            files: self.files,
        }
    }
}
