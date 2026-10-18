//! Deskflow: client status, fix actions, server.conf layout parse/write.
//! Nothing here runs on its own: every action is a direct result of a button click.
use crate::config::{self, Config, Hotkey, Machine};
use crate::util;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

/// connected | hung | disconnected | asleep | offline | unknown
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientState {
    Connected,
    Hung,
    Disconnected,
    Asleep,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClientStatus {
    pub id: String,
    pub state: ClientState,
    pub reason: String,
    pub checked_at: String,
}

const SERVER_LOG: &str = r"C:\ProgramData\Deskflow\deskflow-core.log";
const LOG_TAIL_BYTES: u64 = 2 * 1024 * 1024;
/// Recv-Q above this on the ESTAB :24800 socket = the clipboard-chunk hang.
const HUNG_RECV_Q: u64 = 100_000;
const DESKFLOW_EXE_DEFAULT: &str = r"C:\Program Files\Deskflow\deskflow.exe";
/// Where Deskflow keeps a client's settings: the Flatpak build on Linux, the app on macOS.
const LINUX_CLIENT_CONF: &str =
    "$HOME/.var/app/org.deskflow.deskflow/config/Deskflow/Deskflow.conf";
const MAC_CLIENT_CONF: &str = "$HOME/Library/Deskflow/Deskflow.conf";

fn machine_by_id(id: &str) -> Result<Machine, String> {
    let cfg = config::load()?;
    config::find(&cfg, id)
}

fn status(id: &str, state: ClientState, reason: impl Into<String>) -> ClientStatus {
    ClientStatus {
        id: id.to_string(),
        state,
        reason: reason.into(),
        checked_at: util::now_rfc3339(),
    }
}

fn mb(bytes: u64) -> String {
    format!("{:.2} MB", bytes as f64 / 1_000_000.0)
}

// ---------------------------------------------------------------- status

#[tauri::command]
pub async fn deskflow_status(id: String) -> Result<ClientStatus, String> {
    let m = machine_by_id(&id)?;
    match m.role.as_str() {
        "deskflow-server" => Ok(server_status(&id).await),
        "deskflow-client" => Ok(client_status(&m).await),
        other => Err(format!(
            "{} is not a Deskflow machine (role {other})",
            m.name
        )),
    }
}

async fn server_status(id: &str) -> ClientStatus {
    match util::run_hidden("tasklist", &["/FO", "CSV", "/NH"], 5000).await {
        Ok((_, out, _)) => {
            // The Deskflow window does not matter any more: the service runs the core.
            if parse_tasklist(&out).1 {
                status(
                    id,
                    ClientState::Connected,
                    format!("server running{}", version_note().await),
                )
            } else {
                status(
                    id,
                    ClientState::Disconnected,
                    "Deskflow server not running - press Restart Deskflow server",
                )
            }
        }
        Err(e) => status(
            id,
            ClientState::Unknown,
            format!("can't read process list: {e}"),
        ),
    }
}

/// The Deskflow version the service pipe protocol was read from (daemon.rs). The protocol has no
/// version check, so a different Deskflow gets a visible warning instead of silent breakage.
const TESTED_DESKFLOW: &str = "1.26.";

static DESKFLOW_VERSION: tokio::sync::OnceCell<Option<String>> = tokio::sync::OnceCell::const_new();

/// "" when the installed Deskflow is the tested one (or unreadable), else a short warning.
async fn version_note() -> String {
    let v = DESKFLOW_VERSION
        .get_or_init(|| async {
            let exe = deskflow_exe().await;
            let core = exe.with_file_name("deskflow-core.exe");
            let arg = format!("(Get-Item '{}').VersionInfo.ProductVersion", core.display());
            match util::run_hidden("powershell", &["-NoProfile", "-Command", &arg], 8000).await {
                Ok((0, out, _)) => Some(out.trim().to_string()).filter(|v| !v.is_empty()),
                _ => None,
            }
        })
        .await;
    match v {
        Some(v) if !v.starts_with(TESTED_DESKFLOW) => format!(
            " - Deskflow {v} is untested (built for {TESTED_DESKFLOW}x); restart may not work"
        ),
        _ => String::new(),
    }
}

/// PIDs of deskflow-core.exe from `tasklist /FO CSV /NH` ("name","pid",...).
fn core_pids(out: &str) -> Vec<u32> {
    out.lines()
        .filter_map(|l| {
            let mut c = l.split(',').map(|f| f.trim().trim_matches('"'));
            let name = c.next()?;
            let pid = c.next()?.parse().ok()?;
            name.eq_ignore_ascii_case("deskflow-core.exe")
                .then_some(pid)
        })
        .collect()
}

/// True when every core now running started after the restart (the old ones are gone).
fn is_new_core(before: &[u32], now: &[u32]) -> bool {
    !now.is_empty() && now.iter().all(|p| !before.contains(p))
}

/// (deskflow.exe running, deskflow-core.exe running) from `tasklist /FO CSV /NH`.
fn parse_tasklist(out: &str) -> (bool, bool) {
    let mut gui = false;
    let mut core = false;
    for line in out.lines() {
        let name = line
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .trim_matches('"')
            .to_ascii_lowercase();
        match name.as_str() {
            "deskflow.exe" => gui = true,
            "deskflow-core.exe" => core = true,
            _ => {}
        }
    }
    (gui, core)
}

async fn client_status(m: &Machine) -> ClientStatus {
    // One deadline for the whole check, so it always ends before the next 15 s refresh.
    match tokio::time::timeout(std::time::Duration::from_secs(10), client_status_inner(m)).await {
        Ok(s) => s,
        Err(_) => status(
            &m.id,
            ClientState::Unknown,
            "status check timed out after 10 s",
        ),
    }
}

async fn client_status_inner(m: &Machine) -> ClientStatus {
    let id = m.id.as_str();
    if util::tcp_probe(&m.host, 22, 1500).await.is_err() {
        return status(
            id,
            ClientState::Offline,
            format!("no answer from {} on port 22", m.host),
        );
    }

    let mut note = String::new();
    let mut ssh_failed = false;
    if let Some(alias) = m.ssh_alias.as_deref() {
        match recv_q(alias, &m.os).await {
            Ok(Some(q)) if q > HUNG_RECV_Q => {
                return status(
                    id,
                    ClientState::Hung,
                    format!("stuck: {} waiting on port 24800", mb(q)),
                );
            }
            Ok(_) => {}
            Err(e) => {
                ssh_failed = true;
                note = format!(" (hang check failed: {e})");
            }
        }
    }

    let Some(name) = m.deskflow_name.as_deref() else {
        return status(
            id,
            ClientState::Unknown,
            format!("no deskflow name set{note}"),
        );
    };
    let last = match read_log_tail() {
        Ok(text) => last_client_event(&text, name),
        Err(e) => {
            return status(
                id,
                ClientState::Unknown,
                format!("can't read server log: {e}{note}"),
            );
        }
    };
    match last {
        Some(true) => status(
            id,
            ClientState::Connected,
            format!("\"{name}\" connected{note}"),
        ),
        Some(false) => {
            if m.os == "mac"
                && !ssh_failed
                && let Some(alias) = m.ssh_alias.as_deref()
                && let Ok(0) = caffeinate_count(alias).await
            {
                return status(
                    id,
                    ClientState::Asleep,
                    format!(
                        "disconnected, 0 caffeinate assertions: likely asleep (unverified){note}"
                    ),
                );
            }
            if m.os == "linux"
                && !ssh_failed
                && let Some(alias) = m.ssh_alias.as_deref()
                && let Ok(false) = linux_seat_logged_in(alias).await
            {
                let hint = if config::opt(&m.login_command).is_some() {
                    ": use Log in and type your password"
                } else {
                    ""
                };
                return status(
                    id,
                    ClientState::Disconnected,
                    format!("at the login screen{hint}{note}"),
                );
            }
            status(
                id,
                ClientState::Disconnected,
                format!("\"{name}\" disconnected (server log){note}"),
            )
        }
        None => status(
            id,
            ClientState::Unknown,
            format!("no connect line for \"{name}\" in last 2 MB of log{note}"),
        ),
    }
}

/// True when a user (not the GDM greeter) is logged in on the machine's own screen (seat0).
/// The Deskflow client only runs inside that session.
async fn linux_seat_logged_in(alias: &str) -> Result<bool, String> {
    let (code, out, err) = util::ssh(alias, "loginctl list-sessions --no-legend").await?;
    if code != 0 {
        return Err(format!("loginctl failed: {}", err.trim()));
    }
    Ok(parse_seat_user(&out))
}

/// `loginctl list-sessions --no-legend` columns: SESSION UID USER SEAT LEADER CLASS ...
fn parse_seat_user(out: &str) -> bool {
    out.lines().any(|l| {
        let c: Vec<&str> = l.split_whitespace().collect();
        c.get(3) == Some(&"seat0") && c.get(5) == Some(&"user")
    })
}

/// Max Recv-Q on an established :24800 socket, or None if there is none.
async fn recv_q(alias: &str, os: &str) -> Result<Option<u64>, String> {
    let cmd = if os == "mac" {
        "netstat -an | grep 24800"
    } else {
        "ss -tn | grep 24800"
    };
    let (code, out, err) = util::ssh(alias, cmd).await?;
    // grep exits 1 when nothing matches: that is "no socket", not an error.
    if code != 0 && code != 1 {
        let msg = err.trim();
        return Err(if msg.is_empty() {
            format!("ssh exit {code}")
        } else {
            msg.to_string()
        });
    }
    Ok(parse_recv_q(&out))
}

/// Works for Linux `ss -tn` (state first, Recv-Q col 2) and BSD `netstat -an`
/// (proto first, Recv-Q col 2, state last). Only ESTAB/ESTABLISHED lines count.
fn parse_recv_q(out: &str) -> Option<u64> {
    out.lines()
        .filter(|l| {
            l.split_whitespace()
                .any(|w| w == "ESTAB" || w == "ESTABLISHED")
        })
        .filter_map(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
        .max()
}

async fn caffeinate_count(alias: &str) -> Result<u64, String> {
    let (_, out, _) = util::ssh(alias, "pmset -g assertions | grep -c caffeinate").await?;
    out.trim()
        .parse::<u64>()
        .map_err(|_| format!("unexpected output: {}", out.trim()))
}

fn read_log_tail() -> Result<String, String> {
    let mut f = std::fs::File::open(SERVER_LOG).map_err(|e| e.to_string())?;
    let len = f.metadata().map_err(|e| e.to_string())?.len();
    let start = len.saturating_sub(LOG_TAIL_BYTES);
    f.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Some(true) = last event was "has connected", Some(false) = "has disconnected".
fn last_client_event(log: &str, name: &str) -> Option<bool> {
    let needle = format!("client \"{name}\" has ");
    log.lines().rev().find_map(|l| {
        let i = l.find(&needle)?;
        let rest = &l[i + needle.len()..];
        if rest.starts_with("connected") {
            Some(true)
        } else if rest.starts_with("disconnected") {
            Some(false)
        } else {
            None
        }
    })
}

// ---------------------------------------------------------------- actions

/// action: restart_client | wake | set_remote_host | restart_server (log in: `login_server`)
#[tauri::command]
pub async fn deskflow_action(id: String, action: String) -> Result<String, String> {
    match action.as_str() {
        "restart_client" => restart_client(&machine_by_id(&id)?).await,
        "wake" => run_wake_script(&machine_by_id(&id)?),
        "restart_server" => restart_server().await,
        "set_remote_host" => set_remote_host(&machine_by_id(&id)?).await,
        other => Err(format!("unknown action: {other}")),
    }
}

/// Log a user in on a client's own screen, or unlock it, by running the machine's
/// `login_command` over ssh. The password goes to it as line 1 of stdin and is checked there;
/// it is never stored. Contract: exit 0 = logged in, 3 = wrong password, 4 = already running.
/// Why not the mouse: a Wayland login screen (e.g. GNOME) cannot be driven by Deskflow.
#[tauri::command]
pub async fn login_server(id: String, password: String) -> Result<String, String> {
    if password.is_empty() {
        return Err("Type your password first.".into());
    }
    let m = machine_by_id(&id)?;
    let cmd = config::opt(&m.login_command)
        .ok_or_else(|| format!("{} has no log in command set", m.name))?;
    let alias = need_alias(&m)?;
    // A log in can take a while (a display manager restart, then waiting for the session).
    let (code, out, err) = util::ssh_stdin(alias, cmd, &format!("{password}\n"), 60000)
    .await
    // The server keeps going after we stop waiting, so "failed" would be a guess.
    .map_err(|e| {
        if e.contains("timed out") {
            format!(
                "No answer from {} after 60 s - it may still be logging in. Look at its screen.",
                m.name
            )
        } else {
            e
        }
    })?;
    if code == 4 {
        return Err(format!(
            "A log in on {} is already running. Wait a minute and look at its screen.",
            m.name
        ));
    }
    let text = out.trim();
    match code {
        0 => {
            let said = if text.is_empty() {
                format!("logged in on {}", m.name)
            } else {
                text.to_string()
            };
            Ok(format!("{said} {}", wait_for_link(&m).await))
        }
        3 => Err("Wrong password. Nothing changed.".into()),
        _ => Err(format!(
            "log in on {} failed (exit {code}): {}",
            m.name,
            if text.is_empty() { err.trim() } else { text }
        )),
    }
}

/// After a login the client starts with the desktop, a few seconds after the session appears
/// (measured 2026-10-18: session 19:57:45, connected 19:57:50). The status refresh that follows
/// Log in would otherwise still read the old "disconnected" line.
async fn wait_for_link(m: &Machine) -> String {
    let Some(name) = m.deskflow_name.as_deref() else {
        return String::new();
    };
    for _ in 0..20 {
        if let Ok(log) = read_log_tail()
            && last_client_event(&log, name) == Some(true)
        {
            return "Mouse link connected.".into();
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    "Mouse link not up after 20 s - press refresh in a moment.".into()
}

fn need_alias(m: &Machine) -> Result<&str, String> {
    m.ssh_alias
        .as_deref()
        .filter(|a| !a.is_empty())
        .ok_or_else(|| format!("{} has no ssh alias", m.name))
}

/// Runs the machine's `client_restart_command` over ssh. Deskflow has no standard way to restart
/// a client remotely (it depends on how the client is started there), so the user sets it.
async fn restart_client(m: &Machine) -> Result<String, String> {
    let cmd = config::opt(&m.client_restart_command)
        .ok_or_else(|| format!("{} has no client restart command set", m.name))?;
    let alias = need_alias(m)?;
    // A Linux client that runs in the user's desktop session cannot start at the login screen,
    // and a restart command may exit 0 there without starting anything. When the machine has
    // a log in command, say to use that instead of reporting a restart that did not happen.
    if m.os == "linux"
        && config::opt(&m.login_command).is_some()
        && !linux_seat_logged_in(alias).await?
    {
        return Err(format!(
            "{} is at the login screen: use Log in, the client then starts by itself",
            m.name
        ));
    }
    // 25 s: launchd throttles a KeepAlive agent ~10 s after a restart, and kickstart waits for it.
    let (code, out, err) = util::ssh_timeout(alias, cmd, 25000).await?;
    if code == 0 {
        Ok(format!("restarted Deskflow client on {}", m.name))
    } else {
        Err(format!(
            "restart on {} failed (exit {code}): {}",
            m.name,
            first_text(&err, &out)
        ))
    }
}

fn first_text<'a>(a: &'a str, b: &'a str) -> &'a str {
    let a = a.trim();
    if a.is_empty() { b.trim() } else { a }
}

/// Starts the machine's `wake_script` (a local .ps1) in its own PowerShell window.
fn run_wake_script(m: &Machine) -> Result<String, String> {
    let script =
        config::opt(&m.wake_script).ok_or_else(|| format!("{} has no wake script set", m.name))?;
    let path = PathBuf::from(script);
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("ps1"))
    {
        return Err(format!(
            "wake script must be a .ps1 file: {}",
            path.display()
        ));
    }
    if !path.exists() {
        return Err(format!("script not found: {}", path.display()));
    }
    std::process::Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-File"])
        .arg(&path)
        .spawn()
        .map_err(|e| format!("can't start {}: {e}", path.display()))?;
    Ok(format!("started {} (see its window)", path.display()))
}

async fn deskflow_exe() -> PathBuf {
    if let Ok((0, out, _)) = util::run_hidden("where.exe", &["deskflow"], 4000).await
        && let Some(p) = out
            .lines()
            .map(str::trim)
            .find(|l| l.to_ascii_lowercase().ends_with(".exe"))
    {
        return PathBuf::from(p);
    }
    PathBuf::from(DESKFLOW_EXE_DEFAULT)
}

/// Restart the Deskflow server core through the Deskflow service, with no Deskflow window: the
/// window was only ever a remote control for the service, so desk-monitor is the one app.
async fn restart_server() -> Result<String, String> {
    let exe = deskflow_exe().await;
    let dir = exe
        .parent()
        .ok_or_else(|| format!("odd Deskflow path {}", exe.display()))?;
    if !dir.join("deskflow-core.exe").exists() {
        return Err(format!("Deskflow not found in {}", dir.display()));
    }
    let settings = deskflow_settings()?;
    let cmd = crate::daemon::core_command(&dir.to_string_lossy(), &settings.to_string_lossy())?;
    // Close the Deskflow window if it is open; the core belongs to the service and keeps running.
    let _ = util::run_hidden("taskkill", &["/IM", "deskflow.exe", "/F"], 6000).await;
    let before = match util::run_hidden("tasklist", &["/FO", "CSV", "/NH"], 5000).await {
        Ok((_, out, _)) => core_pids(&out),
        Err(_) => Vec::new(),
    };
    crate::daemon::start_core(&cmd).await?;
    // `start` answers before the core is up, and the old core lingers for a moment: only a core
    // that was not there before counts (measured 2026-10-18: old pid still listed after 1.5 s).
    for _ in 0..20 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok((_, out, _)) = util::run_hidden("tasklist", &["/FO", "CSV", "/NH"], 5000).await
            && is_new_core(&before, &core_pids(&out))
        {
            return Ok("restarted the Deskflow server (no Deskflow window needed)".into());
        }
    }
    Err("asked the Deskflow service to start the server, but deskflow-core is not running after 10 s".into())
}

/// The core's settings file - the one the Deskflow window kept (name, TLS, layout file path).
fn deskflow_settings() -> Result<PathBuf, String> {
    let appdata = std::env::var("APPDATA").map_err(|_| "APPDATA is not set".to_string())?;
    let p = PathBuf::from(appdata)
        .join("Deskflow")
        .join("Deskflow.conf");
    if p.exists() {
        Ok(p)
    } else {
        Err(format!("Deskflow settings not found at {}", p.display()))
    }
}

/// At app start: if no server core is running, ask the service for one (it normally keeps it
/// running by itself, even across reboots). Never opens a window.
pub async fn ensure_server_running() {
    if let Ok((_, out, _)) = util::run_hidden("tasklist", &["/FO", "CSV", "/NH"], 5000).await
        && !parse_tasklist(&out).1
    {
        let _ = restart_server().await;
    }
}

/// This PC's private LAN IPv4, routed toward `toward` when possible.
async fn lan_ipv4(toward: &str) -> Result<String, String> {
    if let Some(ip) = udp_local_ip(toward)
        && ip
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|a| a.is_private())
    {
        return Ok(ip);
    }
    let (_, out, _) = util::run_hidden("ipconfig", &[], 5000).await?;
    parse_ipconfig_lan(&out).ok_or_else(|| "no private LAN IPv4 address found on this PC".into())
}

/// Lower = more likely the home/office LAN: 192.168/16, then 10/8, then 172.16/12 (which is
/// also where WSL and Hyper-V put their virtual switches). Not private = None.
fn lan_rank(ip: std::net::Ipv4Addr) -> Option<u8> {
    match ip.octets() {
        [192, 168, ..] => Some(0),
        [10, ..] => Some(1),
        _ if ip.is_private() => Some(2),
        _ => None,
    }
}

fn udp_local_ip(toward: &str) -> Option<String> {
    use std::net::{ToSocketAddrs, UdpSocket};
    let target = (toward, 24800)
        .to_socket_addrs()
        .ok()?
        .find(|a| a.is_ipv4())?;
    let s = UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect(target).ok()?; // no packet is sent; this only picks a route
    Some(s.local_addr().ok()?.ip().to_string())
}

fn parse_ipconfig_lan(out: &str) -> Option<String> {
    out.lines()
        .filter(|l| l.contains("IPv4"))
        .filter_map(|l| l.rsplit(':').next())
        .filter_map(|v| {
            v.trim()
                .trim_end_matches("(Preferred)")
                .trim()
                .parse::<std::net::Ipv4Addr>()
                .ok()
        })
        .filter_map(|a| lan_rank(a).map(|r| (r, a)))
        .min_by_key(|(r, _)| *r)
        .map(|(_, a)| a.to_string())
}

async fn set_remote_host(m: &Machine) -> Result<String, String> {
    let alias = need_alias(m)?;
    let file = match m.os.as_str() {
        "linux" => LINUX_CLIENT_CONF,
        "mac" => MAC_CLIENT_CONF,
        other => return Err(format!("set_remote_host not supported for os {other}")),
    };
    let ip = lan_ipv4(&m.host).await?;
    // ip is a validated dotted IPv4, safe to put in the remote command.
    let ip: std::net::Ipv4Addr = ip.parse().map_err(|_| format!("bad IP: {ip}"))?;
    let stamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let bak = format!("{file}.bak-desk-monitor-{stamp}");
    // Portable across GNU and BSD sed: back up, then rewrite from the backup.
    let cmd = format!(
        "F=\"{file}\"; B=\"{bak}\"; [ -e \"$B\" ] && B=\"$B-1\"; [ -e \"$B\" ] && B=\"$B-$$\"; grep -q 'remoteHost=' \"$F\" || {{ echo 'no remoteHost= line in '\"$F\" >&2; exit 3; }}; \
         cp \"$F\" \"$B\" && sed 's/^\\(.*remoteHost=\\).*/\\1{ip}/' \"$B\" > \"$F\" && grep 'remoteHost=' \"$F\""
    );
    let (code, out, err) = util::ssh(alias, &cmd).await?;
    if code != 0 {
        return Err(format!(
            "can't set remoteHost on {} (exit {code}): {}",
            m.name,
            first_text(&err, &out)
        ));
    }
    let restart = if config::opt(&m.client_restart_command).is_none() {
        "restart its Deskflow client to use it".to_string()
    } else {
        match restart_client(m).await {
            Ok(s) => s,
            Err(e) => format!("but restart failed: {e}"),
        }
    };
    Ok(format!(
        "remoteHost on {} set to {ip} (backup {bak}); {restart}",
        m.name
    ))
}

// ---------------------------------------------------------------- layout (server.conf)

fn server_conf_path() -> Result<PathBuf, String> {
    let dir = dirs::config_dir().ok_or("can't find %APPDATA%")?;
    Ok(dir.join("Deskflow").join("server.conf"))
}

/// Deskflow screen names, left -> right.
#[tauri::command]
pub async fn layout_get() -> Result<Vec<String>, String> {
    let p = server_conf_path()?;
    let text =
        std::fs::read_to_string(&p).map_err(|e| format!("can't read {}: {e}", p.display()))?;
    Ok(parse_order(&text))
}

#[tauri::command]
pub async fn layout_save(order: Vec<String>) -> Result<String, String> {
    validate_order(&order)?;
    let p = server_conf_path()?;
    let old = match std::fs::read_to_string(&p) {
        Ok(t) => Some(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(format!("can't read {}: {e}", p.display())),
    };
    let mut msg = String::new();
    if old.is_some() {
        let bak = util::backup_path(&p);
        std::fs::copy(&p, &bak).map_err(|e| format!("backup failed, nothing changed: {e}"))?;
        msg.push_str(&format!("backup {}; ", bak.display()));
    } else if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let new = build_conf(&order, old.as_deref().unwrap_or(""));
    std::fs::write(&p, new).map_err(|e| format!("can't write {}: {e}", p.display()))?;
    msg.insert_str(0, &format!("saved layout {}; ", order.join(" | ")));
    match restart_server().await {
        Ok(s) => msg.push_str(&s),
        Err(e) => msg.push_str(&format!("restart failed: {e}")),
    }
    Ok(msg)
}

fn validate_order(order: &[String]) -> Result<(), String> {
    if order.is_empty() {
        return Err("layout is empty".into());
    }
    let mut seen = HashSet::new();
    for n in order {
        if n.is_empty()
            || n.chars()
                .any(|c| c.is_whitespace() || c == ':' || c == '=' || c == '#')
        {
            return Err(format!("bad screen name: {n:?}"));
        }
        if !seen.insert(n.to_ascii_lowercase()) {
            return Err(format!("screen listed twice: {n}"));
        }
    }
    Ok(())
}

enum Chunk {
    /// A whole `section: <name>` .. `end` block, raw lines.
    Section(String, Vec<String>),
    Line(String),
}

fn split_chunks(text: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut cur: Option<(String, Vec<String>)> = None;
    for line in text.lines() {
        let t = line.trim();
        if let Some((name, lines)) = cur.as_mut() {
            lines.push(line.to_string());
            if t == "end" {
                chunks.push(Chunk::Section(std::mem::take(name), std::mem::take(lines)));
                cur = None;
            }
        } else if let Some(rest) = t.strip_prefix("section:") {
            cur = Some((rest.trim().to_ascii_lowercase(), vec![line.to_string()]));
        } else {
            chunks.push(Chunk::Line(line.to_string()));
        }
    }
    if let Some((name, lines)) = cur {
        chunks.push(Chunk::Section(name, lines));
    }
    chunks
}

fn body(lines: &[String]) -> impl Iterator<Item = &str> {
    // skip "section:" header and "end"
    lines.iter().skip(1).map(|s| s.as_str()).filter(|l| {
        let t = l.trim();
        t != "end" && !t.is_empty() && !t.starts_with('#')
    })
}

fn is_screen_header(line: &str) -> Option<&str> {
    let t = line.trim();
    if t.ends_with(':') && !t.contains('=') {
        Some(t.trim_end_matches(':').trim())
    } else {
        None
    }
}

/// Screens in file order, each with its own option lines (raw).
fn parse_screens(chunks: &[Chunk]) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for c in chunks {
        if let Chunk::Section(name, lines) = c {
            if name != "screens" {
                continue;
            }
            for l in body(lines) {
                if let Some(s) = is_screen_header(l) {
                    out.push((s.to_string(), Vec::new()));
                } else if let Some(last) = out.last_mut() {
                    last.1.push(l.to_string());
                }
            }
        }
    }
    out
}

/// (screen, left, right) from the links section.
fn parse_links(chunks: &[Chunk]) -> Vec<(String, Option<String>, Option<String>)> {
    let mut out: Vec<(String, Option<String>, Option<String>)> = Vec::new();
    for c in chunks {
        if let Chunk::Section(name, lines) = c {
            if name != "links" {
                continue;
            }
            for l in body(lines) {
                if let Some(s) = is_screen_header(l) {
                    out.push((s.to_string(), None, None));
                } else if let (Some(last), Some((k, v))) = (out.last_mut(), l.split_once('=')) {
                    let k = k.trim().to_ascii_lowercase();
                    let v = Some(v.trim().to_string());
                    if k.starts_with("left") {
                        last.1 = v;
                    } else if k.starts_with("right") {
                        last.2 = v;
                    }
                }
            }
        }
    }
    out
}

/// Left -> right order: walk `right =` links from the screen that has no left neighbour;
/// screens the walk misses are appended in file order.
fn parse_order(text: &str) -> Vec<String> {
    let chunks = split_chunks(text);
    let screens: Vec<String> = parse_screens(&chunks).into_iter().map(|s| s.0).collect();
    let links = parse_links(&chunks);
    let right_of = |n: &str| links.iter().find(|l| l.0 == n).and_then(|l| l.2.clone());
    let has_left = |n: &str| links.iter().any(|l| l.0 == n && l.1.is_some());

    let mut order = Vec::new();
    let mut seen = HashSet::new();
    if let Some(start) = screens
        .iter()
        .find(|s| !has_left(s) && right_of(s).is_some())
    {
        let mut cur = Some(start.clone());
        while let Some(n) = cur {
            if !seen.insert(n.clone()) {
                break;
            }
            cur = right_of(&n);
            order.push(n);
        }
    }
    for s in screens {
        if seen.insert(s.clone()) {
            order.push(s);
        }
    }
    order
}

/// New server.conf: screens + links regenerated from `order`, every other section and
/// loose line kept verbatim in place. Per-screen option lines are kept.
fn build_conf(order: &[String], old: &str) -> String {
    let nl = if old.contains("\r\n") { "\r\n" } else { "\n" };
    let chunks = split_chunks(old);
    let screen_opts = parse_screens(&chunks);

    let mut screens = vec!["section: screens".to_string()];
    for n in order {
        screens.push(format!("\t{n}:"));
        if let Some((_, opts)) = screen_opts.iter().find(|s| &s.0 == n) {
            screens.extend(opts.iter().cloned());
        }
    }
    screens.push("end".into());

    let mut links = vec!["section: links".to_string()];
    for (i, n) in order.iter().enumerate() {
        links.push(format!("\t{n}:"));
        if i > 0 {
            links.push(format!("\t\tleft = {}", order[i - 1]));
        }
        if i + 1 < order.len() {
            links.push(format!("\t\tright = {}", order[i + 1]));
        }
    }
    links.push("end".into());

    let mut out: Vec<String> = Vec::new();
    let (mut wrote_screens, mut wrote_links) = (false, false);
    for c in chunks {
        match c {
            Chunk::Section(name, _) if name == "screens" => {
                if !wrote_screens {
                    out.extend(screens.iter().cloned());
                    wrote_screens = true;
                    if !wrote_links {
                        out.push(String::new());
                        out.extend(links.iter().cloned());
                        wrote_links = true;
                    }
                }
            }
            Chunk::Section(name, _) if name == "links" => {
                // links already written right after screens; drop a following blank-line duplicate
            }
            Chunk::Section(_, lines) => out.extend(lines),
            Chunk::Line(l) => {
                // collapse runs of blank lines left behind by the dropped links section
                if l.trim().is_empty() && out.last().is_some_and(|p| p.trim().is_empty()) {
                    continue;
                }
                out.push(l)
            }
        }
    }
    if !wrote_screens {
        let mut head = screens;
        head.push(String::new());
        head.extend(links);
        head.push(String::new());
        head.extend(out);
        out = head;
    }
    let mut s = out.join(nl);
    s.push_str(nl);
    s
}

// ---------------------------------------------------------------- hotkeys (server.conf options)
//
// Deskflow server.conf `section: options` accepts `keystroke(<mods>+<key>) = switchToScreen(<name>)`.
// Names verified in deskflow v1.26.0 sources: modifier "Super" (Windows key: VK_LWIN/VK_RWIN map to
// KeyModifierSuper in MSWindowsKeyState.cpp), keys from KeyTypes.cpp. With NumLock off the numpad
// sends the non-extended VK_END/VK_DOWN/... which Deskflow maps to KP_End/KP_Down/...; numpad 5
// sends VK_CLEAR, which maps to plain "Clear". So each binding is written twice.

const HOTKEY_MOD: &str = "Super";

/// digit -> (NumLock on name, NumLock off name)
const NUMPAD: [(&str, &str, &str); 9] = [
    ("1", "KP_1", "KP_End"),
    ("2", "KP_2", "KP_Down"),
    ("3", "KP_3", "KP_PageDown"),
    ("4", "KP_4", "KP_Left"),
    ("5", "KP_5", "Clear"),
    ("6", "KP_6", "KP_Right"),
    ("7", "KP_7", "KP_Home"),
    ("8", "KP_8", "KP_Up"),
    ("9", "KP_9", "KP_PageUp"),
];

fn digit_for_key(name: &str) -> Option<&'static str> {
    NUMPAD
        .iter()
        .find(|(_, on, off)| on.eq_ignore_ascii_case(name) || off.eq_ignore_ascii_case(name))
        .map(|(d, _, _)| *d)
}

/// `keystroke(Super+KP_1) = switchToScreen(office-pc)` -> ("1", "office-pc"). Any other line -> None.
fn parse_hotkey_line(line: &str) -> Option<(&'static str, String)> {
    let t = line.trim();
    let rest = t.strip_prefix("keystroke(")?;
    let (combo, rest) = rest.split_once(')')?;
    let action = rest.trim_start().strip_prefix('=')?.trim();
    let screen = action
        .strip_prefix("switchToScreen(")?
        .strip_suffix(')')?
        .trim();
    let mut parts: Vec<&str> = combo.split('+').map(str::trim).collect();
    let key = parts.pop()?;
    if parts.len() != 1 || !parts[0].eq_ignore_ascii_case(HOTKEY_MOD) || screen.is_empty() {
        return None;
    }
    Some((digit_for_key(key)?, screen.to_string()))
}

/// Win+numpad switchToScreen bindings in `section: options`, sorted by key, one per key.
fn parse_hotkeys(text: &str) -> Vec<Hotkey> {
    let mut out: Vec<Hotkey> = Vec::new();
    for c in split_chunks(text) {
        if let Chunk::Section(name, lines) = c
            && name == "options"
        {
            for l in &lines {
                if let Some((d, screen)) = parse_hotkey_line(l)
                    && !out.iter().any(|h| h.key == d)
                {
                    out.push(Hotkey {
                        key: d.to_string(),
                        screen,
                        monitor: None,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.key.cmp(&b.key));
    out
}

/// Suggestion (shown by the UI, never written on its own): 1-3 = the server's left/middle/right
/// monitor, then 4..9 = the Deskflow clients in machines.json order. Screen names come from each
/// machine's deskflow_name, else its name. No server machine = no monitor keys.
pub fn default_hotkeys(cfg: &Config) -> Vec<Hotkey> {
    let hk = |k: u32, screen: String, monitor: Option<u32>| Hotkey {
        key: k.to_string(),
        screen,
        monitor,
    };
    let mut out = Vec::new();
    if let Some(server) = config::server_name(cfg) {
        out.extend((0..3).map(|i| hk(i + 1, server.clone(), Some(i))));
    }
    let clients = cfg
        .machines
        .iter()
        .filter(|m| m.role == "deskflow-client")
        .map(|m| m.deskflow_name.clone().unwrap_or_else(|| m.name.clone()));
    out.extend((4..=9).zip(clients).map(|(k, screen)| hk(k, screen, None)));
    out
}

/// `monitor` only on keys bound to the server screen, and below `displays` (the number of
/// monitors this PC has; 0 = unknown, then only the server rule is checked).
fn validate_monitors(
    hotkeys: &[Hotkey],
    server: Option<&str>,
    displays: usize,
) -> Result<(), String> {
    for h in hotkeys {
        let Some(m) = h.monitor else { continue };
        if !server.is_some_and(|s| s.eq_ignore_ascii_case(h.screen.trim())) {
            return Err(format!(
                "key {}: a monitor can only be picked for the Deskflow server ({}), not {}",
                h.key,
                server.unwrap_or("none set"),
                h.screen
            ));
        }
        if displays > 0 && m as usize >= displays {
            return Err(format!(
                "key {}: monitor {} does not exist (this PC has {displays})",
                h.key,
                m + 1
            ));
        }
    }
    Ok(())
}

/// Keys 1-9, no key twice, every screen in `section: screens`. Returns the hotkeys sorted
/// by key, with screen names spelled as in the file.
fn validate_hotkeys(hotkeys: &[Hotkey], text: &str) -> Result<Vec<Hotkey>, String> {
    let screens: Vec<String> = parse_screens(&split_chunks(text))
        .into_iter()
        .map(|s| s.0)
        .collect();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for h in hotkeys {
        if !NUMPAD.iter().any(|(d, _, _)| *d == h.key) {
            return Err(format!("bad key {:?}: use 1-9", h.key));
        }
        if !seen.insert(h.key.clone()) {
            return Err(format!("key {} used twice", h.key));
        }
        let Some(name) = screens
            .iter()
            .find(|s| s.eq_ignore_ascii_case(h.screen.trim()))
        else {
            return Err(format!(
                "screen {:?} is not in server.conf (have: {})",
                h.screen,
                screens.join(", ")
            ));
        };
        out.push(Hotkey {
            key: h.key.clone(),
            screen: name.clone(),
            monitor: h.monitor,
        });
    }
    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}

fn hotkey_lines(hotkeys: &[Hotkey]) -> Vec<String> {
    let mut v = Vec::new();
    for h in hotkeys {
        if let Some((_, on, off)) = NUMPAD.iter().find(|(d, _, _)| *d == h.key) {
            for k in [on, off] {
                v.push(format!(
                    "\tkeystroke({HOTKEY_MOD}+{k}) = switchToScreen({})",
                    h.screen
                ));
            }
        }
    }
    v
}

/// server.conf with our hotkey lines replaced; every other line kept verbatim.
fn build_hotkeys(old: &str, hotkeys: &[Hotkey]) -> String {
    let nl = if old.contains("\r\n") { "\r\n" } else { "\n" };
    let new_lines = hotkey_lines(hotkeys);
    let mut out: Vec<String> = Vec::new();
    let mut wrote = false;
    for c in split_chunks(old) {
        match c {
            Chunk::Section(name, lines) if name == "options" && !wrote => {
                let mut kept: Vec<String> = lines
                    .into_iter()
                    .filter(|l| parse_hotkey_line(l).is_none())
                    .collect();
                let closed = kept.last().is_some_and(|l| l.trim() == "end");
                let at = if closed { kept.len() - 1 } else { kept.len() };
                kept.splice(at..at, new_lines.iter().cloned());
                if !closed {
                    kept.push("end".into());
                }
                out.extend(kept);
                wrote = true;
            }
            Chunk::Section(name, lines) if name == "options" => {
                // a second options section: drop our lines there, the first one holds them
                out.extend(lines.into_iter().filter(|l| parse_hotkey_line(l).is_none()));
            }
            Chunk::Section(_, lines) => out.extend(lines),
            Chunk::Line(l) => out.push(l),
        }
    }
    if !wrote && !new_lines.is_empty() {
        if out.last().is_some_and(|l| !l.trim().is_empty()) {
            out.push(String::new());
        }
        out.push("section: options".into());
        out.extend(new_lines);
        out.push("end".into());
    }
    let mut s = out.join(nl);
    s.push_str(nl);
    s
}

/// Saved list from machines.json; when none is saved yet, the Win+numpad lines in server.conf.
pub fn current_hotkeys() -> Result<Vec<Hotkey>, String> {
    if let Some(h) = config::load()?.hotkeys {
        return Ok(h);
    }
    let p = server_conf_path()?;
    let text =
        std::fs::read_to_string(&p).map_err(|e| format!("can't read {}: {e}", p.display()))?;
    Ok(parse_hotkeys(&text))
}

#[tauri::command]
pub async fn hotkeys_get() -> Result<Vec<Hotkey>, String> {
    current_hotkeys()
}

/// Suggested bindings (see `default_hotkeys`). Nothing is written.
#[tauri::command]
pub async fn hotkeys_default() -> Result<Vec<Hotkey>, String> {
    Ok(default_hotkeys(&config::load()?))
}

/// Validate, save to machines.json, replace the Win+numpad lines in server.conf (backup first),
/// restart the server, then reload the app's monitor-key table (passive keyboard hook).
#[tauri::command]
pub async fn hotkeys_save(app: tauri::AppHandle, hotkeys: Vec<Hotkey>) -> Result<String, String> {
    let p = server_conf_path()?;
    let old =
        std::fs::read_to_string(&p).map_err(|e| format!("can't read {}: {e}", p.display()))?;
    let mut cfg = config::load()?;
    let hotkeys = validate_hotkeys(&hotkeys, &old)?;
    let displays = crate::shortcuts::displays(&app).len();
    validate_monitors(&hotkeys, config::server_name(&cfg).as_deref(), displays)?;
    cfg.hotkeys = Some(hotkeys.clone());
    config::save(&cfg)?;
    let bak = util::backup_path(&p);
    std::fs::copy(&p, &bak).map_err(|e| format!("backup failed, server.conf unchanged: {e}"))?;
    std::fs::write(&p, build_hotkeys(&old, &hotkeys))
        .map_err(|e| format!("can't write {}: {e}", p.display()))?;
    let list = if hotkeys.is_empty() {
        "none".to_string()
    } else {
        hotkeys
            .iter()
            .map(|h| match h.monitor {
                Some(m) => format!("Win+{} {} monitor {}", h.key, h.screen, m + 1),
                None => format!("Win+{} {}", h.key, h.screen),
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut msg = format!("saved hotkeys {list}; backup {}; ", bak.display());
    match restart_server().await {
        Ok(s) => msg.push_str(&s),
        Err(e) => msg.push_str(&format!("restart failed: {e}")),
    }
    let problems = crate::shortcuts::register_all(&app, &hotkeys);
    if !problems.is_empty() {
        msg.push_str(&format!("; monitor keys inactive: {}", problems.join("; ")));
    }
    Ok(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Machine id for the live tests below: `DESK_MONITOR_LIVE_CLIENT=<id> cargo test -- --ignored`.
    fn live_client() -> String {
        std::env::var("DESK_MONITOR_LIVE_CLIENT")
            .expect("set DESK_MONITOR_LIVE_CLIENT to a machine id from machines.json")
    }

    /// Live: sends a WRONG password to the live client (it needs a login_command) and expects
    /// a refusal with nothing changed. Proves the stdin path end to end.
    #[tokio::test]
    #[ignore]
    async fn login_server_refuses_wrong_password() {
        let r = login_server(live_client(), "definitely-not-the-password".into()).await;
        assert_eq!(r, Err("Wrong password. Nothing changed.".to_string()));
    }

    /// Live: this PC runs the tested Deskflow, so no warning.
    #[tokio::test]
    #[ignore]
    async fn installed_deskflow_is_the_tested_one() {
        assert_eq!(version_note().await, "");
    }

    #[test]
    fn restart_waits_for_a_core_that_was_not_there_before() {
        let out = "\"deskflow-core.exe\",\"456\",\"Services\",\"0\",\"9,000 K\"
\"x.exe\",\"9\"
";
        assert_eq!(core_pids(out), vec![456]);
        assert!(!is_new_core(&[456], &[456]), "old core still there");
        assert!(!is_new_core(&[456], &[456, 789]), "old one not gone yet");
        assert!(!is_new_core(&[456], &[]), "none running");
        assert!(is_new_core(&[456], &[789]));
    }

    /// Live: restarts the real Deskflow server core through the service (mouse link blips ~2 s)
    /// and closes the Deskflow window. `cargo test -- --ignored restart_server_through_service`
    #[tokio::test]
    #[ignore]
    async fn restart_server_through_service() {
        let r = restart_server().await;
        assert_eq!(
            r,
            Ok("restarted the Deskflow server (no Deskflow window needed)".to_string())
        );
        let (_, out, _) = util::run_hidden("tasklist", &["/FO", "CSV", "/NH"], 5000)
            .await
            .unwrap();
        assert_eq!(
            parse_tasklist(&out),
            (false, true),
            "window closed, core running"
        );
    }

    /// Live: reads this PC's real Deskflow log; the live client must be connected right now.
    #[tokio::test]
    #[ignore]
    async fn wait_for_link_sees_connected_server() {
        let m = machine_by_id(&live_client()).unwrap();
        assert_eq!(wait_for_link(&m).await, "Mouse link connected.");
    }

    #[tokio::test]
    async fn login_server_needs_a_password() {
        let r = login_server("no-such-machine".into(), String::new()).await;
        assert_eq!(r, Err("Type your password first.".to_string()));
    }

    const CONF: &str = "section: screens\n\toffice-pc:\n\thomeserver:\n\tstudio-mac:\nend\n\nsection: links\n\toffice-pc:\n\t\tleft = studio-mac\n\tstudio-mac:\n\t\tright = office-pc\n\t\tleft = homeserver\n\thomeserver:\n\t\tright = studio-mac\nend\n\nsection: options\n\tclipboardSharing = true\nend\n";

    #[test]
    fn order_from_links() {
        assert_eq!(
            parse_order(CONF),
            vec!["homeserver", "studio-mac", "office-pc"]
        );
    }

    #[test]
    fn order_falls_back_to_screens() {
        let t = "section: screens\n\ta:\n\tb:\nend\n";
        assert_eq!(parse_order(t), vec!["a", "b"]);
    }

    #[test]
    fn write_keeps_options_and_roundtrips() {
        let order: Vec<String> = ["studio-mac", "homeserver", "office-pc"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let out = build_conf(&order, CONF);
        assert!(out.contains("section: options\n\tclipboardSharing = true\nend\n"));
        assert_eq!(out.matches("section: links").count(), 1);
        assert_eq!(out.matches("section: screens").count(), 1);
        assert!(!out.contains("\n\n\n"));
        assert_eq!(parse_order(&out), order);
        assert!(out.contains("\tstudio-mac:\n\t\tright = homeserver\n"));
        assert!(out.contains("\toffice-pc:\n\t\tleft = homeserver\nend"));
    }

    #[test]
    fn write_keeps_crlf_and_screen_options() {
        let old = "section: screens\r\n\ta:\r\n\t\thalfDuplexCapsLock = false\r\n\tb:\r\nend\r\nsection: options\r\n\tx = 1\r\nend\r\n";
        let order = vec!["b".to_string(), "a".to_string()];
        let out = build_conf(&order, old);
        assert!(out.contains("\ta:\r\n\t\thalfDuplexCapsLock = false\r\n"));
        assert!(out.contains("section: options\r\n\tx = 1\r\nend\r\n"));
        assert_eq!(parse_order(&out), order);
    }

    #[test]
    fn write_from_empty_file() {
        let order = vec!["a".to_string(), "b".to_string()];
        let out = build_conf(&order, "");
        assert_eq!(parse_order(&out), order);
    }

    #[test]
    fn validate_rejects_dupes_and_bad_names() {
        assert!(validate_order(&["a".into(), "A".into()]).is_err());
        assert!(validate_order(&["a b".into()]).is_err());
        assert!(validate_order(&[]).is_err());
        assert!(validate_order(&["office-pc".into(), "studio-mac".into()]).is_ok());
    }

    #[test]
    fn seat_user_greeter_only() {
        let out = " 1  1000 alex         -     1487 manager       -    no  -
 c1 60578 gdm-greeter seat0 1521 greeter       tty1 yes 1min 13s ago
";
        assert!(!parse_seat_user(out));
    }

    #[test]
    fn seat_user_logged_in() {
        let out = " 2  1000 alex seat0 2100 user tty2 no -
";
        assert!(parse_seat_user(out));
    }

    #[test]
    fn recv_q_linux_ss() {
        let out = "ESTAB 1150000 0 192.0.2.5:41234 192.0.2.8:24800\n";
        assert_eq!(parse_recv_q(out), Some(1_150_000));
        assert_eq!(parse_recv_q("ESTAB 0 0 a:1 b:24800\n"), Some(0));
        assert_eq!(parse_recv_q("TIME-WAIT 9999999 0 a:1 b:24800\n"), None);
        assert_eq!(parse_recv_q(""), None);
    }

    #[test]
    fn recv_q_mac_netstat() {
        let out = "tcp4  4190000      0  192.0.2.6.52311   192.0.2.8.24800   ESTABLISHED\n\
                   tcp4       0      0  192.0.2.6.52300   192.0.2.8.24800   TIME_WAIT\n";
        assert_eq!(parse_recv_q(out), Some(4_190_000));
        assert_eq!(mb(4_190_000), "4.19 MB");
    }

    #[test]
    fn log_last_event() {
        let log = "[x] NOTE: client \"studio-mac\" has connected\n[y] NOTE: client \"homeserver\" has connected\n[z] NOTE: client \"studio-mac\" has disconnected\n";
        assert_eq!(last_client_event(log, "studio-mac"), Some(false));
        assert_eq!(last_client_event(log, "homeserver"), Some(true));
        assert_eq!(last_client_event(log, "nobody"), None);
    }

    #[test]
    fn tasklist_parse() {
        let out = "\"deskflow.exe\",\"123\",\"Console\",\"1\",\"50,000 K\"\r\n\"deskflow-core.exe\",\"456\",\"Services\",\"0\",\"9,000 K\"\r\n";
        assert_eq!(parse_tasklist(out), (true, true));
        assert_eq!(parse_tasklist("\"explorer.exe\",\"1\"\r\n"), (false, false));
    }

    #[test]
    fn ipconfig_parse() {
        let out = "   IPv4 Address. . . . . . . . . . . : 172.20.0.1\r\n   IPv4 Address. . . . . . . . . . . : 10.0.0.8(Preferred)\r\n   IPv4 Address. . . . . . . . . . . : 192.0.2.8\r\n";
        assert_eq!(parse_ipconfig_lan(out), Some("10.0.0.8".into()));
        assert_eq!(parse_ipconfig_lan("IPv4 Address. . : 192.0.2.8\r\n"), None);
    }

    fn hk(k: &str, s: &str) -> Hotkey {
        Hotkey {
            key: k.into(),
            screen: s.into(),
            monitor: None,
        }
    }

    fn three() -> Vec<Hotkey> {
        vec![
            hk("1", "office-pc"),
            hk("2", "studio-mac"),
            hk("3", "homeserver"),
        ]
    }

    fn plain(v: Vec<Hotkey>) -> Vec<Hotkey> {
        v.into_iter()
            .map(|h| Hotkey { monitor: None, ..h })
            .collect()
    }

    fn desk() -> Config {
        let m = |id: &str, role: &str, name: &str| Machine {
            id: id.into(),
            name: name.into(),
            os: "windows".into(),
            role: role.into(),
            host: "192.0.2.1".into(),
            deskflow_name: Some(name.into()),
            ..Default::default()
        };
        Config {
            machines: vec![
                m("a", "deskflow-server", "office-pc"),
                m("b", "deskflow-client", "studio-mac"),
                m("c", "deskflow-client", "homeserver"),
                m("d", "rdp", "pc-a"),
            ],
            hotkeys: None,
            desk_name: None,
        }
    }

    #[test]
    fn defaults_follow_config_names() {
        let mut cfg = desk();
        let d = default_hotkeys(&cfg);
        let got: Vec<_> = d
            .iter()
            .map(|h| (h.key.as_str(), h.screen.as_str(), h.monitor))
            .collect();
        assert_eq!(
            got,
            vec![
                ("1", "office-pc", Some(0)),
                ("2", "office-pc", Some(1)),
                ("3", "office-pc", Some(2)),
                ("4", "studio-mac", None),
                ("5", "homeserver", None),
            ]
        );
        cfg.machines[0].deskflow_name = Some("Box".into());
        cfg.machines[1].deskflow_name = None;
        let d = default_hotkeys(&cfg);
        assert_eq!(d[0].screen, "Box");
        assert_eq!(
            d[3].screen, "studio-mac",
            "no deskflow_name: falls back to the name"
        );
        cfg.machines.remove(0);
        let d = default_hotkeys(&cfg);
        assert_eq!(d.len(), 2, "no server: client keys only");
        assert_eq!((d[0].key.as_str(), d[0].monitor), ("4", None));
    }

    #[test]
    fn monitors_only_on_server_and_in_range() {
        let mut a = hk("1", "OFFICE-PC");
        a.monitor = Some(2);
        assert!(validate_monitors(std::slice::from_ref(&a), Some("office-pc"), 3).is_ok());
        assert!(validate_monitors(std::slice::from_ref(&a), Some("office-pc"), 2).is_err());
        assert!(validate_monitors(std::slice::from_ref(&a), Some("office-pc"), 0).is_ok());
        assert!(validate_monitors(std::slice::from_ref(&a), None, 3).is_err());
        let mut b = hk("4", "studio-mac");
        b.monitor = Some(0);
        assert!(validate_monitors(&[b], Some("office-pc"), 3).is_err());
        assert!(validate_monitors(&[hk("4", "studio-mac")], Some("office-pc"), 3).is_ok());
    }

    #[test]
    fn server_monitor_keys_still_get_deskflow_lines() {
        let want = validate_hotkeys(&default_hotkeys(&desk()), CONF).unwrap();
        assert_eq!(want[1].monitor, Some(1));
        let out = build_hotkeys(CONF, &want);
        assert!(out.contains("\tkeystroke(Super+KP_2) = switchToScreen(office-pc)\n"));
        assert!(out.contains("\tkeystroke(Super+KP_Down) = switchToScreen(office-pc)\n"));
        assert!(out.contains("\tkeystroke(Super+Clear) = switchToScreen(homeserver)\n"));
        assert_eq!(parse_hotkeys(&out), plain(want));
    }

    #[test]
    fn hotkeys_none_in_spec_file() {
        assert!(parse_hotkeys(CONF).is_empty());
    }

    #[test]
    fn hotkeys_write_then_parse_spec_file() {
        let want = validate_hotkeys(&three(), CONF).unwrap();
        let out = build_hotkeys(CONF, &want);
        assert_eq!(parse_hotkeys(&out), want);
        assert!(out.contains(
            "section: options\n\tclipboardSharing = true\n\tkeystroke(Super+KP_1) = switchToScreen(office-pc)\n\tkeystroke(Super+KP_End) = switchToScreen(office-pc)\n"
        ));
        assert!(out.contains("\tkeystroke(Super+KP_PageDown) = switchToScreen(homeserver)\nend\n"));
        // screens and links untouched
        let head = &CONF[..CONF.find("section: options").unwrap()];
        assert!(out.starts_with(head));
        // a second save replaces, never duplicates
        let again = build_hotkeys(&out, &[hk("5", "studio-mac")]);
        assert_eq!(parse_hotkeys(&again), vec![hk("5", "studio-mac")]);
        assert!(again.contains(
            "\tkeystroke(Super+KP_5) = switchToScreen(studio-mac)\n\tkeystroke(Super+Clear) = switchToScreen(studio-mac)\nend\n"
        ));
        assert!(!again.contains("office-pc)\n\tkeystroke"));
        // an empty list removes ours and gives back the original file byte for byte
        assert_eq!(build_hotkeys(&out, &[]), CONF);
    }

    #[test]
    fn hotkeys_crlf_roundtrip_keeps_other_lines() {
        let old = "section: screens\r\n\toffice-pc:\r\n\tstudio-mac:\r\nend\r\nsection: options\r\n\tclipboardSharing = true\r\n\tkeystroke(Control+F1) = switchToScreen(studio-mac)\r\n\tkeystroke(Super+KP_9) = switchToScreen(studio-mac)\r\nend\r\n";
        assert_eq!(parse_hotkeys(old), vec![hk("9", "studio-mac")]);
        let out = build_hotkeys(old, &[hk("1", "office-pc")]);
        assert!(
            !out.replace("\r\n", "").contains('\n'),
            "all line ends stay CRLF"
        );
        assert!(out.contains(
            "\tclipboardSharing = true\r\n\tkeystroke(Control+F1) = switchToScreen(studio-mac)\r\n"
        ));
        assert!(out.contains("\tkeystroke(Super+KP_End) = switchToScreen(office-pc)\r\nend\r\n"));
        assert_eq!(parse_hotkeys(&out), vec![hk("1", "office-pc")]);
        assert_eq!(
            build_hotkeys(&out, &[hk("9", "studio-mac")]).len(),
            old.len() + "\tkeystroke(Super+KP_PageUp) = switchToScreen(studio-mac)\r\n".len()
        );
    }

    #[test]
    fn hotkeys_parse_numlock_off_and_case() {
        let t = "section: options\n\tkeystroke(Super+KP_1) = switchToScreen(office-pc)\n\tkeystroke(Super+KP_End) = switchToScreen(office-pc)\n\tkeystroke(Super+KP_Down) = switchToScreen(studio-mac)\n\tkeystroke(super + kp_pageup) = switchToScreen(z)\nend\n";
        assert_eq!(
            parse_hotkeys(t),
            vec![hk("1", "office-pc"), hk("2", "studio-mac"), hk("9", "z")]
        );
        assert_eq!(
            parse_hotkey_line("\tkeystroke(Super+Control+KP_1) = switchToScreen(a)"),
            None
        );
        assert_eq!(
            parse_hotkey_line("\tkeystroke(Super+KP_1) = switchInDirection(left)"),
            None
        );
        assert_eq!(
            parse_hotkey_line("\tkeystroke(KP_1) = switchToScreen(a)"),
            None
        );
        assert_eq!(
            parse_hotkey_line("\tkeystroke(Super+Clear) = switchToScreen(a)"),
            Some(("5", "a".into()))
        );
    }

    #[test]
    fn hotkeys_added_when_no_options_section() {
        let old = "section: screens\n\ta:\nend\n";
        let out = build_hotkeys(old, &[hk("4", "a")]);
        assert_eq!(
            out,
            "section: screens\n\ta:\nend\n\nsection: options\n\tkeystroke(Super+KP_4) = switchToScreen(a)\n\tkeystroke(Super+KP_Left) = switchToScreen(a)\nend\n"
        );
        assert_eq!(build_hotkeys(old, &[]), old);
    }

    #[test]
    fn hotkeys_validate() {
        assert!(validate_hotkeys(&[hk("0", "studio-mac")], CONF).is_err());
        assert!(validate_hotkeys(&[hk("10", "studio-mac")], CONF).is_err());
        assert!(validate_hotkeys(&[hk("1", "studio-mac"), hk("1", "office-pc")], CONF).is_err());
        assert!(validate_hotkeys(&[hk("1", "nobody")], CONF).is_err());
        assert_eq!(
            validate_hotkeys(&[hk("2", "STUDIO-MAC"), hk("1", "OFFICE-PC")], CONF).unwrap(),
            vec![hk("1", "office-pc"), hk("2", "studio-mac")]
        );
        assert!(validate_hotkeys(&[], CONF).unwrap().is_empty());
    }

    #[test]
    fn layout_save_keeps_hotkey_lines() {
        let with = build_hotkeys(CONF, &three());
        let order: Vec<String> = ["office-pc", "studio-mac", "homeserver"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let out = build_conf(&order, &with);
        let opts = &with[with.find("section: options").unwrap()..];
        assert!(out.ends_with(opts), "options block kept verbatim");
        assert_eq!(parse_hotkeys(&out), three());
        assert_eq!(parse_order(&out), order);
    }
}
