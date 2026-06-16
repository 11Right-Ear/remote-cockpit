//! Audit logging.
//!
//! Per ADR 0001 / SECURITY.md, the gateway records security-relevant events
//! to SQLite. DB writes are blocking, so a single background task drains an
//! unbounded channel and writes rows off the async runtime.

use std::path::Path;

use anyhow::Context;
use tokio::sync::mpsc;

/// Audit event name constants.
pub mod event {
    pub const AUTH_OK: &str = "auth_ok";
    pub const AUTH_FAIL: &str = "auth_fail";
    pub const SESSION_OPEN: &str = "session_open";
    pub const SESSION_CLOSE: &str = "session_close";
    pub const DANGER_WARN: &str = "danger_warn";
    pub const DESKTOP_ONLINE: &str = "desktop_online";
    pub const DESKTOP_OFFLINE: &str = "desktop_offline";
    pub const DIR_LIST: &str = "dir_list";
}

#[derive(Debug)]
struct AuditRow {
    ts_ms: i64,
    actor: String,
    event: &'static str,
    session_id: Option<String>,
    detail: Option<String>,
}

/// Cloneable handle used to enqueue audit rows from async connection tasks.
#[derive(Clone)]
pub struct AuditHandle {
    tx: mpsc::UnboundedSender<AuditRow>,
}

impl AuditHandle {
    /// Enqueue an audit row. Never blocks; a dropped channel is silently ignored.
    pub fn record(
        &self,
        event: &'static str,
        actor: impl Into<String>,
        session_id: Option<String>,
        detail: Option<String>,
    ) {
        let _ = self.tx.send(AuditRow {
            ts_ms: chrono::Utc::now().timestamp_millis(),
            actor: actor.into(),
            event,
            session_id,
            detail,
        });
    }
}

/// Open (or create) the audit DB and spawn the background writer task.
pub fn spawn(db_path: &Path) -> anyhow::Result<AuditHandle> {
    let conn = open_db(db_path)?;
    let (tx, mut rx) = mpsc::unbounded_channel::<AuditRow>();
    tokio::task::spawn_blocking(move || {
        while let Some(row) = rx.blocking_recv() {
            if let Err(e) = write_row(&conn, &row) {
                tracing::error!(error = %e, event = row.event, "audit write failed");
            }
        }
        tracing::info!("audit writer stopped");
    });
    Ok(AuditHandle { tx })
}

fn open_db(path: &Path) -> anyhow::Result<rusqlite::Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let conn =
        rusqlite::Connection::open(path).with_context(|| format!("open {}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS audit_log (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            ts_ms       INTEGER NOT NULL,
            actor       TEXT NOT NULL,
            event       TEXT NOT NULL,
            session_id  TEXT,
            detail      TEXT
        );
        CREATE INDEX IF NOT EXISTS audit_log_ts_idx ON audit_log(ts_ms);",
    )?;
    Ok(conn)
}

fn write_row(conn: &rusqlite::Connection, row: &AuditRow) -> anyhow::Result<()> {
    conn.execute(
        "INSERT INTO audit_log (ts_ms, actor, event, session_id, detail)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            row.ts_ms,
            row.actor,
            row.event,
            row.session_id,
            row.detail,
        ],
    )?;
    Ok(())
}
