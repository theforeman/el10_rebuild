use crate::spec::macros::{evaluate_condition, MacroExpander};
use crate::spec::types::*;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(Name|Version|Release|Epoch)\s*:\s*(.+)$").unwrap());

static DEP_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(Requires|BuildRequires|Provides|Conflicts|BuildConflicts|Obsoletes)(?:\([^)]*\))?\s*:\s*(.+)$").unwrap()
});

static PACKAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^%package\s+(.+)$").unwrap());

static IF_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^%if\s+(.+)$").unwrap());

static SIMPLE_DEP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\S+)\s*(>=|<=|>|<|=)\s*(\S+)$").unwrap());

pub fn parse_spec(content: &str, path: &Path) -> SpecFile {
    let warnings = Vec::new();

    let content = join_continuations(content);

    let mut expander = MacroExpander::new();
    expander.scan_definitions(&content);

    // First pass: extract Name, Version, Release before full parsing
    let (raw_name, raw_version, raw_release, raw_epoch) = extract_metadata(&content);
    let name = expander.expand(&raw_name);
    let version = expander.expand(&raw_version);
    let release = expander.expand(&raw_release);
    let epoch = raw_epoch.as_ref().and_then(|e| {
        let expanded = expander.expand(&e);
        expanded.trim().parse::<u32>().ok()
    });

    if let Some(ref e) = raw_epoch {
        let expanded_epoch = expander.expand(e);
        expander.set_epoch(&expanded_epoch);
    }
    expander.set_name_version(&name, &version, &release);

    let mut main_pkg = Package::new(name.clone());
    main_pkg.version = version;
    main_pkg.release = release;
    main_pkg.epoch = epoch;

    let mut subpackages: Vec<Package> = Vec::new();
    let mut current_pkg: Option<usize> = None; // None = main, Some(idx) = subpackage

    let mut if_stack: Vec<Option<bool>> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Handle %if/%else/%endif
        if let Some(caps) = IF_RE.captures(trimmed) {
            let cond = &caps[1];
            let result = evaluate_condition(cond, &expander);
            if_stack.push(result);
            continue;
        }
        if trimmed == "%else" {
            if let Some(last) = if_stack.last_mut() {
                *last = last.map(|v| !v);
            }
            continue;
        }
        if trimmed == "%endif" {
            if_stack.pop();
            continue;
        }

        // Check if we're in an inactive conditional block
        let in_active_block = if_stack.iter().all(|v| v.unwrap_or(true));
        let in_unevaluated = if_stack.iter().any(|v| v.is_none());

        if !in_active_block && !in_unevaluated {
            continue;
        }

        // Stop parsing deps at %description, %prep, %build, %install, %files, %changelog
        if trimmed.starts_with("%description")
            || trimmed.starts_with("%prep")
            || trimmed.starts_with("%build")
            || trimmed.starts_with("%install")
            || trimmed.starts_with("%files")
            || trimmed.starts_with("%changelog")
            || trimmed.starts_with("%check")
        {
            // Reset to main package context at section boundaries
            // (subpackage deps are only in the preamble between %package and %description)
            if trimmed.starts_with("%description") || trimmed.starts_with("%files") {
                // These can be for subpackages too, but deps come before them
            }
            continue;
        }

        // Handle %package
        if let Some(caps) = PACKAGE_RE.captures(trimmed) {
            let pkg_suffix = caps[1].trim();
            let sub_name = if pkg_suffix.starts_with("-n ") || pkg_suffix.starts_with("-n\t") {
                let raw = pkg_suffix.strip_prefix("-n").unwrap().trim();
                expander.expand(raw)
            } else {
                format!("{}-{}", main_pkg.name, expander.expand(pkg_suffix))
            };
            let mut sub = Package::new(sub_name);
            sub.is_subpackage = true;
            sub.version = main_pkg.version.clone();
            sub.release = main_pkg.release.clone();
            sub.epoch = main_pkg.epoch;
            subpackages.push(sub);
            current_pkg = Some(subpackages.len() - 1);
            continue;
        }

        // Parse dependency directives
        if let Some(caps) = DEP_RE.captures(trimmed) {
            let directive = &caps[1];
            let dep_str = caps[2].trim();
            let expanded = expander.expand(dep_str);

            let pkg = match current_pkg {
                None => &mut main_pkg,
                Some(idx) => &mut subpackages[idx],
            };

            let deps = parse_dependency_line(&expanded, in_unevaluated);

            for dep in deps {
                match directive {
                    "Requires" => pkg.requires.push(dep),
                    "BuildRequires" => pkg.build_requires.push(dep),
                    "Provides" => {
                        for entry in &dep.entries {
                            pkg.provides.push(Provide {
                                name: entry.name.clone(),
                                flags: entry.flags.clone(),
                                version: entry.version.clone(),
                            });
                        }
                    }
                    "Conflicts" | "BuildConflicts" => pkg.conflicts.push(dep),
                    "Obsoletes" => pkg.obsoletes.push(dep),
                    _ => {}
                }
            }
        }
    }

    // Infer implicit Provides for rubygem packages
    infer_provides(&mut main_pkg, &expander);
    for sub in &mut subpackages {
        infer_provides(sub, &expander);
    }

    SpecFile {
        path: path.to_path_buf(),
        source_repo: String::new(),
        category: String::new(),
        main_package: main_pkg,
        subpackages,
        parse_warnings: warnings,
    }
}

