pub mod macros;
pub mod parser;
pub mod types;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn discover_specs(repo_path: &Path) -> Result<Vec<PathBuf>> {
    let packages_dir = repo_path.join("packages");
    if !packages_dir.exists() {
        anyhow::bail!("No packages/ directory found in {}", repo_path.display());
    }

    let mut specs = Vec::new();
    discover_specs_recursive(&packages_dir, &mut specs)?;
    specs.sort();
    Ok(specs)
}

fn discover_specs_recursive(dir: &Path, specs: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_specs_recursive(&path, specs)?;
        } else if path.extension().is_some_and(|ext| ext == "spec") {
            specs.push(path);
        }
    }
    Ok(())
}

pub fn repo_name_from_path(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".into())
}

pub fn category_from_spec_path(repo_path: &Path, spec_path: &Path) -> String {
    let rel = spec_path.strip_prefix(repo_path).unwrap_or(spec_path);
    let components: Vec<_> = rel.components().collect();
    // packages/<category>/<pkg>/<pkg>.spec -> category
    // packages/<pkg>/<pkg>.spec -> "" (flat layout)
    if components.len() >= 4 {
        components[1].as_os_str().to_string_lossy().into_owned()
    } else {
        String::new()
    }
}
