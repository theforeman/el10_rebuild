use crate::analysis::resolver::Universe;
use crate::report::types::*;
use crate::spec::types::SpecFile;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn write_gap_report(
    report: &GapReport,
    output_dir: &Path,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)?;
    let mut out = String::new();

    // Header
    out.push_str("# EL10 Gap Analysis Report\n\n");

    // Executive Summary
    out.push_str("## Executive Summary\n\n");
    out.push_str(&format!("| Metric | Count |\n|--------|-------|\n"));
    out.push_str(&format!("| Spec files analyzed | {} |\n", report.total_specs));
    out.push_str(&format!(
        "| Total packages (including subpackages) | {} |\n",
        report.total_packages
    ));
    out.push_str(&format!("| Total Requires | {} |\n", report.total_requires));
    out.push_str(&format!(
        "| Total BuildRequires | {} |\n",
        report.total_build_requires
    ));
    out.push_str(&format!(
        "| Satisfied Requires | {} ({:.1}%) |\n",
        report.satisfied_requires,
        pct(report.satisfied_requires, report.total_requires)
    ));
    out.push_str(&format!(
        "| Satisfied BuildRequires | {} ({:.1}%) |\n",
        report.satisfied_build_requires,
        pct(report.satisfied_build_requires, report.total_build_requires)
    ));
    out.push_str(&format!(
        "| **Missing dependencies** | **{}** |\n",
        report.missing.len()
    ));
    out.push_str(&format!(
        "| Version mismatches | {} |\n",
        report.version_mismatches.len()
    ));
    out.push_str(&format!(
        "| Unresolved macros | {} |\n",
        report.unresolved_macros.len()
    ));
    out.push_str("\n");

    out.push_str("### Repos Analyzed\n\n");
    for repo in &report.repos_analyzed {
        out.push_str(&format!(
            "- **{}** (`{}`): {} specs, {} packages\n",
            repo.name, repo.path, repo.spec_count, repo.package_count
        ));
    }
    out.push_str("\n");

    out.push_str("### EL10 Repos Queried\n\n");
    for repo in &report.el10_repos {
        out.push_str(&format!("- {repo}\n"));
    }
    out.push_str("\n");

    // Runtime Version Matrix
    out.push_str("## Runtime Version Matrix\n\n");
    out.push_str("| Runtime | EL10 Version | Source | Required Versions |\n");
    out.push_str("|---------|-------------|--------|-------------------|\n");
    for rt in &report.runtime_matrix {
        let ver = rt.el10_version.as_deref().unwrap_or("N/A");
        let src = rt.el10_source.as_deref().unwrap_or("-");
        let reqs = if rt.required_versions.is_empty() {
            "-".into()
        } else {
            rt.required_versions.join(", ")
        };
        out.push_str(&format!("| {} | {} | {} | {} |\n", rt.name, ver, src, reqs));
    }
    out.push_str("\n");

    // Missing Dependencies
    if !report.missing.is_empty() {
        out.push_str("## Missing Dependencies\n\n");
        out.push_str("Dependencies not found in EL10 repos or self-provided by packaging repos.\n\n");
        out.push_str("| Dependency | Type | Virtual | Required By (count) | Example Packages |\n");
        out.push_str("|-----------|------|---------|--------------------|-----------------|\n");

        let mut sorted = report.missing.clone();
        sorted.sort_by(|a, b| b.required_by.len().cmp(&a.required_by.len()));

        for dep in &sorted {
            let virt = if dep.is_virtual { "yes" } else { "no" };
            let count = dep.required_by.len();
            let examples: Vec<_> = dep
                .required_by
                .iter()
                .take(3)
                .map(|r| r.package.as_str())
                .collect();
            let examples_str = examples.join(", ");
            let more = if count > 3 {
                format!(" (+{})", count - 3)
            } else {
                String::new()
            };
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {}{} |\n",
                dep.name, dep.dep_type, virt, count, examples_str, more
            ));
        }
        out.push_str("\n");
    }

    // Version Mismatches
    if !report.version_mismatches.is_empty() {
        out.push_str("## Version Mismatches\n\n");
        out.push_str("Dependencies found in EL10 but at a version that doesn't satisfy constraints.\n\n");
        out.push_str("| Dependency | Available | Source | Required By (count) | Constraints |\n");
        out.push_str("|-----------|-----------|--------|--------------------|-----------|\n");

        for mm in &report.version_mismatches {
            let count = mm.required_by.len();
            let constraints: Vec<_> = mm
                .required_by
                .iter()
                .filter_map(|r| r.version_constraint.as_deref())
                .take(3)
                .collect();
            let constraints_str = if constraints.is_empty() {
                "-".into()
            } else {
                constraints.join("; ")
            };
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} |\n",
                mm.dep_name, mm.available_version, mm.available_source, count, constraints_str
            ));
        }
        out.push_str("\n");
    }

    // Overlapping Packages
    if !report.overlapping_packages.is_empty() {
        out.push_str("## Overlapping Packages\n\n");
        out.push_str("Packages present in multiple packaging repos.\n\n");
        out.push_str("| Package | Repos | Versions |\n");
        out.push_str("|---------|-------|----------|\n");
        for op in &report.overlapping_packages {
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                op.name,
                op.repos.join(", "),
                op.versions.join(", ")
            ));
        }
        out.push_str("\n");
    }

    // Unresolved Macros
    if !report.unresolved_macros.is_empty() {
        out.push_str("## Unresolved Macros\n\n");
        out.push_str("Dependencies containing unexpanded RPM macros.\n\n");
        out.push_str("| Raw | Spec | Package |\n");
        out.push_str("|-----|------|---------|\n");
        for um in &report.unresolved_macros {
            out.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                um.raw, um.spec_path, um.package
            ));
        }
        out.push_str("\n");
    }

    // Parse Warnings
    if !report.parse_warnings.is_empty() {
        out.push_str("## Parse Warnings\n\n");
        for pw in &report.parse_warnings {
            out.push_str(&format!("- `{}`: {}\n", pw.spec_path, pw.message));
        }
        out.push_str("\n");
    }

    let path = output_dir.join("gap-analysis.md");
    fs::write(&path, &out)?;
    eprintln!("  Wrote {}", path.display());
    Ok(())
}