fn join_continuations(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut continuation = String::new();

    for line in content.lines() {
        if line.ends_with('\\') {
            continuation.push_str(&line[..line.len() - 1]);
            continuation.push(' ');
        } else if !continuation.is_empty() {
            continuation.push_str(line);
            result.push_str(&continuation);
            result.push('\n');
            continuation.clear();
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    if !continuation.is_empty() {
        result.push_str(&continuation);
        result.push('\n');
    }
    result
}

fn extract_metadata(content: &str) -> (String, String, String, Option<String>) {
    let mut name = String::new();
    let mut version = String::new();
    let mut release = String::new();
    let mut epoch = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(caps) = TAG_RE.captures(trimmed) {
            let tag = &caps[1];
            let val = caps[2].trim().to_string();
            match tag {
                "Name" => name = val,
                "Version" => version = val,
                "Release" => release = val,
                "Epoch" => epoch = Some(val),
                _ => {}
            }
        }
        if trimmed.starts_with("%description") {
            break;
        }
    }
    (name, version, release, epoch)
}

fn parse_dependency_line(line: &str, conditional: bool) -> Vec<Dependency> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }

    // Rich dependency: starts with (
    if line.starts_with('(') {
        if let Some(dep) = parse_rich_dep(line, conditional) {
            return vec![dep];
        }
    }

    // May be multiple deps separated by commas or spaces (rare but possible)
    // Most specs have one dep per line, but some use comma separation
    let parts: Vec<&str> = if line.contains(',') {
        line.split(',').map(|s| s.trim()).collect()
    } else {
        vec![line]
    };

    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .filter_map(|part| {
            let part = part.trim();
            if part.starts_with('(') {
                parse_rich_dep(part, conditional)
            } else {
                parse_simple_dep(part, conditional)
            }
        })
        .collect()
}

fn parse_simple_dep(s: &str, conditional: bool) -> Option<Dependency> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let entry = if let Some(caps) = SIMPLE_DEP_RE.captures(s) {
        DepEntry {
            name: caps[1].to_string(),
            flags: VersionFlags::from_str(&caps[2]),
            version: Some(caps[3].to_string()),
        }
    } else {
        DepEntry {
            name: s.to_string(),
            flags: None,
            version: None,
        }
    };

    Some(Dependency {
        raw_line: s.to_string(),
        kind: DepKind::Simple,
        entries: vec![entry],
        conditional,
    })
}

