//! Shared helpers: hidden process runner, ssh, TCP probe, time, backups.
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Mutex;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::timeout;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Run a program with no console window. Returns (exit_code, stdout, stderr).
/// On timeout the child is killed (kill_on_drop) and an Err is returned.
pub async fn run_hidden(
    cmd: &str,
    args: &[&str],
    timeout_ms: u64,
) -> Result<(i32, String, String), String> {
    let mut c = Command::new(cmd);
    c.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);

    let child = c.spawn().map_err(|e| format!("cannot start {cmd}: {e}"))?;
    // Tracked until it ends, so Quit can stop it (see `kill_tracked`).
    let _guard = child.id().map(Tracked::new);
    match timeout(Duration::from_millis(timeout_ms), child.wait_with_output()).await {
        Ok(Ok(out)) => Ok((
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )),
        Ok(Err(e)) => Err(format!("{cmd} failed: {e}")),
        // The future (and the child it owns) is dropped here, which kills the process.
        Err(_) => Err(format!("{cmd} timed out after {timeout_ms} ms")),
    }
}

/// `ssh -o BatchMode=yes -o ConnectTimeout=4 <alias> "<cmd>"`, 8 s total timeout.
pub async fn ssh(alias: &str, cmd: &str) -> Result<(i32, String, String), String> {
    ssh_timeout(alias, cmd, 8000).await
}

/// Same as `ssh`, with a caller-chosen total timeout in milliseconds.
pub async fn ssh_timeout(alias: &str, cmd: &str, ms: u64) -> Result<(i32, String, String), String> {
    run_hidden(
        "ssh",
        &[
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=4",
            "--",
            alias,
            cmd,
        ],
        ms,
    )
    .await
}

/// `ssh` with `input` written to the remote command's stdin, then closed. For secrets: they never
/// appear on a command line (ours or the server's), so no process list or shell history sees them.
pub async fn ssh_stdin(
    alias: &str,
    cmd: &str,
    input: &str,
    ms: u64,
) -> Result<(i32, String, String), String> {
    let mut c = Command::new("ssh");
    c.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=4",
        "--",
        alias,
        cmd,
    ])
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .kill_on_drop(true);
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);

    let mut child = c.spawn().map_err(|e| format!("cannot start ssh: {e}"))?;
    let _guard = child.id().map(Tracked::new);
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(|e| format!("ssh stdin: {e}"))?;
        // dropped here: the remote side sees end of input
    }
    match timeout(Duration::from_millis(ms), child.wait_with_output()).await {
        Ok(Ok(out)) => Ok((
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )),
        Ok(Err(e)) => Err(format!("ssh failed: {e}")),
        Err(_) => Err(format!("ssh timed out after {ms} ms")),
    }
}

/// TCP connect to host:port within `ms` milliseconds.
pub async fn tcp_probe(host: &str, port: u16, ms: u64) -> Result<(), String> {
    match timeout(Duration::from_millis(ms), TcpStream::connect((host, port))).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(format!("{host}:{port} {e}")),
        Err(_) => Err(format!("{host}:{port} timed out after {ms} ms")),
    }
}

pub fn now_rfc3339() -> String {
    chrono::Local::now().to_rfc3339()
}

/// `<p>.bak-desk-monitor-<yyyymmddHHMMSS>`, with `-1`, `-2`... if that name exists.
pub fn backup_path(p: &Path) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let mut s = p.as_os_str().to_os_string();
    s.push(format!(".bak-desk-monitor-{stamp}"));
    let base = PathBuf::from(s);
    let mut cand = base.clone();
    let mut n = 1;
    while cand.exists() {
        let mut t = base.as_os_str().to_os_string();
        t.push(format!("-{n}"));
        cand = PathBuf::from(t);
        n += 1;
    }
    cand
}

/// PIDs of children started by `run_hidden` that are still running. Detached programs
/// (mstsc, the .ps1 windows, the Deskflow GUI) are started elsewhere and never land here.
static TRACKED: Mutex<Option<HashSet<u32>>> = Mutex::new(None);

/// Registers a PID on creation, removes it on drop (finished, failed or timed out).
struct Tracked(u32);

impl Tracked {
    fn new(pid: u32) -> Self {
        if let Ok(mut t) = TRACKED.lock() {
            t.get_or_insert_with(HashSet::new).insert(pid);
        }
        Tracked(pid)
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        if let Ok(mut t) = TRACKED.lock()
            && let Some(set) = t.as_mut()
        {
            set.remove(&self.0);
        }
    }
}

/// PIDs currently tracked (for tests and logging).
pub fn tracked_pids() -> Vec<u32> {
    TRACKED
        .lock()
        .ok()
        .and_then(|t| t.as_ref().map(|s| s.iter().copied().collect()))
        .unwrap_or_default()
}

/// Force-stop every tracked child (ssh, qwinsta, tasklist...). Called on Quit, because
/// `app.exit` ends the process without running the async drops that would kill them.
/// No `/T`: only the direct child is stopped, never anything it handed off to.
pub fn kill_tracked() {
    for pid in tracked_pids() {
        let mut c = std::process::Command::new("taskkill");
        c.args(["/PID", &pid.to_string(), "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(CREATE_NO_WINDOW);
        }
        let _ = c.status();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_registers_and_clears() {
        {
            let _g = Tracked::new(4_000_000_001);
            assert!(tracked_pids().contains(&4_000_000_001));
        }
        assert!(!tracked_pids().contains(&4_000_000_001));
    }
}
