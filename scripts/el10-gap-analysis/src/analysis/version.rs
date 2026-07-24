use std::cmp::Ordering;

/// Compare two RPM version strings using the rpmvercmp algorithm.
pub fn rpmvercmp(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }

    let a_segs = split_segments(a);
    let b_segs = split_segments(b);

    let max_len = a_segs.len().max(b_segs.len());
    for i in 0..max_len {
        let sa = a_segs.get(i);
        let sb = b_segs.get(i);

        match (sa, sb) {
            (Some(a), Some(b)) => {
                let ord = compare_segments(a, b);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            // One side ran out of segments
            (Some(Segment::Alpha(s)), None) if s == "~" => return Ordering::Less,
            (None, Some(Segment::Alpha(s))) if s == "~" => return Ordering::Greater,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }

    Ordering::Equal
}

/// Compare two EVR (Epoch:Version-Release) tuples.
pub fn compare_evr(
    a_epoch: u32,
    a_ver: &str,
    a_rel: &str,
    b_epoch: u32,
    b_ver: &str,
    b_rel: &str,
) -> Ordering {
    match a_epoch.cmp(&b_epoch) {
        Ordering::Equal => {}
        ord => return ord,
    }

    match rpmvercmp(a_ver, b_ver) {
        Ordering::Equal => {}
        ord => return ord,
    }

    if !a_rel.is_empty() && !b_rel.is_empty() {
        rpmvercmp(a_rel, b_rel)
    } else {
        Ordering::Equal
    }
}

/// Check if `available` version satisfies `op required`.
pub fn version_satisfies(available: &str, op: &str, required: &str) -> bool {
    let ord = rpmvercmp(available, required);
    match op {
        ">=" | "GE" => ord != Ordering::Less,
        "<=" | "LE" => ord != Ordering::Greater,
        ">" | "GT" => ord == Ordering::Greater,
        "<" | "LT" => ord == Ordering::Less,
        "=" | "==" | "EQ" => ord == Ordering::Equal,
        _ => true,
    }
}

#[derive(Debug, PartialEq)]
enum Segment {
    Numeric(u64),
    Alpha(String),
}

fn split_segments(version: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut chars = version.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut num = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() {
                    num.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            // Strip leading zeros for numeric comparison
            let val = num.parse::<u64>().unwrap_or(0);
            segments.push(Segment::Numeric(val));
        } else if c.is_ascii_alphabetic() {
            let mut alpha = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphabetic() {
                    alpha.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            segments.push(Segment::Alpha(alpha));
        } else {
            // Skip separators (., -, _, ~, ^, etc.)
            // Handle ~ (sorts before everything, including empty)
            if c == '~' {
                segments.push(Segment::Alpha("~".into()));
            }
            chars.next();
        }
    }

    segments
}

fn compare_segments(a: &Segment, b: &Segment) -> Ordering {
    match (a, b) {
        // Tilde sorts before everything
        (Segment::Alpha(a), _) if a == "~" => {
            if matches!(b, Segment::Alpha(b) if b == "~") {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
        (_, Segment::Alpha(b)) if b == "~" => Ordering::Greater,

        (Segment::Numeric(a), Segment::Numeric(b)) => a.cmp(b),
        (Segment::Alpha(a), Segment::Alpha(b)) => a.cmp(b),
        // Numeric segments sort higher than alpha
        (Segment::Numeric(_), Segment::Alpha(_)) => Ordering::Greater,
        (Segment::Alpha(_), Segment::Numeric(_)) => Ordering::Less,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert_eq!(rpmvercmp("1.0", "1.0"), Ordering::Equal);
        assert_eq!(rpmvercmp("1.0.0", "1.0.0"), Ordering::Equal);
    }

    #[test]
    fn test_basic_ordering() {
        assert_eq!(rpmvercmp("1.0", "1.1"), Ordering::Less);
        assert_eq!(rpmvercmp("1.1", "1.0"), Ordering::Greater);
        assert_eq!(rpmvercmp("1.0.1", "1.0.2"), Ordering::Less);
        assert_eq!(rpmvercmp("2.0", "1.999"), Ordering::Greater);
    }

    #[test]
    fn test_numeric_vs_alpha() {
        // Numeric segments sort higher than alpha
        assert_eq!(rpmvercmp("1.0.1a", "1.0.2"), Ordering::Less);
        assert_eq!(rpmvercmp("1.0a", "1.0.1"), Ordering::Less);
    }

    #[test]
    fn test_different_lengths() {
        assert_eq!(rpmvercmp("1.0", "1.0.1"), Ordering::Less);
        assert_eq!(rpmvercmp("1.0.1", "1.0"), Ordering::Greater);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(rpmvercmp("1.01", "1.1"), Ordering::Equal);
        assert_eq!(rpmvercmp("1.001", "1.1"), Ordering::Equal);
    }

    #[test]
    fn test_tilde() {
        // Tilde sorts before everything
        assert_eq!(rpmvercmp("1.0~rc1", "1.0"), Ordering::Less);
        assert_eq!(rpmvercmp("1.0~rc1", "1.0~rc2"), Ordering::Less);
    }

    #[test]
    fn test_evr_epoch() {
        assert_eq!(
            compare_evr(1, "1.0", "1", 0, "99.0", "1"),
            Ordering::Greater
        );
        assert_eq!(
            compare_evr(0, "2.0", "1", 0, "1.0", "1"),
            Ordering::Greater
        );
    }

    #[test]
    fn test_version_satisfies() {
        assert!(version_satisfies("7.0.10", ">=", "7.0.3"));
        assert!(version_satisfies("7.0.10", "<", "7.1.0"));
        assert!(!version_satisfies("7.1.0", "<", "7.1.0"));
        assert!(version_satisfies("2.7", ">=", "2.7"));
        assert!(!version_satisfies("2.6", ">=", "2.7"));
        assert!(version_satisfies("5.0", "=", "5.0"));
    }

    #[test]
    fn test_realistic_versions() {
        assert_eq!(rpmvercmp("3.12.4", "3.12.3"), Ordering::Greater);
        assert_eq!(rpmvercmp("7.0.10", "7.0.3"), Ordering::Greater);
        assert_eq!(rpmvercmp("4.2.24", "4.2.0"), Ordering::Greater);
        assert_eq!(rpmvercmp("4.2.24", "5.0"), Ordering::Less);
    }
}
