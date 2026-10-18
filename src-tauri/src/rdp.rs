use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RdpStatus {
    pub id: String,
    pub online: bool,
    pub ip: Option<String>,
    pub reason: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RdpSession {
    pub name: String,
    pub user: String,
    pub id: String,
    pub state: String,
}

const RDP_PORT: u16 = 3389;

/// Look up an rdp-role machine and return its host. The host is checked so it
/// can never be read as a command-line switch.
fn rdp_host(id: &str) -> Result<String, String> {
    let cfg = crate::config::load()?;
    let m = crate::config::find(&cfg, id)?;
    if m.role != "rdp" {
        return Err(format!("{} is not an rdp machine", m.name));
    }
    let host = m.host.trim().to_string();
    let ok = !host.is_empty()
        && !host.starts_with('-')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ':'));
    if !ok {
        return Err(format!("bad host for {}: {host:?}", m.name));
    }
    Ok(host)
}

fn pick_addr(host: &str) -> Result<SocketAddr, String> {
    let addrs: Vec<SocketAddr> = (host, RDP_PORT)
        .to_socket_addrs()
        .map_err(|e| format!("can't resolve {host}: {e}"))?
        .collect();
    addrs
        .iter()
        .find(|a| a.is_ipv4())
        .or_else(|| addrs.first())
        .copied()
        .ok_or_else(|| format!("can't resolve {host}: no address"))
}

#[tauri::command]
pub async fn rdp_status(id: String) -> Result<RdpStatus, String> {
    let host = rdp_host(&id)?;
    let h = host.clone();
    let resolved = tokio::task::spawn_blocking(move || pick_addr(&h))
        .await
        .map_err(|e| format!("lookup failed: {e}"))?;
    let (online, ip, reason) = match resolved {
        Err(e) => (false, None, e),
        Ok(addr) => {
            let ip = addr.ip().to_string();
            match crate::util::tcp_probe(&ip, RDP_PORT, 1500).await {
                Ok(()) => (true, Some(ip), "port 3389 open".to_string()),
                Err(e) => (
                    false,
                    Some(ip),
                    format!("port 3389 closed or no answer: {e}"),
                ),
            }
        }
    };
    Ok(RdpStatus {
        id,
        online,
        ip,
        reason,
        checked_at: crate::util::now_rfc3339(),
    })
}

#[tauri::command]
pub async fn rdp_connect(id: String) -> Result<(), String> {
    let host = rdp_host(&id)?;
    let mut cmd = std::process::Command::new("mstsc");
    cmd.arg(format!("/v:{host}"));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
        cmd.creation_flags(0x0000_0008 | 0x0000_0200);
    }
    cmd.spawn()
        .map(|_child| ())
        .map_err(|e| format!("can't start mstsc: {e}"))
}

/// Turn a qwinsta/logoff/taskkill failure into a plain sentence.
fn plain_error(what: &str, out: &str, err: &str) -> String {
    let text = format!("{err}\n{out}");
    let low = text.to_lowercase();
    if low.contains("error 5") || low.contains("access is denied") || low.contains("access denied")
    {
        return format!("{what}: access denied");
    }
    if low.contains("error 1722") || low.contains("rpc server") {
        return format!("{what}: RPC server unavailable (host off, or firewall blocks it)");
    }
    if low.contains("error 53") || low.contains("network path") || low.contains("error 1707") {
        return format!("{what}: host not reachable");
    }
    if low.contains("error 1168") || low.contains("no session exists") {
        return format!("{what}: no such session");
    }
    let first = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("unknown error");
    format!("{what}: {first}")
}

/// Parse `qwinsta` output. Columns are fixed width; positions come from the header.
fn parse_qwinsta(out: &str) -> Vec<RdpSession> {
    let mut lines = out.lines().filter(|l| !l.trim().is_empty());
    let Some(header) = lines.next() else {
        return vec![];
    };
    let up = header.to_uppercase();
    let find = |word: &str| -> Option<usize> { up.find(word).map(|b| up[..b].chars().count()) };
    let (user_col, state_col, type_col) = (find("USERNAME"), find("STATE"), find("TYPE"));

    let mut rows = Vec::new();
    for line in lines {
        let ch: Vec<char> = line.chars().collect();
        let slice = |a: usize, b: usize| -> String {
            let b = b.min(ch.len());
            if a >= b {
                String::new()
            } else {
                ch[a..b].iter().collect()
            }
        };
        let row = match (user_col, state_col) {
            (Some(u), Some(s)) if u < s => {
                let name = slice(1, u).trim().to_string();
                let region = slice(u, s);
                let mut toks: Vec<&str> = region.split_whitespace().collect();
                let id = match toks.pop() {
                    Some(t) if t.chars().all(|c| c.is_ascii_digit()) => t.to_string(),
                    _ => continue,
                };
                let user = toks.join(" ");
                let end = type_col.filter(|t| *t > s).unwrap_or(usize::MAX);
                let state = slice(s, end)
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                RdpSession {
                    name,
                    user,
                    id,
                    state,
                }
            }
            _ => match parse_loose(line) {
                Some(r) => r,
                None => continue,
            },
        };
        rows.push(row);
    }
    rows
}

