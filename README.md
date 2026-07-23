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

## Contributing

Open an issue for any rebuild-related problem or task. Reference relevant upstream bugs and package names where applicable.
