//! Phase 2 file browser: read-only directory listing with a path-jail guard.
//!
//! SECURITY (SECURITY.md / BACKEND.md): listing is read-only. Requests are
//! jailed to a root directory — env `RC_FS_ROOT` if set, otherwise the agent's
//! current working dir. A request that canonicalizes outside the root is
//! rejected. The gateway audits every `list_dir` by path regardless of outcome.

use std::path::{Path, PathBuf};

use anyhow::Context;
use protocol::DirEntry;

/// Resolve the jail root: `RC_FS_ROOT` if set (canonicalized), else the cwd.
fn root() -> anyhow::Result<PathBuf> {
    if let Ok(raw) = std::env::var("RC_FS_ROOT") {
        let p = PathBuf::from(raw);
        return Ok(p.canonicalize().unwrap_or(p));
    }
    // Canonicalize so the jail check matches canonicalized request paths
    // (Windows canonicalize yields a `\\?\`-prefixed verbatim path; the cwd
    // does not, so a naive starts_with would always mismatch).
    std::env::current_dir()
        .context("get cwd for fs root")?
        .canonicalize()
        .context("canonicalize cwd for fs root")
}

/// List a directory (read-only). `requested` may be absolute or relative to
/// the jail root. Returns the canonicalized path actually listed plus its
/// entries. Errors if the path is outside the root or unreadable.
pub fn list_dir(requested: &str) -> anyhow::Result<(PathBuf, Vec<DirEntry>)> {
    let root = root()?;
    let base = if Path::new(requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        root.join(requested)
    };
    let canon = base
        .canonicalize()
        .with_context(|| format!("canonicalize {}", base.display()))?;
    // Jail: reject anything that escapes the root after canonicalization.
    if !canon.starts_with(&root) {
        anyhow::bail!("path escapes fs root: {}", canon.display());
    }
    let entries = read_entries(&canon)?;
    Ok((canon, entries))
}

fn read_entries(dir: &Path) -> anyhow::Result<Vec<DirEntry>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let meta = entry.metadata()?;
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        out.push(DirEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: meta.is_dir(),
            size: meta.len(),
            modified_ms,
        });
    }
    // Folders first, then files; alphabetical within each group.
    out.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
    Ok(out)
}
