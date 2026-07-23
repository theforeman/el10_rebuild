use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static MACRO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%\{([^}]+)\}").unwrap());

static GLOBAL_DEFINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^%(?:global|define)\s+(\S+)\s+(.*)$").unwrap());

static BCOND_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^%bcond(?:_with(?:out)?)?\s+(\S+)(?:\s+(\d+))?").unwrap());

static PERCENT_BARE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"%([a-zA-Z_]\w*)(?![({])").unwrap());

pub struct MacroExpander {
    macros: HashMap<String, String>,
    bconds: HashMap<String, bool>,
}

impl MacroExpander {
    pub fn new() -> Self {
        let mut macros = HashMap::new();
        // EL10 system defaults
        macros.insert("rhel".into(), "10".into());
        macros.insert("dist".into(), ".el10".into());
        macros.insert("?dist".into(), ".el10".into());

        MacroExpander {
            macros,
            bconds: HashMap::new(),
        }
    }

    pub fn scan_definitions(&mut self, content: &str) {
        for line in content.lines() {
            let trimmed = line.trim();

            if let Some(caps) = BCOND_RE.captures(trimmed) {
                let name = caps[1].to_string();
                if trimmed.starts_with("%bcond_without") {
                    // %bcond_without X = X is ON by default
                    self.bconds.insert(name, true);
                } else if trimmed.starts_with("%bcond_with ") || trimmed.starts_with("%bcond_with\t")
                {
                    // %bcond_with X = X is OFF by default
                    self.bconds.insert(name, false);
                } else if trimmed.starts_with("%bcond ") || trimmed.starts_with("%bcond\t") {
                    // %bcond X 0 = OFF, %bcond X 1 = ON
                    let val = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
                    self.bconds.insert(name, val != "0");
                }
            }

            if let Some(caps) = GLOBAL_DEFINE_RE.captures(trimmed) {
                let name = caps[1].to_string();
                let mut value = caps[2].trim().to_string();
                // Expand macros in the value using what we know so far
                value = self.expand(&value);
                self.macros.insert(name, value);
            }
        }
    }

    pub fn set_epoch(&mut self, epoch: &str) {
        self.macros.insert("epoch".into(), epoch.into());
    }

