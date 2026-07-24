use crate::analysis::version::version_satisfies;
use crate::repo::types::RepoPackage;
use crate::report::types::ResolutionResult;
use crate::spec::types::*;
use std::collections::HashMap;

/// Strip epoch prefix from a version string (e.g., "1:2.0.0" -> "2.0.0").
fn strip_epoch(version: &str) -> &str {
    if let Some(pos) = version.find(':') {
        let prefix = &version[..pos];
        if prefix.chars().all(|c| c.is_ascii_digit()) {
            return &version[pos + 1..];
        }
    }
    version
}

pub struct Universe {
    packages: HashMap<String, Vec<AvailablePackage>>,
    provides: HashMap<String, Vec<AvailableProvide>>,
    files: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct AvailablePackage {
    version: String,
    release: String,
    epoch: u32,
    source: String,
}

#[derive(Debug, Clone)]
struct AvailableProvide {
    version: Option<String>,
    release: Option<String>,
    epoch: Option<u32>,
    flags: Option<String>,
    source: String,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
            provides: HashMap::new(),
            files: HashMap::new(),
        }
    }

    pub fn add_repo_packages(&mut self, packages: &[RepoPackage], source: &str) {
        for pkg in packages {
            self.packages
                .entry(pkg.name.clone())
                .or_default()
                .push(AvailablePackage {
                    version: pkg.version.clone(),
                    release: pkg.release.clone(),
                    epoch: pkg.epoch,
                    source: source.into(),
                });

            // Add explicit provides
            for prov in &pkg.provides {
                self.provides
                    .entry(prov.name.clone())
                    .or_default()
                    .push(AvailableProvide {
                        version: prov.version.clone(),
                        release: prov.release.clone(),
                        epoch: prov.epoch,
                        flags: prov.flags.clone(),
                        source: source.into(),
                    });
            }

            // Add file paths
            for f in &pkg.files {
                self.files.insert(f.clone(), pkg.name.clone());
            }
        }
    }

    pub fn add_spec_packages(&mut self, specs: &[SpecFile], source: &str) {
        for spec in specs {
            let pkg = &spec.main_package;
            self.add_spec_package(pkg, source);
            for sub in &spec.subpackages {
                self.add_spec_package(sub, source);
            }
        }
    }

    fn add_spec_package(&mut self, pkg: &Package, source: &str) {
        self.packages
            .entry(pkg.name.clone())
            .or_default()
            .push(AvailablePackage {
                version: pkg.version.clone(),
                release: pkg.release.clone(),
                epoch: pkg.epoch.unwrap_or(0),
                source: source.into(),
            });

        for prov in &pkg.provides {
            self.provides
                .entry(prov.name.clone())
                .or_default()
                .push(AvailableProvide {
                    version: prov.version.clone(),
                    release: None,
                    epoch: None,
                    flags: prov
                        .flags
                        .as_ref()
                        .map(|f| match f {
                            VersionFlags::EQ => "EQ",
                            VersionFlags::LT => "LT",
                            VersionFlags::GT => "GT",
                            VersionFlags::LE => "LE",
                            VersionFlags::GE => "GE",
                        })
                        .map(String::from),
                    source: source.into(),
                });
        }
    }

    pub fn resolve_dep(&self, dep: &Dependency) -> ResolutionResult {
        match dep.kind {
            DepKind::Simple => {
                if dep.entries.is_empty() {
                    return ResolutionResult::Missing;
                }
                self.resolve_entry(&dep.entries[0])
            }
            DepKind::RichWith => {
                // All entries must be satisfied (they refer to the same package with bounds)
                let mut worst = ResolutionResult::Missing;
                for entry in &dep.entries {
                    match self.resolve_entry(entry) {
                        ResolutionResult::Satisfied { source, available_version } => {
                            if matches!(worst, ResolutionResult::Missing) {
                                worst = ResolutionResult::Satisfied { source, available_version };
                            }
                        }
                        other => return other,
                    }
                }
                worst
            }
            DepKind::RichOr => {
                // At least one must be satisfied
                for entry in &dep.entries {
                    if let r @ ResolutionResult::Satisfied { .. } = self.resolve_entry(entry) {
                        return r;
                    }
                }
                ResolutionResult::Missing
            }
            DepKind::RichIf => {
                // The primary dep (first entry) must be satisfiable
                if dep.entries.is_empty() {
                    return ResolutionResult::Missing;
                }
                self.resolve_entry(&dep.entries[0])
            }
        }
    }

    fn resolve_entry(&self, entry: &DepEntry) -> ResolutionResult {
        let name = &entry.name;

        // Check for unresolved macros
        if name.contains("%{") || name.contains("%(") {
            return ResolutionResult::UnresolvedMacro { raw: name.clone() };
        }

        // File path dependency
        if name.starts_with('/') {
            return if self.files.contains_key(name) {
                ResolutionResult::Satisfied {
                    source: "file-provides".into(),
                    available_version: String::new(),
                }
            } else {
                // Many file deps are in packages we don't enumerate files for.
                // Treat as satisfied unless we have reason to doubt.
                ResolutionResult::Satisfied {
                    source: "file-provides (assumed)".into(),
                    available_version: String::new(),
                }
            };
        }

        // Check virtual provides first (rubygem(), npm(), python3dist(), etc.)
        if let Some(provs) = self.provides.get(name) {
            return self.check_version_against_provides(provs, entry);
        }

        // Check by package name
        if let Some(pkgs) = self.packages.get(name) {
            return self.check_version_against_packages(pkgs, entry);
        }

        ResolutionResult::Missing
    }

    fn check_version_against_packages(
        &self,
        pkgs: &[AvailablePackage],
        entry: &DepEntry,
    ) -> ResolutionResult {
        if entry.flags.is_none() || entry.version.is_none() {
            let best = pkgs
                .iter()
                .max_by(|a, b| crate::analysis::version::rpmvercmp(&a.version, &b.version));
            return match best {
                Some(p) => ResolutionResult::Satisfied {
                    source: p.source.clone(),
                    available_version: p.version.clone(),
                },
                None => ResolutionResult::Missing,
            };
        }

        let op = match entry.flags.as_ref().unwrap() {
            VersionFlags::EQ => "=",
            VersionFlags::LT => "<",
            VersionFlags::GT => ">",
            VersionFlags::LE => "<=",
            VersionFlags::GE => ">=",
        };
        let raw_required = entry.version.as_ref().unwrap();
        let required = strip_epoch(raw_required);

        // RPM compares version-release when the constraint includes a release
        let req_has_release = required.contains('-');

        for pkg in pkgs {
            let available = if req_has_release && !pkg.release.is_empty() {
                format!("{}-{}", pkg.version, pkg.release)
            } else {
                pkg.version.clone()
            };
            if version_satisfies(&available, op, required) {
                return ResolutionResult::Satisfied {
                    source: pkg.source.clone(),
                    available_version: pkg.version.clone(),
                };
            }
        }

        let best = pkgs
            .iter()
            .max_by(|a, b| crate::analysis::version::rpmvercmp(&a.version, &b.version));
        match best {
            Some(p) => ResolutionResult::VersionMismatch {
                source: p.source.clone(),
                available_version: p.version.clone(),
                required: format!("{op} {raw_required}"),
            },
            None => ResolutionResult::Missing,
        }
    }

    fn check_version_against_provides(
        &self,
        provs: &[AvailableProvide],
        entry: &DepEntry,
    ) -> ResolutionResult {
        if entry.flags.is_none() || entry.version.is_none() {
            if let Some(p) = provs.first() {
                return ResolutionResult::Satisfied {
                    source: p.source.clone(),
                    available_version: p.version.clone().unwrap_or_default(),
                };
            }
            return ResolutionResult::Missing;
        }

        let op = match entry.flags.as_ref().unwrap() {
            VersionFlags::EQ => "=",
            VersionFlags::LT => "<",
            VersionFlags::GT => ">",
            VersionFlags::LE => "<=",
            VersionFlags::GE => ">=",
        };
        let raw_required = entry.version.as_ref().unwrap();
        let required = strip_epoch(raw_required);
        let req_has_release = required.contains('-');

        for prov in provs {
            if let Some(ref pv) = prov.version {
                let available = if req_has_release {
                    if let Some(ref rel) = prov.release {
                        format!("{}-{}", pv, rel)
                    } else {
                        pv.clone()
                    }
                } else {
                    pv.clone()
                };
                if version_satisfies(&available, op, required) {
                    return ResolutionResult::Satisfied {
                        source: prov.source.clone(),
                        available_version: pv.clone(),
                    };
                }
            } else {
                return ResolutionResult::Satisfied {
                    source: prov.source.clone(),
                    available_version: String::new(),
                };
            }
        }

        let best = provs.first();
        match best {
            Some(p) => ResolutionResult::VersionMismatch {
                source: p.source.clone(),
                available_version: p.version.clone().unwrap_or_default(),
                required: format!("{op} {raw_required}"),
            },
            None => ResolutionResult::Missing,
        }
    }
}
