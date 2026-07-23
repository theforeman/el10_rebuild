#!/usr/bin/env python3
"""Check gem runtime dependencies against packaged gems in foreman-packaging.

Walks all rubygem-* package directories, fetches each gem's runtime
dependencies from rubygems.org, and reports any that aren't satisfied
by another package in the repo. Filters out Ruby stdlib gems and flags
deps that have been explicitly obsoleted.

Usage:
    python3 scripts/check_gem_deps.py /path/to/foreman-packaging
"""

import json
import os
import re
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import Request, urlopen

PACKAGES_DIRS = ["packages/foreman", "packages/plugins"]
RUBYGEMS_API = "https://rubygems.org/api/v2/rubygems/{name}/versions/{version}.json"
MAX_WORKERS = 10
OBSOLETE_SPEC = "packages/foreman/foreman-obsolete-packages/foreman-obsolete-packages.spec"

# Gems bundled with Ruby itself — no separate RPM needed
STDLIB_GEMS = {
    "base64", "benchmark", "bigdecimal", "bundler", "cgi", "csv", "date",
    "delegate", "did_you_mean", "digest", "drb", "english", "erb", "etc",
    "fcntl", "fiddle", "fileutils", "find", "forwardable", "getoptlong",
    "io-console", "io-nonblock", "io-wait", "ipaddr", "irb", "json",
    "logger", "minitest", "mutex_m", "net-ftp", "net-http", "net-imap",
    "net-pop", "net-protocol", "net-smtp", "nkf", "observer", "open-uri",
    "open3", "openssl", "optparse", "ostruct", "pathname", "pp",
    "prettyprint", "prime", "pstore", "psych", "racc", "rake", "rbs",
    "rdoc", "readline", "readline-ext", "reline", "resolv", "resolv-replace",
    "rexml", "rinda", "ruby2_keywords", "securerandom", "set", "shellwords",
    "singleton", "stringio", "strscan", "syslog", "tempfile", "time",
    "timeout", "tmpdir", "tsort", "typeprof", "un", "uri", "weakref",
    "win32ole", "yaml", "zlib",
}


def find_packaged_gems(repo_root):
    """Find all rubygem-* packages. Uses .gem filename for version, falls back to dir name."""
    gems = {}
    for pkg_dir in PACKAGES_DIRS:
        full_path = repo_root / pkg_dir
        if not full_path.exists():
            continue
        for entry in sorted(full_path.iterdir()):
            if not entry.is_dir() or not entry.name.startswith("rubygem-"):
                continue
            gem_name_from_dir = entry.name[len("rubygem-"):]
            gem_files = list(entry.glob("*.gem"))
            if gem_files:
                gem_file = gem_files[0].name
                match = re.match(r"^(.+)-(\d+\S*)\.gem$", gem_file)
                if match:
                    gem_name, gem_version = match.groups()
                    gems[gem_name] = {
                        "version": gem_version,
                        "pkg_dir": str(entry.relative_to(repo_root)),
                    }
                    continue
            gems[gem_name_from_dir] = {
                "version": None,
                "pkg_dir": str(entry.relative_to(repo_root)),
            }
    return gems


def parse_obsoletes(repo_root):
    """Parse obsoleted gem names from foreman-obsolete-packages.spec."""
    obsoleted = set()
    spec_path = repo_root / OBSOLETE_SPEC
    if not spec_path.exists():
        return obsoleted
    for line in spec_path.read_text().splitlines():
        m = re.match(r"^Obsoletes:\s+rubygem-(\S+?)(?:-doc)?\s", line)
        if m:
            obsoleted.add(m.group(1).replace("-", "_"))
            obsoleted.add(m.group(1))
    return obsoleted


def normalize_names(name):
    """Return all plausible name variants for matching."""
    return {name, name.replace("-", "_"), name.replace("_", "-")}


def fetch_deps(gem_name, gem_version):
    """Fetch runtime dependencies from RubyGems.org API."""
    url = RUBYGEMS_API.format(name=gem_name, version=gem_version)
    req = Request(url, headers={"Accept": "application/json"})
    try:
        with urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read())
    except HTTPError as e:
        if e.code == 404:
            return gem_name, gem_version, None, "not found on rubygems.org"
        return gem_name, gem_version, None, f"HTTP {e.code}"
    except Exception as e:
        return gem_name, gem_version, None, str(e)

    runtime_deps = []
    for dep in data.get("dependencies", {}).get("runtime", []):
        runtime_deps.append(dep["name"])

    return gem_name, gem_version, runtime_deps, None


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} /path/to/foreman-packaging", file=sys.stderr)
        sys.exit(2)

    repo_root = Path(sys.argv[1]).resolve()
    if not (repo_root / "packages").is_dir():
        print(f"Error: {repo_root}/packages not found", file=sys.stderr)
        sys.exit(2)

    packaged = find_packaged_gems(repo_root)
    obsoleted = parse_obsoletes(repo_root)

    packaged_names = set()
    for name in packaged:
        packaged_names.update(normalize_names(name))

    fetchable = {n: i for n, i in packaged.items() if i["version"]}

    print(f"Found {len(packaged)} rubygem packages ({len(fetchable)} with .gem files)")
    print(f"Found {len(obsoleted)} obsoleted gems")
    print(f"Fetching runtime deps from rubygems.org ({MAX_WORKERS} concurrent)...\n")

    missing = {}
    errors = []
    done = 0

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = {
            pool.submit(fetch_deps, name, info["version"]): name
            for name, info in fetchable.items()
        }
        for future in as_completed(futures):
            gem_name, gem_version, deps, err = future.result()
            done += 1
            if done % 50 == 0:
                print(f"  ... {done}/{len(fetchable)}", file=sys.stderr)

            if err:
                errors.append(f"{gem_name}-{gem_version}: {err}")
                continue

            for dep in deps:
                if dep in STDLIB_GEMS:
                    continue
                if any(v in packaged_names for v in normalize_names(dep)):
                    continue
                if dep not in missing:
                    missing[dep] = []
                missing[dep].append(f"{gem_name}-{gem_version}")

    print()
    if errors:
        print(f"ERRORS ({len(errors)}):")
        for e in sorted(errors):
            print(f"  {e}")
        print()

    if missing:
        print(f"MISSING DEPENDENCIES ({len(missing)}):")
        for dep_name in sorted(missing.keys()):
            needed_by = sorted(missing[dep_name])
            is_obsoleted = any(v in obsoleted for v in normalize_names(dep_name))
            obsolete_flag = " [OBSOLETED]" if is_obsoleted else ""
            print(f"  {dep_name}{obsolete_flag}")
            for pkg in needed_by:
                print(f"    needed by: {pkg}")
    else:
        print("All gem runtime dependencies are satisfied!")

    sys.exit(1 if missing else 0)


if __name__ == "__main__":
    main()