pub fn build_gap_report(
    specs: &[SpecFile],
    universe: &Universe,
    el10_repo_names: &[String],
    runtime_matrix: Vec<RuntimeEntry>,
) -> GapReport {
    let mut total_requires = 0usize;
    let mut total_build_requires = 0usize;
    let mut satisfied_requires = 0usize;
    let mut satisfied_build_requires = 0usize;

    let mut missing_map: HashMap<String, MissingDep> = HashMap::new();
    let mut mismatch_map: HashMap<String, VersionMismatchEntry> = HashMap::new();
    let mut unresolved = Vec::new();
    let mut warnings = Vec::new();

    let mut repo_stats: HashMap<String, (usize, usize)> = HashMap::new();

    for spec in specs {
        let repo_entry = repo_stats.entry(spec.source_repo.clone()).or_default();
        repo_entry.0 += 1;
        repo_entry.1 += 1 + spec.subpackages.len();

        for pw in &spec.parse_warnings {
            warnings.push(ParseWarning {
                spec_path: spec.path.display().to_string(),
                message: pw.clone(),
            });
        }

        let all_pkgs = std::iter::once(&spec.main_package).chain(spec.subpackages.iter());
        for pkg in all_pkgs {
            process_deps(
                &pkg.requires,
                "Requires",
                pkg,
                spec,
                universe,
                &mut total_requires,
                &mut satisfied_requires,
                &mut missing_map,
                &mut mismatch_map,
                &mut unresolved,
            );
            process_deps(
                &pkg.build_requires,
                "BuildRequires",
                pkg,
                spec,
                universe,
                &mut total_build_requires,
                &mut satisfied_build_requires,
                &mut missing_map,
                &mut mismatch_map,
                &mut unresolved,
            );
        }
    }

    // Detect overlapping packages
    let mut pkg_repos: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for spec in specs {
        pkg_repos
            .entry(spec.main_package.name.clone())
            .or_default()
            .push((spec.source_repo.clone(), spec.main_package.version.clone()));
    }
    let overlapping: Vec<OverlappingPackage> = pkg_repos
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(name, entries)| OverlappingPackage {
            name,
            repos: entries.iter().map(|(r, _)| r.clone()).collect(),
            versions: entries.iter().map(|(_, v)| v.clone()).collect(),
        })
        .collect();

    let repos_analyzed: Vec<RepoSummary> = repo_stats
        .into_iter()
        .map(|(name, (spec_count, pkg_count))| RepoSummary {
            name: name.clone(),
            path: name,
            spec_count,
            package_count: pkg_count,
        })
        .collect();

    let total_packages = specs
        .iter()
        .map(|s| 1 + s.subpackages.len())
        .sum();

    GapReport {
        repos_analyzed,
        el10_repos: el10_repo_names.to_vec(),
        total_specs: specs.len(),
        total_packages,
        total_requires,
        total_build_requires,
        satisfied_requires,
        satisfied_build_requires,
        missing: missing_map.into_values().collect(),
        version_mismatches: mismatch_map.into_values().collect(),
        runtime_matrix,
        overlapping_packages: overlapping,
        unresolved_macros: unresolved,
        parse_warnings: warnings,
    }
}

