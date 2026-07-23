use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct SpecFile {
    pub path: PathBuf,
    pub source_repo: String,
    pub category: String,
    pub main_package: Package,
    pub subpackages: Vec<Package>,
    pub parse_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Package {
    pub name: String,
    pub epoch: Option<u32>,
    pub version: String,
    pub release: String,
    pub requires: Vec<Dependency>,
    pub build_requires: Vec<Dependency>,
    pub provides: Vec<Provide>,
    pub conflicts: Vec<Dependency>,
    pub obsoletes: Vec<Dependency>,
    pub is_subpackage: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dependency {
    pub raw_line: String,
    pub kind: DepKind,
    pub entries: Vec<DepEntry>,
    pub conditional: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum DepKind {
    Simple,
    RichWith,
    RichOr,
    RichIf,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepEntry {
    pub name: String,
    pub flags: Option<VersionFlags>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum VersionFlags {
    EQ,
    LT,
    GT,
    LE,
    GE,
}

#[derive(Debug, Clone, Serialize)]
pub struct Provide {
    pub name: String,
    pub flags: Option<VersionFlags>,
    pub version: Option<String>,
}

impl VersionFlags {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "=" | "==" => Some(VersionFlags::EQ),
            "<" => Some(VersionFlags::LT),
            ">" => Some(VersionFlags::GT),
            "<=" => Some(VersionFlags::LE),
            ">=" => Some(VersionFlags::GE),
            _ => None,
        }
    }
}

impl Package {
    pub fn new(name: String) -> Self {
        Package {
            name,
            epoch: None,
            version: String::new(),
            release: String::new(),
            requires: Vec::new(),
            build_requires: Vec::new(),
            provides: Vec::new(),
            conflicts: Vec::new(),
            obsoletes: Vec::new(),
            is_subpackage: false,
        }
    }
}