    pub fn set_name_version(&mut self, name: &str, version: &str, release: &str) {
        self.macros.insert("name".into(), name.into());
        self.macros.insert("version".into(), version.into());
        self.macros.insert("release".into(), release.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.macros.get(key).map(|s| s.as_str())
    }

    pub fn is_defined(&self, key: &str) -> bool {
        self.macros.contains_key(key)
    }

    pub fn bcond_enabled(&self, name: &str) -> bool {
        self.bconds.get(name).copied().unwrap_or(false)
    }

    pub fn expand(&self, input: &str) -> String {
        let mut result = input.to_string();

        for _ in 0..10 {
            let prev = result.clone();
            result = self.expand_once(&result);
            if result == prev {
                break;
            }
        }

        result
    }

    fn expand_once(&self, input: &str) -> String {
        MACRO_RE
            .replace_all(input, |caps: &regex::Captures| {
                let inner = &caps[1];
                self.resolve_macro(inner)
            })
            .into_owned()
    }

    fn resolve_macro(&self, inner: &str) -> String {
        // %{with X} / %{without X}
        if let Some(name) = inner.strip_prefix("with ") {
            return if self.bcond_enabled(name.trim()) {
                "1"
            } else {
                "0"
            }
            .into();
        }
        if let Some(name) = inner.strip_prefix("without ") {
            return if self.bcond_enabled(name.trim()) {
                "0"
            } else {
                "1"
            }
            .into();
        }

        // %{?var:text} — expand text if var defined
        if let Some(rest) = inner.strip_prefix('?') {
            if let Some((var, text)) = rest.split_once(':') {
                return if self.is_effectively_defined(var) {
                    self.expand(text)
                } else {
                    String::new()
                };
            }
            // %{?var} — expand to value if defined, empty if not
            return if self.is_effectively_defined(rest) {
                self.macros
                    .get(rest)
                    .cloned()
                    .unwrap_or_else(String::new)
            } else {
                String::new()
            };
        }

        // %{!?var:text} — expand text if var NOT defined
        if let Some(rest) = inner.strip_prefix("!?") {
            if let Some((var, text)) = rest.split_once(':') {
                return if self.is_effectively_defined(var) {
                    String::new()
                } else {
                    self.expand(text)
                };
            }
            return if self.is_effectively_defined(rest) {
                String::new()
            } else {
                rest.to_string()
            };
        }

        // Plain %{var}
        if let Some(val) = self.macros.get(inner) {
            return val.clone();
        }

        // Unknown — keep as-is
        format!("%{{{inner}}}")
    }

    fn is_effectively_defined(&self, key: &str) -> bool {
        if self.macros.contains_key(key) {
            // Defined but empty counts as "defined" for %{?var}
            return true;
        }
        // SCL and other known-undefined macros on EL10
        matches!(
            key,
            "fedora"
                | "suse_version"
                | "scl_prefix"
                | "scl_prefix_ruby"
                | "scl_prefix_nodejs"
                | "scl"
                | "prereleasesource"
                | "prerelease"
        ) == false
            && false
    }
}

/// Evaluate a simple %if condition for EL10 context.
/// Returns Some(true/false) if evaluable, None if not.
pub fn evaluate_condition(expr: &str, expander: &MacroExpander) -> Option<bool> {
    let expanded = expander.expand(expr.trim());
    let expanded = expanded.trim();

    // %{with X} / %{without X} already expanded to 1/0
    // Handle: "1", "0"
    if expanded == "1" {
        return Some(true);
    }
    if expanded == "0" {
        return Some(false);
    }

    // Handle: 0%{?rhel} >= 8 -> "010" >= 8
    // After expansion, we may have things like "010 >= 8" or "0 >= 8"
    let parts: Vec<&str> = expanded.split_whitespace().collect();

    match parts.len() {
        1 => {
            // Single value — truthy if non-zero
            parts[0].parse::<i64>().ok().map(|v| v != 0)
        }
        3 => {
            // expr op expr
            let lhs = parts[0].parse::<i64>().ok()?;
            let rhs = parts[2].parse::<i64>().ok()?;
            let result = match parts[1] {
                ">=" => lhs >= rhs,
                "<=" => lhs <= rhs,
                ">" => lhs > rhs,
                "<" => lhs < rhs,
                "==" => lhs == rhs,
                "!=" => lhs != rhs,
                _ => return None,
            };
            Some(result)
        }
        _ => {
            // Handle || and &&
            if let Some(pos) = parts.iter().position(|&p| p == "||") {
                let left = parts[..pos].join(" ");
                let right = parts[pos + 1..].join(" ");
                let l = evaluate_condition(&left, expander)?;
                let r = evaluate_condition(&right, expander)?;
                return Some(l || r);
            }
            if let Some(pos) = parts.iter().position(|&p| p == "&&") {
                let left = parts[..pos].join(" ");
                let right = parts[pos + 1..].join(" ");
                let l = evaluate_condition(&left, expander)?;
                let r = evaluate_condition(&right, expander)?;
                return Some(l && r);
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_expansion() {
        let mut exp = MacroExpander::new();
        exp.scan_definitions("%global gem_name hammer_cli\n%global pypi_name Django");
        assert_eq!(exp.expand("%{gem_name}"), "hammer_cli");
        assert_eq!(exp.expand("%{pypi_name}"), "Django");
    }

    #[test]
    fn test_name_version() {
        let mut exp = MacroExpander::new();
        exp.set_name_version("rubygem-rails", "7.0.10", "1.el10");
        assert_eq!(exp.expand("%{name}"), "rubygem-rails");
        assert_eq!(exp.expand("%{version}"), "7.0.10");
    }

    #[test]
    fn test_conditional_defined() {
        let mut exp = MacroExpander::new();
        exp.scan_definitions("%global prerelease rc1");
        assert_eq!(exp.expand("%{?prerelease:0.}"), "0.");
        assert_eq!(exp.expand("%{?prerelease}"), "rc1");
    }

    #[test]
    fn test_conditional_undefined() {
        let exp = MacroExpander::new();
        assert_eq!(exp.expand("%{?prerelease:0.}"), "");
        assert_eq!(exp.expand("%{?prerelease}"), "");
    }

    #[test]
    fn test_negated_conditional() {
        let exp = MacroExpander::new();
        assert_eq!(
            exp.expand("%{!?scl:%{name}}"),
            "%{name}" // scl undefined, so expand the text
        );
    }

    #[test]
    fn test_scl_prefix_empty() {
        let exp = MacroExpander::new();
        assert_eq!(exp.expand("%{?scl_prefix}rubygem-rails"), "rubygem-rails");
        assert_eq!(
            exp.expand("%{?scl_prefix_ruby}rubygem-rails"),
            "rubygem-rails"
        );
    }

    #[test]
    fn test_rhel_defined() {
        let exp = MacroExpander::new();
        assert_eq!(exp.expand("%{?rhel}"), "10");
        assert_eq!(exp.expand("0%{?rhel}"), "010");
    }

    #[test]
    fn test_dist() {
        let exp = MacroExpander::new();
        assert_eq!(exp.expand("1%{?dist}"), "1.el10");
    }

    #[test]
    fn test_chained_macros() {
        let mut exp = MacroExpander::new();
        exp.scan_definitions(
            "%global python3_pkgversion 3.12\n%global pypi_name requests\n",
        );
        assert_eq!(
            exp.expand("python%{python3_pkgversion}-%{pypi_name}"),
            "python3.12-requests"
        );
    }

    #[test]
    fn test_bcond() {
        let mut exp = MacroExpander::new();
        exp.scan_definitions("%bcond bootstrap 0\n%bcond_without websockets");
        assert!(!exp.bcond_enabled("bootstrap"));
        assert!(exp.bcond_enabled("websockets"));
        assert_eq!(exp.expand("%{without bootstrap}"), "1");
        assert_eq!(exp.expand("%{with bootstrap}"), "0");
    }

    #[test]
    fn test_evaluate_rhel_condition() {
        let exp = MacroExpander::new();
        assert_eq!(evaluate_condition("0%{?rhel} >= 8", &exp), Some(true));
        assert_eq!(evaluate_condition("0%{?rhel} == 10", &exp), Some(true));
        assert_eq!(evaluate_condition("0%{?rhel} == 7", &exp), Some(false));
        assert_eq!(evaluate_condition("0%{?fedora}", &exp), Some(false));
        assert_eq!(evaluate_condition("0%{?suse_version}", &exp), Some(false));
    }

    #[test]
    fn test_evaluate_bcond_condition() {
        let mut exp = MacroExpander::new();
        exp.scan_definitions("%bcond bootstrap 0");
        assert_eq!(evaluate_condition("%{without bootstrap}", &exp), Some(true));
        assert_eq!(evaluate_condition("%{with bootstrap}", &exp), Some(false));
    }
}
