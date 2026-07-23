use crate::analysis::resolver::Universe;
use crate::report::types::*;
use crate::spec::types::SpecFile;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct ManifestOutput {
    total_specs: usize,
    total_packages: usize,
    packages: Vec<ManifestPackage>,
}

#[derive(Serialize)]
struct ManifestPackage {
    name: String,
    version: String,
    epoch: Option<u32>,
    source_repo: String,
    category: String,
    is_subpackage: bool,
    provides: Vec<String>,
    requires: Vec<ManifestDep>,
    build_requires: Vec<ManifestDep>,
    el10_status: String,
}

#[derive(Serialize)]
struct ManifestDep {
    name: String,
    constraint: Option<String>,
    status: String,
    resolved_source: Option<String>,
    resolved_version: Option<String>,
}

pub fn write_manifest(
    specs: &[SpecFile],
    universe: &Universe,
    output_dir: &Path,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;

    let mut packages = Vec::new();

    for spec in specs {
        let main = build_manifest_package(&spec.main_package, spec, universe);
        packages.push(main);

        for sub in &spec.subpackages {
            let s = build_manifest_package(sub, spec, universe);
            packages.push(s);
        }
    }

    let manifest = ManifestOutput {
        total_specs: specs.len(),
        total_packages: packages.len(),
        packages,
    };

    let json = serde_json::to_string_pretty(&manifest)?;
    let path = output_dir.join("manifest.json");
    fs::write(&path, &json)?;
    eprintln!("  Wrote {}", path.display());
    Ok(())
}

fn build_manifest_package(
    pkg: &crate::spec::types::Package,
    spec: &SpecFile,
    universe: &Universe,
) -> ManifestPackage {
    let requires: Vec<ManifestDep> = pkg
        .requires
        .iter()
        .map(|d| build_manifest_dep(d, universe))
        .collect();

    let build_requires: Vec<ManifestDep> = pkg
        .build_requires
        .iter()
        .map(|d| build_manifest_dep(d, universe))
        .collect();

    let has_gaps = requires.iter().chain(build_requires.iter()).any(|d| {
        d.status == "missing" || d.status == "version_mismatch"
    });

    let has_warnings = requires.iter().chain(build_requires.iter()).any(|d| {
        d.status == "unresolved_macro"
    });

    let status = if has_gaps {
        "has_gaps"
    } else if has_warnings {
        "has_warnings"
    } else {
        "all_deps_satisfied"
    };

    ManifestPackage {
        name: pkg.name.clone(),
        version: pkg.version.clone(),
        epoch: pkg.epoch,
        source_repo: spec.source_repo.clone(),
        category: spec.category.clone(),
        is_subpackage: pkg.is_subpackage,
        provides: pkg
            .provides
            .iter()
            .map(|p| {
                if let (Some(f), Some(v)) = (&p.flags, &p.version) {
                    let op = match f {
                        crate::spec::types::VersionFlags::EQ => "=",
                        crate::spec::types::VersionFlags::LT => "<",
                        crate::spec::types::VersionFlags::GT => ">",
                        crate::spec::types::VersionFlags::LE => "<=",
                        crate::spec::types::VersionFlags::GE => ">=",
                    };
                    format!("{} {op} {v}", p.name)
                } else {
                    p.name.clone()
                }
            })
            .collect(),
        requires,
        build_requires,
        el10_status: status.into(),
    }
}

fn build_manifest_dep(
    dep: &crate::spec::types::Dependency,
    universe: &Universe,
) -> ManifestDep {
    let result = universe.resolve_dep(dep);

    let (name, constraint) = if !dep.entries.is_empty() {
        let e = &dep.entries[0];
        let c = e
            .flags
            .as_ref()
            .zip(e.version.as_ref())
            .map(|(f, v)| {
                let op = match f {
                    crate::spec::types::VersionFlags::EQ => "=",
                    crate::spec::types::VersionFlags::LT => "<",
                    crate::spec::types::VersionFlags::GT => ">",
                    crate::spec::types::VersionFlags::LE => "<=",
                    crate::spec::types::VersionFlags::GE => ">=",
                };
                format!("{op} {v}")
            });
        (e.name.clone(), c)
    } else {
        (dep.raw_line.clone(), None)
    };

    let (status, resolved_source, resolved_version) = match result {
        ResolutionResult::Satisfied {
            source,
            available_version,
        } => ("satisfied".into(), Some(source), Some(available_version)),
        ResolutionResult::VersionMismatch {
            source,
            available_version,
            ..
        } => (
            "version_mismatch".into(),
            Some(source),
            Some(available_version),
        ),
        ResolutionResult::Missing => ("missing".into(), None, None),
        ResolutionResult::UnresolvedMacro { .. } => ("unresolved_macro".into(), None, None),
    };

    ManifestDep {
        name,
        constraint,
        status,
        resolved_source,
        resolved_version,
    }
}
