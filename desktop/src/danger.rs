//! Dangerous-command detection.
//!
//! Per SECURITY.md, V1 only *warns* — execution still proceeds. Detection runs
//! on the Desktop Agent (which sees the raw PTY input bytes), not the gateway
//! (which only relays). Matches are reported via `ReportDanger`.

use regex::bytes::Regex;

/// A positive danger detection.
pub struct DangerHit {
    /// The regex pattern that matched.
    pub pattern: String,
    /// The (lossy) command text that matched.
    pub command: String,
}

/// Checks terminal input bytes against a small set of high-risk patterns.
pub struct DangerChecker {
    patterns: Vec<(String, Regex)>,
}

impl DangerChecker {
    pub fn new() -> Self {
        // Intentionally conservative: a handful of clearly destructive
        // commands. False negatives are acceptable in V1 (warn-only); false
        // positives would annoy. Tunable later.
        let raw: &[&str] = &[
            r"rm\s+(-[a-zA-Z]*r[a-zA-Z]*f|--recursive\b.*--force\b)",
            r"\bshutdown\b",
            r"\breboot\b",
            r"\bmkfs\b",
            r"dd\s+.*of=/dev/",
        ];
        let patterns = raw
            .iter()
            .filter_map(|p| Regex::new(p).ok().map(|re| ((*p).to_string(), re)))
            .collect();
        Self { patterns }
    }

    /// Returns the first matching pattern, if any.
    pub fn check(&self, input: &[u8]) -> Option<DangerHit> {
        for (pat, re) in &self.patterns {
            if re.is_match(input) {
                return Some(DangerHit {
                    pattern: pat.clone(),
                    command: String::from_utf8_lossy(input).into_owned(),
                });
            }
        }
        None
    }
}

impl Default for DangerChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rm_rf() {
        let c = DangerChecker::new();
        assert!(c.check(b"rm -rf /tmp/x").is_some());
        assert!(c.check(b"ls -la").is_none());
    }

    #[test]
    fn detects_shutdown_reboot() {
        let c = DangerChecker::new();
        assert!(c.check(b"sudo shutdown now").is_some());
        assert!(c.check(b"reboot").is_some());
    }
}
