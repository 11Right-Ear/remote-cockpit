//! PTY lifecycle via `portable-pty` (Win11 ConPTY / Unix forkpty).
//!
//! V1 holds a single session at a time. The reader (PTY output) is handed to
//! a blocking task by the caller; writes and resize happen here.

use std::io::{Read, Write};

use anyhow::Context;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// An owned PTY session: master, child, and a writer handle.
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn Write + Send>,
}

impl PtySession {
    /// Spawn `shell` in a new PTY at the given size. Returns the session plus
    /// a reader for the PTY's output stream (to be drained in a blocking task).
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> anyhow::Result<(Self, Box<dyn Read + Send>)> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("openpty")?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(".");

        // The slave must be dropped after spawning so the PTY behaves correctly
        // (EOF propagation, no extra open handles).
        let child = pair.slave.spawn_command(cmd).context("spawn shell")?;
        drop(pair.slave);

        let writer = pair.master.take_writer().context("take pty writer")?;
        let reader = pair.master.try_clone_reader().context("take pty reader")?;
        let master = pair.master;

        Ok((Self { master, child, writer }, reader))
    }

    /// Write input bytes to the PTY (keystrokes / pasted commands).
    pub fn write(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("pty resize")
    }

    /// Kill the child process and wait for it to exit.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