/// Fallback when the header is not English: anchor on the first all-digit token.
fn parse_loose(line: &str) -> Option<RdpSession> {
    let body = line.trim_start().strip_prefix('>').unwrap_or(line);
    let toks: Vec<&str> = body.split_whitespace().collect();
    let i = toks
        .iter()
        .position(|t| t.chars().all(|c| c.is_ascii_digit()))?;
    let state = toks.get(i + 1).copied().unwrap_or("").to_string();
    let before = &toks[..i];
    let is_sys = |w: &str| {
        let w = w.to_lowercase();
        w == "console" || w == "services" || w.starts_with("rdp-")
    };
    let (name, user) = match before.len() {
        0 => (String::new(), String::new()),
        1 if is_sys(before[0]) => (before[0].to_string(), String::new()),
        1 => (String::new(), before[0].to_string()),
        _ => (before[0].to_string(), before[1..].join(" ")),
    };
    Some(RdpSession {
        name,
        user,
        id: toks[i].to_string(),
        state,
    })
}

#[tauri::command]
pub async fn rdp_sessions(id: String) -> Result<Vec<RdpSession>, String> {
    let host = rdp_host(&id)?;
    let server = format!("/server:{host}");
    let (code, out, err) = crate::util::run_hidden("qwinsta", &[&server], 8000)
        .await
        .map_err(|e| format!("can't read sessions: {e}"))?;
    if code != 0 {
        return Err(plain_error("can't read sessions", &out, &err));
    }
    Ok(parse_qwinsta(&out))
}

#[tauri::command]
pub async fn rdp_logoff(id: String, session_id: String) -> Result<String, String> {
    let host = rdp_host(&id)?;
    let sid = session_id.trim();
    if sid.is_empty() || sid.len() > 6 || !sid.chars().all(|c| c.is_ascii_digit()) {
        return Err("session id must be digits".into());
    }
    let server = format!("/server:{host}");
    let (code, out, err) = crate::util::run_hidden("logoff", &[sid, &server], 8000)
        .await
        .map_err(|e| format!("can't log off: {e}"))?;
    if code != 0 {
        return Err(plain_error("can't log off", &out, &err));
    }
    Ok(format!("Logged off session {sid} on {host}"))
}

#[tauri::command]
pub async fn rdp_fix_paste(id: String) -> Result<String, String> {
    let host = rdp_host(&id)?;
    let (code, out, err) =
        crate::util::run_hidden("taskkill", &["/s", &host, "/im", "rdpclip.exe", "/f"], 8000)
            .await
            .map_err(|e| format!("can't fix paste: {e}"))?;
    if code != 0 {
        return Err(plain_error("can't fix paste", &out, &err));
    }
    Ok(format!(
        "Stopped rdpclip.exe on {host}. Disconnect and reconnect your RDP window so paste works again."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = concat!(
        " SESSIONNAME       USERNAME                 ID  STATE   TYPE        DEVICE \n",
        ">console           alex                      1  Active                     \n",
        " rdp-tcp#0         bob smith                 2  Active  rdpwd              \n",
        " services                                    0  Disc                       \n",
        "                   ghost                     3  Disc                       \n",
        " rdp-tcp                                 65536  Listen                     \n",
    );

    fn t(r: &RdpSession) -> (&str, &str, &str, &str) {
        (&r.name, &r.user, &r.id, &r.state)
    }

    #[test]
    fn parses_columns() {
        let r = parse_qwinsta(SAMPLE);
        assert_eq!(r.len(), 5);
        assert_eq!(t(&r[0]), ("console", "alex", "1", "Active"));
        assert_eq!(t(&r[1]), ("rdp-tcp#0", "bob smith", "2", "Active"));
        assert_eq!(t(&r[2]), ("services", "", "0", "Disc"));
        assert_eq!(t(&r[3]), ("", "ghost", "3", "Disc"));
        assert_eq!(t(&r[4]), ("rdp-tcp", "", "65536", "Listen"));
    }

    #[test]
    fn empty_and_header_only() {
        assert!(parse_qwinsta("").is_empty());
        assert!(parse_qwinsta(" SESSIONNAME USERNAME ID STATE TYPE DEVICE\n").is_empty());
    }

    #[test]
    fn loose_fallback() {
        let r =
            parse_qwinsta("NAME USER NUM ZUSTAND\n>console alex 1 Aktiv\n services 0 Getrennt\n");
        assert_eq!(r.len(), 2);
        assert_eq!(t(&r[0]), ("console", "alex", "1", "Aktiv"));
        assert_eq!(t(&r[1]), ("services", "", "0", "Getrennt"));
    }

    #[test]
    fn errors_are_plain() {
        assert_eq!(
            plain_error("can't read sessions", "", "Error 5 Access is denied"),
            "can't read sessions: access denied"
        );
        assert!(plain_error("x", "", "Error 1722 The RPC server is unavailable").contains("RPC"));
    }
}
