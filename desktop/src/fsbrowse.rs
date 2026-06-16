//! Phase 2 file browser: read-only directory listing + file reading, with a
//! path-jail guard.
//!
//! SECURITY (SECURITY.md / BACKEND.md): all access is read-only. Requests are
//! jailed to a root directory — env `RC_FS_ROOT` if set, otherwise the agent's
//! current working dir. A request that canonicalizes outside the root is
//! rejected. The gateway audits every `list_dir`/`read_file` by path.

use std::path::{Path, PathBuf};

use anyhow::Context;
use protocol::DirEntry;

/// Max bytes returned by `read_file`. Larger files are truncated.
const READ_LIMIT_BYTES: usize = 256 * 1024;

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

/// Strip the Windows `\\?\` verbatim prefix for display.
fn clean_display(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.strip_prefix(r"\\?\")
        .map(str::to_owned)
        .unwrap_or_else(|| s.into_owned())
}

/// Resolve + jail a request, returning the canonicalized absolute path.
fn resolve(requested: &str) -> anyhow::Result<PathBuf> {
    let root = root()?;
    let base = if Path::new(requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        root.join(requested)
    };
    let canon = base
        .canonicalize()
        .with_context(|| format!("canonicalize {}", base.display()))?;
    if !canon.starts_with(&root) {
        anyhow::bail!("path escapes fs root: {}", canon.display());
    }
    Ok(canon)
}

/// List a directory (read-only). Returns the clean display path plus entries.
pub fn list_dir(requested: &str) -> anyhow::Result<(String, Vec<DirEntry>)> {
    let canon = resolve(requested)?;
    let entries = read_entries(&canon)?;
    Ok((clean_display(&canon), entries))
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

/// Content read from a file (read-only). `truncated` is true if the file
/// exceeded [READ_LIMIT_BYTES].
pub struct FileContent {
    pub path: String,
    pub content: String,
    pub truncated: bool,
}

/// Read a (small, text) file's content. Errors if the path is outside the
/// jail, is a directory, or is not valid UTF-8.
pub fn read_file(requested: &str) -> anyhow::Result<FileContent> {
    let canon = resolve(requested)?;
    let meta =
        std::fs::metadata(&canon).with_context(|| format!("metadata {}", canon.display()))?;
    if meta.is_dir() {
        anyhow::bail!("is a directory");
    }
    let total = meta.len();
    let truncated = total > READ_LIMIT_BYTES as u64;

    use std::io::Read;
    let mut buf = Vec::new();
    std::fs::File::open(&canon)
        .with_context(|| format!("open {}", canon.display()))?
        .take(READ_LIMIT_BYTES as u64)
        .read_to_end(&mut buf)
        .with_context(|| format!("read {}", canon.display()))?;

    let content = match std::str::from_utf8(&buf) {
        Ok(s) => s.to_owned(),
        Err(_) => anyhow::bail!(
            "not a text file (invalid UTF-8{})",
            if truncated { " — or truncated mid-character" } else { "" }
        ),
    };

    Ok(FileContent {
        path: clean_display(&canon),
        content,
        truncated,
    })
}
