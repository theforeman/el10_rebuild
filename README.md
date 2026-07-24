# EL10 Rebuild

Coordination hub for the EL10 rebuild effort. This repo tracks issues, scripts, and work logs related to rebuilding packages for Enterprise Linux 10.

## What's Here

- **Issues** — Track rebuild blockers, packaging problems, and tasks
- **Scripts** — Automation and tooling for rebuild workflows
- **Work Log** — Record of progress and decisions made during the rebuild

## Scripts

### check_gem_deps.py

Checks that all rubygem runtime dependencies in foreman-packaging are satisfied by other packages in the repo. Fetches dependency data from rubygems.org (since `.gem` files are git-annex symlinks), filters out Ruby stdlib gems, and flags any missing deps — including ones that have been added to `foreman-obsolete-packages`.

```bash
python3 scripts/check_gem_deps.py /path/to/foreman-packaging
```

Run this before removing any rubygem package to verify it's truly orphaned. Spec files don't always list gem dependencies explicitly (they're auto-generated at RPM build time), so grepping specs for `Requires:` will miss transitive gem deps.

### el10-gap-analysis

A Rust CLI tool that audits RPM spec files from packaging repos and cross-references all dependencies against CentOS Stream 10 repositories. It produces a comprehensive gap analysis report identifying missing dependencies, version mismatches, and runtime version shifts for the EL10 rebuild.

Located at `scripts/el10-gap-analysis/`.

**What it does:**

1. Parses all `.spec` files from packaging repos (macro expansion, `%if` conditional evaluation, rich dependency parsing)
2. Downloads and parses CentOS Stream 10 repodata (BaseOS, AppStream, CRB)
3. Builds a package universe and resolves every Requires/BuildRequires against it
4. Generates reports: `gap-analysis.md`, `gap-analysis.json`, and `manifest.json`

**Prerequisites:**

- Rust toolchain (`rustup` / `cargo`)
- Cloned packaging repos in `workspace/` (use `scripts/setup_workspace.py`)

**Build:**

```bash
cd scripts/el10-gap-analysis
cargo build --release
```

**Usage:**

```bash
# Full run (downloads repodata on first run, caches for 24h)
cargo run --release -- \
  --repos workspace/foreman-packaging workspace/pulpcore-packaging \
  --output reports/

# Use cached repodata (skip download)
cargo run --release -- \
  --repos workspace/foreman-packaging workspace/pulpcore-packaging \
  --output reports/ \
  --skip-download

# Include EPEL 10
cargo run --release -- \
  --repos workspace/foreman-packaging workspace/pulpcore-packaging \
  --output reports/ \
  --epel
```

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--repos` | (required) | Paths to packaging repos to analyze |
| `--output` | `reports/` | Directory for generated reports |
| `--cache-dir` | `cache/` | Directory for cached repodata downloads |
| `--skip-download` | off | Use cached repodata only |
| `--epel` | off | Include EPEL 10 repository in analysis |

**Output files:**

| File | Description |
|------|-------------|
| `reports/gap-analysis.md` | Human-readable gap analysis with executive summary, runtime matrix, missing deps, version mismatches, and unresolved macros |
| `reports/gap-analysis.json` | Machine-readable version of the same data |
| `reports/manifest.json` | Full package manifest with per-dependency resolution status |

## Contributing

Open an issue for any rebuild-related problem or task. Reference relevant upstream bugs and package names where applicable.
