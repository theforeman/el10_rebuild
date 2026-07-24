use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct GapReport {
    pub repos_analyzed: Vec<RepoSummary>,
    pub el10_repos: Vec<String>,
    pub total_specs: usize,
    pub total_packages: usize,
    pub total_requires: usize,
    pub total_build_requires: usize,
    pub satisfied_requires: usize,
    pub satisfied_build_requires: usize,
    pub missing: Vec<MissingDep>,
    pub version_mismatches: Vec<VersionMismatchEntry>,
    pub runtime_matrix: Vec<RuntimeEntry>,
    pub overlapping_packages: Vec<OverlappingPackage>,
    pub unresolved_macros: Vec<UnresolvedMacro>,
    pub parse_warnings: Vec<ParseWarning>,
}

#[derive(Debug, Serialize)]
pub struct RepoSummary {
    pub name: String,
    pub path: String,
    pub spec_count: usize,
    pub package_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MissingDep {
    pub name: String,
    pub dep_type: String,
    pub is_virtual: bool,
    pub required_by: Vec<RequiredByEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RequiredByEntry {
    pub package: String,
    pub source_repo: String,
    pub version_constraint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VersionMismatchEntry {
    pub dep_name: String,
    pub available_version: String,
    pub available_source: String,
    pub required_by: Vec<RequiredByEntry>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeEntry {
    pub name: String,
    pub el10_version: Option<String>,
    pub el10_source: Option<String>,
    pub required_versions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OverlappingPackage {
    pub name: String,
    pub repos: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct UnresolvedMacro {
    pub raw: String,
    pub spec_path: String,
    pub package: String,
}

#[derive(Debug, Serialize)]
pub struct ParseWarning {
    pub spec_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ResolutionResult {
    Satisfied {
        source: String,
        available_version: String,
    },
    VersionMismatch {
        source: String,
        available_version: String,
        required: String,
    },
    Missing,
    UnresolvedMacro {
        raw: String,
    },
}
