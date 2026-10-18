//! Config + Machine storage at `%APPDATA%\desk-monitor\machines.json`
//! (via `dirs::config_dir()`, which is the Roaming AppData folder on Windows).
//!
//! Other modules (deskflow.rs, rdp.rs) use:
//!   - `config::load() -> Result<Config, String>`   SYNC, small file read. Creates the file
//!     with defaults (this PC as the Deskflow server) on first run. Safe to call from async code (it is a tiny read).
//!   - `config::find(&Config, &str) -> Result<Machine, String>`   clone of the machine with that id.
//!   - `config::server_name(&Config) -> Option<String>`   Deskflow name of the deskflow-server machine.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Machine {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// "windows" | "linux" | "mac"
    pub os: String,
    /// "deskflow-server" | "deskflow-client" | "rdp"
    pub role: String,
    pub host: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deskflow_name: Option<String>,
    /// Optional, Deskflow clients: full path to a local PowerShell script (.ps1) that wakes the
    /// machine (e.g. sends Wake-on-LAN). Set = the Wake button shows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wake_script: Option<String>,
    /// Optional, Deskflow clients: command run over ssh that logs a user in on the machine's own
    /// screen. It reads the password as line 1 of stdin and exits 0 = logged in, 3 = wrong
    /// password, 4 = a log in is already running. Set = the Log in button shows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub login_command: Option<String>,
    /// Optional, Deskflow clients: command run over ssh that restarts the Deskflow client there.
    /// Set = the Restart client button shows (Fix server IP also runs it afterwards).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_restart_command: Option<String>,
}

/// An optional text setting, None when unset or blank.
pub fn opt(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// One Win+numpad binding. `key` is "1".."9". `monitor` = index into `displays_list`
/// (0 = leftmost), only when `screen` is the Deskflow server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Hotkey {
    pub key: String,
    pub screen: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub machines: Vec<Machine>,
    /// Full hotkey list (None in files written before hotkeys were stored here).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotkeys: Option<Vec<Hotkey>>,
}

/// Lowercase letters/digits/dashes, from a display name (same rule as the UI form's id slug).
fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// First-run defaults, written only when machines.json does not exist yet: one entry for this
/// PC as the Deskflow server. Deskflow names a screen after the computer name by default, so
/// COMPUTERNAME is the best guess for `deskflow_name`. Clients and RDP PCs are added in the app.
/// The server entry must exist from the start: the app's form cannot add a server, only clients.
pub fn defaults() -> Config {
    defaults_for(std::env::var("COMPUTERNAME").ok().as_deref())
}

fn defaults_for(computer: Option<&str>) -> Config {
    let name = computer
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .unwrap_or("this-pc")
        .to_string();
    let id = Some(slug(&name))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "this-pc".into());
    Config {
        machines: vec![Machine {
            id,
            name: name.clone(),
            os: "windows".into(),
            role: "deskflow-server".into(),
            host: "127.0.0.1".into(),
            deskflow_name: Some(name),
            ..Default::default()
        }],
        hotkeys: None,
    }
}

fn config_path() -> Result<PathBuf, String> {
    let base = dirs::config_dir().ok_or("cannot find the AppData folder")?;
    Ok(base.join("desk-monitor").join("machines.json"))
}

/// Unique non-empty ids; other required text fields non-empty; known os/role values.
pub fn validate(cfg: &Config) -> Result<(), String> {
    let mut seen = HashSet::new();
    for (i, m) in cfg.machines.iter().enumerate() {
        let id = m.id.trim();
        if id.is_empty() {
            return Err(format!("machine #{} has an empty id", i + 1));
        }
        if !seen.insert(id.to_string()) {
            return Err(format!("duplicate machine id '{id}'"));
        }
        if m.name.trim().is_empty() {
            return Err(format!("machine '{id}' has an empty name"));
        }
        if m.host.trim().is_empty() {
            return Err(format!("machine '{id}' has an empty host"));
        }
        if let Some(a) = m.ssh_alias.as_deref()
            && (a.is_empty()
                || a.starts_with('-')
                || !a
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '@' | '-')))
        {
            return Err(format!(
                "machine '{id}': bad ssh alias '{a}' (use letters, digits, . _ @ -, not starting with -)"
            ));
        }
        if !matches!(m.os.as_str(), "windows" | "linux" | "mac") {
            return Err(format!("machine '{id}': unknown os '{}'", m.os));
        }
        if !matches!(
            m.role.as_str(),
            "deskflow-server" | "deskflow-client" | "rdp"
        ) {
            return Err(format!("machine '{id}': unknown role '{}'", m.role));
        }
    }
    Ok(())
}

