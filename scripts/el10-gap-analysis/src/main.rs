mod analysis;
mod repo;
mod report;
mod spec;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "el10-gap-analysis")]
#[command(about = "EL10 package manifest and dependency gap analysis for Foreman/Katello/Pulp")]
struct Cli {
    /// Paths to packaging repos (e.g., workspace/foreman-packaging workspace/pulpcore-packaging)
    #[arg(short, long, required = true, num_args = 1..)]
    repos: Vec<PathBuf>,

    /// Output directory for reports
    #[arg(short, long, default_value = "reports")]
    output: PathBuf,

    /// Cache directory for downloaded repodata
    #[arg(long, default_value = "cache")]
    cache_dir: PathBuf,

    /// Skip downloading repodata (use cached files)
    #[arg(long)]
    skip_download: bool,

    /// Include EPEL 10 repository
    #[arg(long)]
    epel: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    eprintln!("=== EL10 Gap Analysis ===\n");

    // Phase 1: Discover and parse specs
    eprintln!("[Phase 1] Discovering and parsing spec files...");
    let mut all_specs = Vec::new();

    for repo_path in &cli.repos {
        let repo_name = spec::repo_name_from_path(repo_path);
        eprintln!("  Scanning {repo_name}...");

        let spec_paths = spec::discover_specs(repo_path)
            .with_context(|| format!("Failed to discover specs in {}", repo_path.display()))?;

        eprintln!("  Found {} spec files", spec_paths.len());

        let pb = ProgressBar::new(spec_paths.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  Parsing [{bar:40}] {pos}/{len} specs")
                .unwrap(),
        );

        let specs: Vec<spec::types::SpecFile> = spec_paths
            .par_iter()
            .map(|path| {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                let mut parsed = spec::parser::parse_spec(&content, path);
                parsed.source_repo = repo_name.clone();
                parsed.category = spec::category_from_spec_path(repo_path, path);
                pb.inc(1);
                parsed
            })
            .collect();

        pb.finish_and_clear();
        eprintln!(
            "  Parsed {} specs ({} packages total)",
            specs.len(),
            specs.iter().map(|s| 1 + s.subpackages.len()).sum::<usize>()
        );
        all_specs.extend(specs);
    }

    eprintln!();

    // Phase 2: Download and parse repodata
    eprintln!("[Phase 2] Fetching EL10 repodata...");
    let mut repo_configs = repo::types::RepoConfig::centos_stream_10();
    if cli.epel {
        repo_configs.push(repo::types::RepoConfig::epel10());
    }

    let mut all_repo_packages: Vec<(String, Vec<repo::types::RepoPackage>)> = Vec::new();

    for config in &repo_configs {
        if cli.skip_download && !cli.cache_dir.join(format!("{}-primary.xml.gz", config.name)).exists() {
            eprintln!("  Skipping {} (no cached data)", config.name);
            continue;
        }

        match repo::repodata::fetch_repodata(config, &cli.cache_dir) {
            Ok(packages) => {
                eprintln!("  {}: {} packages", config.name, packages.len());
                all_repo_packages.push((config.name.clone(), packages));
            }
            Err(e) => {
                eprintln!("  Warning: Failed to fetch {}: {e}", config.name);
                if config.name == "EPEL10" {
                    eprintln!("  (EPEL 10 is optional, continuing without it)");
                }
            }
        }
    }

    eprintln!();

    // Phase 3: Build universe and cross-reference
    eprintln!("[Phase 3] Building package universe and resolving dependencies...");
    let mut universe = analysis::resolver::Universe::new();

    // Add EL10 repo packages
    for (name, pkgs) in &all_repo_packages {
        universe.add_repo_packages(pkgs, name);
    }

    // Add self-provided packages from each packaging repo
    for repo_path in &cli.repos {
        let repo_name = spec::repo_name_from_path(repo_path);
        let repo_specs: Vec<_> = all_specs
            .iter()
            .filter(|s| s.source_repo == repo_name)
            .cloned()
            .collect();
        universe.add_spec_packages(&repo_specs, &repo_name);
    }

    // Detect runtime versions
    let repo_pkg_refs: Vec<(&str, &[repo::types::RepoPackage])> = all_repo_packages
        .iter()
        .map(|(name, pkgs)| (name.as_str(), pkgs.as_slice()))
        .collect();
    let runtime_matrix = analysis::runtime::detect_runtimes(&repo_pkg_refs, &all_specs);

    // Build gap report
    let el10_repo_names: Vec<String> = all_repo_packages.iter().map(|(n, _)| n.clone()).collect();
    let gap_report = report::markdown::build_gap_report(
        &all_specs,
        &universe,
        &el10_repo_names,
        runtime_matrix,
    );

    eprintln!(
        "  Resolved {} Requires, {} BuildRequires",
        gap_report.total_requires, gap_report.total_build_requires
    );
    eprintln!(
        "  Missing: {}, Version mismatches: {}",
        gap_report.missing.len(),
        gap_report.version_mismatches.len()
    );
    eprintln!();

    // Phase 4: Generate reports
    eprintln!("[Phase 4] Generating reports...");
    report::manifest::write_manifest(&all_specs, &universe, &cli.output)?;
    report::markdown::write_gap_report(&gap_report, &cli.output)?;

    // Also write the raw gap report as JSON
    let json_path = cli.output.join("gap-analysis.json");
    let json = serde_json::to_string_pretty(&gap_report)?;
    std::fs::write(&json_path, &json)?;
    eprintln!("  Wrote {}", json_path.display());

    eprintln!("\n=== Done! ===");
    eprintln!("Reports written to {}/", cli.output.display());

    if !gap_report.missing.is_empty() {
        eprintln!(
            "\n{} missing dependencies found. See gap-analysis.md for details.",
            gap_report.missing.len()
        );
    }

    Ok(())
}
