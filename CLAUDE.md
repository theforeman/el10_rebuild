# EL10 Rebuild

This repo coordinates the EL10 (Enterprise Linux 10) rebuild effort. It serves as:

- A place to track rebuild-related issues via the GitHub issue tracker
- A home for scripts and automation used during the rebuild
- A log of work being done on the EL10 rebuild

Issues should be filed in the GitHub issue tracker, not as files in the repo.

## Conventions

- Prefer Python for scripts and automation.

## Scripts

- `scripts/check_gem_deps.py` — Checks rubygem runtime deps in foreman-packaging against what's actually packaged. Takes the foreman-packaging repo path as an argument. Uses rubygems.org API since .gem files are git-annex symlinks. Filters Ruby stdlib gems. Run before removing any rubygem package.