fn write_config(path: &PathBuf, cfg: &Config) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    let json =
        serde_json::to_string_pretty(cfg).map_err(|e| format!("cannot encode config: {e}"))?;
    // Write to a temp file then rename, so a crash never leaves a half-written file.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("cannot replace {}: {e}", path.display()))
}

/// Load the config (sync). On first run, writes the defaults and returns them.
pub fn load() -> Result<Config, String> {
    let path = config_path()?;
    if !path.exists() {
        let cfg = defaults();
        write_config(&path, &cfg)?;
        return Ok(cfg);
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not valid: {e}", path.display()))
}

/// Validate, then save (sync).
pub fn save(cfg: &Config) -> Result<(), String> {
    validate(cfg)?;
    write_config(&config_path()?, cfg)
}

/// Find a machine by id (returns a clone).
pub fn find(cfg: &Config, id: &str) -> Result<Machine, String> {
    cfg.machines
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .ok_or_else(|| format!("no machine with id '{id}'"))
}

/// Deskflow screen name of the deskflow-server machine (deskflow_name, else name).
pub fn server_name(cfg: &Config) -> Option<String> {
    cfg.machines
        .iter()
        .find(|m| m.role == "deskflow-server")
        .map(|m| m.deskflow_name.clone().unwrap_or_else(|| m.name.clone()))
}

#[tauri::command]
pub async fn get_config() -> Result<Config, String> {
    load()
}

#[tauri::command]
pub async fn save_config(mut config: Config) -> Result<(), String> {
    // The machines editor may send no hotkeys: keep the stored ones instead of wiping them.
    if config.hotkeys.is_none() {
        config.hotkeys = load().ok().and_then(|c| c.hotkeys);
    }
    save(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_file_without_hotkeys_loads() {
        let t = r#"{"machines":[{"id":"a","name":"A","os":"windows","role":"deskflow-server","host":"h","deskflow_name":"office-pc"}]}"#;
        let c: Config = serde_json::from_str(t).unwrap();
        assert!(c.hotkeys.is_none());
        assert_eq!(server_name(&c).as_deref(), Some("office-pc"));
        assert!(!serde_json::to_string(&c).unwrap().contains("hotkeys"));
    }

    #[test]
    fn action_fields_are_optional_and_roundtrip() {
        let t = r#"{"id":"b","name":"B","os":"linux","role":"deskflow-client","host":"192.0.2.5","login_command":"login-helper","client_restart_command":"  "}"#;
        let m: Machine = serde_json::from_str(t).unwrap();
        assert_eq!(opt(&m.login_command), Some("login-helper"));
        assert_eq!(opt(&m.client_restart_command), None, "blank = unset");
        assert_eq!(m.wake_script, None);
        let out = serde_json::to_string(&m).unwrap();
        assert!(out.contains("login_command") && !out.contains("wake_script"));
    }

    #[test]
    fn first_run_seeds_only_this_pc() {
        let c = defaults_for(Some("Office PC"));
        assert_eq!(c.machines.len(), 1);
        let m = &c.machines[0];
        assert_eq!(
            (m.id.as_str(), m.role.as_str()),
            ("office-pc", "deskflow-server")
        );
        assert_eq!(server_name(&c).as_deref(), Some("Office PC"));
        assert!(validate(&c).is_ok());
        assert_eq!(defaults_for(None).machines[0].id, "this-pc");
        assert_eq!(defaults_for(Some("  ")).machines[0].id, "this-pc");
    }

    #[test]
    fn hotkeys_roundtrip_with_monitor() {
        let mut c = defaults();
        c.hotkeys = Some(vec![
            Hotkey {
                key: "1".into(),
                screen: "office-pc".into(),
                monitor: Some(0),
            },
            Hotkey {
                key: "4".into(),
                screen: "studio-mac".into(),
                monitor: None,
            },
        ]);
        let t = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&t).unwrap();
        assert_eq!(back.hotkeys, c.hotkeys);
        let h: Hotkey = serde_json::from_str(r#"{"key":"2","screen":"x","monitor":null}"#).unwrap();
        assert_eq!(h.monitor, None);
    }
}