fn parse_rich_dep(s: &str, conditional: bool) -> Option<Dependency> {
    // Strip outer parens
    let inner = s.trim().strip_prefix('(')?.strip_suffix(')')?.trim();

    // Try: A with B
    if let Some((left, right)) = split_rich_op(inner, " with ") {
        let a = parse_dep_entry(left)?;
        let b = parse_dep_entry(right)?;
        return Some(Dependency {
            raw_line: s.to_string(),
            kind: DepKind::RichWith,
            entries: vec![a, b],
            conditional,
        });
    }

    // Try: A or B
    if let Some((left, right)) = split_rich_op(inner, " or ") {
        let a = parse_dep_entry(left)?;
        let b = parse_dep_entry(right)?;
        return Some(Dependency {
            raw_line: s.to_string(),
            kind: DepKind::RichOr,
            entries: vec![a, b],
            conditional,
        });
    }

    // Try: A if B
    if let Some((left, right)) = split_rich_op(inner, " if ") {
        let a = parse_dep_entry(left)?;
        let b = parse_dep_entry(right.trim_start_matches('(').trim_end_matches(')'))?;
        return Some(Dependency {
            raw_line: s.to_string(),
            kind: DepKind::RichIf,
            entries: vec![a, b],
            conditional,
        });
    }

    // Single dep in parens
    let entry = parse_dep_entry(inner)?;
    Some(Dependency {
        raw_line: s.to_string(),
        kind: DepKind::Simple,
        entries: vec![entry],
        conditional,
    })
}

fn split_rich_op<'a>(s: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    // Find the operator outside of nested parens
    let mut depth = 0;
    let op_bytes = op.as_bytes();
    let s_bytes = s.as_bytes();

    for i in 0..s_bytes.len() {
        if s_bytes[i] == b'(' {
            depth += 1;
        } else if s_bytes[i] == b')' {
            depth -= 1;
        } else if depth == 0 && i + op_bytes.len() <= s_bytes.len() {
            if &s_bytes[i..i + op_bytes.len()] == op_bytes {
                return Some((&s[..i], &s[i + op.len()..]));
            }
        }
    }
    None
}

fn parse_dep_entry(s: &str) -> Option<DepEntry> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if let Some(caps) = SIMPLE_DEP_RE.captures(s) {
        Some(DepEntry {
            name: caps[1].to_string(),
            flags: VersionFlags::from_str(&caps[2]),
            version: Some(caps[3].to_string()),
        })
    } else {
        Some(DepEntry {
            name: s.to_string(),
            flags: None,
            version: None,
        })
    }
}

