//! Talks to the Deskflow service (deskflow-daemon.exe, runs as LocalSystem) the way the Deskflow
//! window does, so the window is never needed: the service keeps the core command, starts the core
//! elevated in the user's session (it can then drive admin windows and UAC prompts), restarts it if it
//! dies, and starts it again at boot. So desk-monitor is the only app needed.
//!
//! Protocol (Deskflow v1.26.0, lib/deskflow/ipc/DaemonIpcServer.cpp): named pipe `deskflow-daemon`,
//! one UTF-8 line per message, one reply line each. `hello` -> `hello`; `elevate=yes`, `command=<cmd>`,
//! `start`, `stop` -> `ok`. No version check exists, so a Deskflow update can change it.
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

const PIPE: &str = r"\\.\pipe\deskflow-daemon";
const CORE_EXE: &str = "deskflow-core.exe";

/// The core command the Deskflow window would send: `"<dir>/deskflow-core.exe" server --settings <conf>`.
/// The core reads everything else (name, TLS, layout file) from that settings file.
pub fn core_command(install_dir: &str, settings: &str) -> Result<String, String> {
    let core =
        format!("{}/{CORE_EXE}", install_dir.trim_end_matches(['/', '\\'])).replace('\\', "/");
    let settings = settings.replace('\\', "/");
    for (what, s) in [("Deskflow path", &core), ("settings path", &settings)] {
        // The service splits each message at '=' and widens bytes one by one.
        if s.contains('=') || !s.is_ascii() || s.contains('"') {
            return Err(format!("{what} can't be sent to the Deskflow service: {s}"));
        }
    }
    let quote = |s: &str| {
        if s.contains(' ') {
            format!("\"{s}\"")
        } else {
            s.to_string()
        }
    };
    Ok(format!(
        "{} server --settings {}",
        quote(&core),
        quote(&settings)
    ))
}

/// Send `lines` in order; each must get its expected reply (`hello` for hello, `ok` otherwise).
async fn talk(lines: &[String]) -> Result<(), String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    let mut pipe = None;
    for _ in 0..5 {
        match ClientOptions::new().open(PIPE) {
            Ok(p) => {
                pipe = Some(p);
                break;
            }
            // ERROR_PIPE_BUSY: another client (the Deskflow window) is connected for a moment
            Err(e) if e.raw_os_error() == Some(231) => {
                tokio::time::sleep(Duration::from_millis(300)).await
            }
            Err(e) => {
                return Err(format!(
                    "can't reach the Deskflow service ({e}) - is the \"Deskflow\" Windows service running?"
                ));
            }
        }
    }
    let mut pipe = pipe.ok_or("the Deskflow service stayed busy")?;
    for line in lines {
        let want = if line == "hello" { "hello" } else { "ok" };
        pipe.write_all(format!("{line}\n").as_bytes())
            .await
            .map_err(|e| format!("Deskflow service: write failed: {e}"))?;
        let mut got = Vec::new();
        let mut byte = [0u8; 1];
        let read = async {
            loop {
                let n = pipe.read(&mut byte).await?;
                if n == 0 || byte[0] == b'\n' {
                    return Ok::<_, std::io::Error>(());
                }
                got.push(byte[0]);
            }
        };
        match timeout(Duration::from_secs(3), read).await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(format!("Deskflow service: read failed: {e}")),
            Err(_) => {
                return Err(format!(
                    "Deskflow service did not answer \"{}\"",
                    verb(line)
                ));
            }
        }
        let got = String::from_utf8_lossy(&got).trim().to_string();
        if got != want {
            return Err(format!(
                "Deskflow service answered \"{got}\" to \"{}\"",
                verb(line)
            ));
        }
    }
    Ok(())
}

/// The message name without its value (the command line is long and not interesting in an error).
fn verb(line: &str) -> &str {
    line.split('=').next().unwrap_or(line)
}

/// Ask the service to (re)start the core, elevated. `start` on a running core restarts it.
pub async fn start_core(command: &str) -> Result<(), String> {
    talk(&[
        "hello".into(),
        "elevate=yes".into(),
        format!("command={command}"),
        "start".into(),
    ])
    .await
}

/// Just the handshake: proves the service is there and speaks this protocol (live test only).
#[cfg(test)]
pub async fn ping() -> Result<(), String> {
    talk(&["hello".into(), "noop".into()]).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_matches_what_the_deskflow_window_sends() {
        assert_eq!(
            core_command(
                r"C:\Program Files\Deskflow",
                r"C:\Users\me\AppData\Roaming\Deskflow\Deskflow.conf"
            )
            .unwrap(),
            "\"C:/Program Files/Deskflow/deskflow-core.exe\" server --settings C:/Users/me/AppData/Roaming/Deskflow/Deskflow.conf"
        );
    }

    #[test]
    fn command_refuses_what_the_service_would_mangle() {
        assert!(core_command(r"C:\a=b", r"C:\s.conf").is_err());
        assert!(core_command(r"C:\Deskflow", r"C:\Users\Mé\s.conf").is_err());
    }

    /// Live: handshake with the real Deskflow service on this PC. Changes nothing.
    #[tokio::test]
    #[ignore]
    async fn service_answers_hello() {
        ping().await.unwrap();
    }
}