fn process_deps(
    deps: &[crate::spec::types::Dependency],
    dep_type: &str,
    pkg: &crate::spec::types::Package,
    spec: &SpecFile,
    universe: &Universe,
    total: &mut usize,
    satisfied: &mut usize,
    missing_map: &mut HashMap<String, MissingDep>,
    mismatch_map: &mut HashMap<String, VersionMismatchEntry>,
    unresolved: &mut Vec<UnresolvedMacro>,
) {
    for dep in deps {
        *total += 1;
        let result = universe.resolve_dep(dep);

        let dep_name = dep
            .entries
            .first()
            .map(|e| e.name.as_str())
            .unwrap_or(&dep.raw_line);

        let constraint = dep
            .entries
            .first()
            .and_then(|e| {
                e.flags.as_ref().zip(e.version.as_ref()).map(|(f, v)| {
                    let op = match f {
                        crate::spec::types::VersionFlags::EQ => "=",
                        crate::spec::types::VersionFlags::LT => "<",
                        crate::spec::types::VersionFlags::GT => ">",
                        crate::spec::types::VersionFlags::LE => "<=",
                        crate::spec::types::VersionFlags::GE => ">=",
                    };
                    format!("{op} {v}")
                })
            });

        match result {
            ResolutionResult::Satisfied { .. } => {
                *satisfied += 1;
            }
            ResolutionResult::Missing => {
                let entry = missing_map
                    .entry(dep_name.to_string())
                    .or_insert_with(|| MissingDep {
                        name: dep_name.to_string(),
                        dep_type: dep_type.into(),
                        is_virtual: dep_name.contains('('),
                        required_by: Vec::new(),
                    });
                entry.required_by.push(RequiredByEntry {
                    package: pkg.name.clone(),
                    source_repo: spec.source_repo.clone(),
                    version_constraint: constraint,
                });
            }
            ResolutionResult::VersionMismatch {
                source,
                available_version,
                ..
            } => {
                let entry = mismatch_map
                    .entry(dep_name.to_string())
                    .or_insert_with(|| VersionMismatchEntry {
                        dep_name: dep_name.to_string(),
                        available_version: available_version.clone(),
                        available_source: source.clone(),
                        required_by: Vec::new(),
                    });
                entry.required_by.push(RequiredByEntry {
                    package: pkg.name.clone(),
                    source_repo: spec.source_repo.clone(),
                    version_constraint: constraint,
                });
            }
            ResolutionResult::UnresolvedMacro { raw } => {
                *satisfied += 1; // Don't count as missing
                unresolved.push(UnresolvedMacro {
                    raw,
                    spec_path: spec.path.display().to_string(),
                    package: pkg.name.clone(),
                });
            }
        }
    }
}

fn pct(n: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (n as f64 / total as f64) * 100.0
    }
}