fn infer_provides(pkg: &mut Package, expander: &MacroExpander) {
    let has_rubygems_devel = pkg
        .build_requires
        .iter()
        .any(|d| d.entries.iter().any(|e| e.name == "rubygems-devel"));

    if has_rubygems_devel || pkg.name.starts_with("rubygem-") {
        let gem_name = expander
            .get("gem_name")
            .map(String::from)
            .unwrap_or_else(|| {
                pkg.name
                    .strip_prefix("rubygem-")
                    .unwrap_or(&pkg.name)
                    .to_string()
            });

        let already_provided = pkg.provides.iter().any(|p| {
            p.name.starts_with("rubygem(") || p.name == format!("rubygem-{gem_name}")
        });

        if !already_provided {
            pkg.provides.push(Provide {
                name: format!("rubygem({gem_name})"),
                flags: Some(VersionFlags::EQ),
                version: Some(pkg.version.clone()),
            });
        }
    }

    // Infer npm() provides for nodejs packages
    if pkg.name.starts_with("nodejs-") {
        let npm_name = expander
            .get("npm_name")
            .map(String::from)
            .unwrap_or_else(|| {
                pkg.name
                    .strip_prefix("nodejs-")
                    .unwrap_or(&pkg.name)
                    .to_string()
            });

        let already_provided = pkg.provides.iter().any(|p| p.name.starts_with("npm("));
        if !already_provided {
            pkg.provides.push(Provide {
                name: format!("npm({npm_name})"),
                flags: Some(VersionFlags::EQ),
                version: Some(pkg.version.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dep() {
        let deps = parse_dependency_line("ruby >= 2.7", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].entries[0].name, "ruby");
        assert_eq!(deps[0].entries[0].flags, Some(VersionFlags::GE));
        assert_eq!(deps[0].entries[0].version.as_deref(), Some("2.7"));
    }

    #[test]
    fn test_parse_simple_no_version() {
        let deps = parse_dependency_line("rubygems-devel", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].entries[0].name, "rubygems-devel");
        assert_eq!(deps[0].entries[0].flags, None);
    }

    #[test]
    fn test_parse_rich_with() {
        let deps =
            parse_dependency_line("(rubygem(rails) >= 7.0.3 with rubygem(rails) < 7.1.0)", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].kind, DepKind::RichWith);
        assert_eq!(deps[0].entries.len(), 2);
        assert_eq!(deps[0].entries[0].name, "rubygem(rails)");
        assert_eq!(deps[0].entries[0].flags, Some(VersionFlags::GE));
        assert_eq!(deps[0].entries[1].name, "rubygem(rails)");
        assert_eq!(deps[0].entries[1].flags, Some(VersionFlags::LT));
    }

    #[test]
    fn test_parse_rich_or() {
        let deps = parse_dependency_line("(openvox-agent or puppet-agent)", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].kind, DepKind::RichOr);
        assert_eq!(deps[0].entries[0].name, "openvox-agent");
        assert_eq!(deps[0].entries[1].name, "puppet-agent");
    }

    #[test]
    fn test_parse_rich_if() {
        let deps = parse_dependency_line("(foreman-selinux if selinux-policy-targeted)", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].kind, DepKind::RichIf);
        assert_eq!(deps[0].entries[0].name, "foreman-selinux");
        assert_eq!(deps[0].entries[1].name, "selinux-policy-targeted");
    }

    #[test]
    fn test_parse_file_path_dep() {
        let deps = parse_dependency_line("/usr/bin/psql", false);
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].entries[0].name, "/usr/bin/psql");
    }

    #[test]
    fn test_parse_spec_basic() {
        let content = r#"
%global gem_name test_gem

Name:           rubygem-%{gem_name}
Version:        1.2.3
Release:        1%{?dist}
Summary:        A test gem

BuildRequires:  rubygems-devel
Requires:       ruby >= 2.7
Requires:       (rubygem(activesupport) >= 6.0 with rubygem(activesupport) < 8.0)

%description
Test package.
"#;
        let spec = parse_spec(content, Path::new("test.spec"));
        assert_eq!(spec.main_package.name, "rubygem-test_gem");
        assert_eq!(spec.main_package.version, "1.2.3");
        assert_eq!(spec.main_package.requires.len(), 2);
        assert_eq!(spec.main_package.build_requires.len(), 1);
        // Should have inferred rubygem(test_gem) provide
        assert!(spec
            .main_package
            .provides
            .iter()
            .any(|p| p.name == "rubygem(test_gem)"));
    }

    #[test]
    fn test_parse_subpackage() {
        let content = r#"
Name:           foreman
Version:        3.18.0
Release:        1%{?dist}

Requires:       ruby

%package cli
Requires:       rubygem(hammer_cli)

%description
Main package.
"#;
        let spec = parse_spec(content, Path::new("foreman.spec"));
        assert_eq!(spec.main_package.name, "foreman");
        assert_eq!(spec.subpackages.len(), 1);
        assert_eq!(spec.subpackages[0].name, "foreman-cli");
        assert_eq!(spec.subpackages[0].requires.len(), 1);
    }

    #[test]
    fn test_parse_subpackage_dash_n() {
        let content = r#"
%global python3_pkgversion 3.12
%global pypi_name requests

Name:           python-requests
Version:        2.31.0
Release:        1%{?dist}

%package -n python%{python3_pkgversion}-%{pypi_name}
Requires:       python%{python3_pkgversion}-urllib3

%description
Test.
"#;
        let spec = parse_spec(content, Path::new("test.spec"));
        assert_eq!(spec.subpackages.len(), 1);
        assert_eq!(spec.subpackages[0].name, "python3.12-requests");
    }

    #[test]
    fn test_conditional_rhel() {
        let content = r#"
Name:           test
Version:        1.0
Release:        1

%if 0%{?rhel} >= 8
Requires:       glibc-langpack-en
%endif

%if 0%{?suse_version}
Requires:       suse-only-package
%endif

%description
Test.
"#;
        let spec = parse_spec(content, Path::new("test.spec"));
        let req_names: Vec<_> = spec
            .main_package
            .requires
            .iter()
            .flat_map(|d| d.entries.iter().map(|e| e.name.as_str()))
            .collect();
        assert!(req_names.contains(&"glibc-langpack-en"));
        assert!(!req_names.contains(&"suse-only-package"));
    }

    #[test]
    fn test_join_continuations() {
        let input = "Requires: foo \\\nbar \\\nbaz\nName: test\n";
        let result = join_continuations(input);
        assert!(result.contains("Requires: foo  bar  baz"));
    }
}
