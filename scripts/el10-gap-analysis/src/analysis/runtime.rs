use crate::repo::types::RepoPackage;
use crate::report::types::RuntimeEntry;
use crate::spec::types::SpecFile;

const KEY_RUNTIMES: &[(&str, &[&str])] = &[
    ("Ruby", &["ruby"]),
    ("Python", &["python3.12", "python3", "python3.13"]),
    ("Node.js", &["nodejs"]),
    ("PostgreSQL", &["postgresql-server", "postgresql"]),
    ("Rails", &["rubygem-rails"]),
    ("Django", &["python3.12-django", "python-django"]),
    ("Ansible", &["ansible-core"]),
    ("Puppet/OpenVox", &["puppet-agent", "openvox-agent"]),
    ("systemd", &["systemd"]),
    ("OpenSSL", &["openssl", "openssl-libs"]),
];

pub fn detect_runtimes(
    repo_packages: &[(&str, &[RepoPackage])],
    spec_files: &[SpecFile],
) -> Vec<RuntimeEntry> {
    let mut entries = Vec::new();

    for &(display_name, package_names) in KEY_RUNTIMES {
        let mut el10_version = None;
        let mut el10_source = None;

        // Search in repo packages first
        for &(source, pkgs) in repo_packages {
            for name in package_names.iter() {
                if let Some(pkg) = pkgs.iter().find(|p| &p.name == name) {
                    el10_version = Some(pkg.version.clone());
                    el10_source = Some(source.to_string());
                    break;
                }
            }
            if el10_version.is_some() {
                break;
            }
        }

        // If not in repos, check self-provided specs
        if el10_version.is_none() {
            for spec in spec_files {
                for name in package_names.iter() {
                    if &spec.main_package.name == name {
                        el10_version = Some(spec.main_package.version.clone());
                        el10_source = Some(spec.source_repo.clone());
                        break;
                    }
                    for sub in &spec.subpackages {
                        if &sub.name == name {
                            el10_version = Some(sub.version.clone());
                            el10_source = Some(spec.source_repo.clone());
                            break;
                        }
                    }
                }
                if el10_version.is_some() {
                    break;
                }
            }
        }

        // Collect version requirements from all specs
        let mut required_versions = Vec::new();
        for spec in spec_files {
            collect_version_reqs(
                &spec.main_package,
                package_names,
                &mut required_versions,
            );
            for sub in &spec.subpackages {
                collect_version_reqs(sub, package_names, &mut required_versions);
            }
        }
        required_versions.sort();
        required_versions.dedup();

        entries.push(RuntimeEntry {
            name: display_name.into(),
            el10_version,
            el10_source,
            required_versions,
        });
    }

    entries
}

fn collect_version_reqs(
    pkg: &crate::spec::types::Package,
    names: &[&str],
    out: &mut Vec<String>,
) {
    for dep in pkg.requires.iter().chain(pkg.build_requires.iter()) {
        for entry in &dep.entries {
            let dep_name = &entry.name;
            let matches = names.iter().any(|n| dep_name == n)
                || names.iter().any(|n| {
                    // Also check virtual provides like rubygem(rails), python3dist(django)
                    dep_name == &format!("rubygem({})", n.strip_prefix("rubygem-").unwrap_or(""))
                        || dep_name == &format!("python3dist({})", n.strip_prefix("python-").or_else(|| n.strip_prefix("python3.12-")).unwrap_or(""))
                });

            if matches {
                if let (Some(flags), Some(ver)) = (&entry.flags, &entry.version) {
                    let op = match flags {
                        crate::spec::types::VersionFlags::EQ => "=",
                        crate::spec::types::VersionFlags::LT => "<",
                        crate::spec::types::VersionFlags::GT => ">",
                        crate::spec::types::VersionFlags::LE => "<=",
                        crate::spec::types::VersionFlags::GE => ">=",
                    };
                    out.push(format!("{op} {ver}"));
                }
            }
        }
    }
}
