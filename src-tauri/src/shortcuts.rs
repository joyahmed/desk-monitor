//! The app's own monitor keys: Win+numpad keys bound to a monitor of the Deskflow server.
//!
//! Deskflow can only switch to a screen (it restores the last cursor position there), so for
//! keys with a `monitor` the server.conf line brings input home and this module then puts the
//! cursor in the centre of the chosen display. Works only while the app runs.
//!
//! Deskflow 1.26 itself owns these combos through Win32 RegisterHotKey, and a combo can have only
//! one owner. So the app does NOT register anything: it installs a passive WH_KEYBOARD_LL hook on
//! its own thread that only watches, always calls CallNextHookEx and never swallows a key.
//!   - NumLock on:  VK_NUMPAD1..9.
//!   - NumLock off: VK_END/DOWN/NEXT/LEFT/CLEAR/RIGHT/HOME/UP/PRIOR WITHOUT LLKHF_EXTENDED (the
//!     dedicated navigation keys carry the extended flag, numpad keys do not; Clear = numpad 5).
//!   - Injected events (LLKHF_INJECTED, e.g. Deskflow's synthetic input) are ignored.
//!
//! The hook callback only queues the monitor index; a worker thread waits 150 ms and moves the
//! cursor. A hook that fails to install never crashes: the error is logged and kept for
//! `hotkeys_status`.
use crate::config::Hotkey;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};
use tauri::AppHandle;

/// One monitor of this PC in physical pixels. Index 0 = leftmost.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DisplayInfo {
    pub index: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

/// Digit (1..9, index 0 unused) -> monitor index, for hotkeys that have a monitor.
static TABLE: RwLock<[Option<u32>; 10]> = RwLock::new([None; 10]);
/// Why the hook is not running (None = installed, or not tried yet).
static HOOK_ERROR: Mutex<Option<String>> = Mutex::new(None);
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Sort left to right (then top to bottom) and number them.
fn sort_displays(mut v: Vec<DisplayInfo>) -> Vec<DisplayInfo> {
    v.sort_by_key(|d| (d.x, d.y));
    for (i, d) in v.iter_mut().enumerate() {
        d.index = i as u32;
    }
    v
}

/// All monitors, leftmost first. Empty when Tauri can't list them.
pub fn displays(app: &AppHandle) -> Vec<DisplayInfo> {
    let primary = app.primary_monitor().ok().flatten().map(|m| *m.position());
    let list = app.available_monitors().unwrap_or_default();
    sort_displays(
        list.iter()
            .map(|m| DisplayInfo {
                index: 0,
                name: m.name().cloned().unwrap_or_default(),
                x: m.position().x,
                y: m.position().y,
                width: m.size().width,
                height: m.size().height,
                primary: primary == Some(*m.position()),
            })
            .collect(),
    )
}

/// Centre of a display (physical pixels).
fn centre(d: &DisplayInfo) -> (i32, i32) {
    (d.x + (d.width / 2) as i32, d.y + (d.height / 2) as i32)
}

// Win32 virtual-key codes, numeric so the mapping is testable on any OS.
const VK_CLEAR: u32 = 0x0C;
const VK_PRIOR: u32 = 0x21;
const VK_NEXT: u32 = 0x22;
const VK_END: u32 = 0x23;
const VK_HOME: u32 = 0x24;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const VK_NUMPAD0: u32 = 0x60;
const VK_NUMPAD9: u32 = 0x69;

/// Numpad digit 1..9 for a key, or None. `extended` = LLKHF_EXTENDED. NumLock on gives
/// VK_NUMPADn; NumLock off gives navigation VKs, which count only WITHOUT the extended flag
/// (with it they come from the dedicated arrow/Home/End/PgUp/PgDn block).
pub fn numpad_digit(vk: u32, extended: bool, numlock: bool) -> Option<u8> {
    if numlock {
        return (vk > VK_NUMPAD0 && vk <= VK_NUMPAD9).then(|| (vk - VK_NUMPAD0) as u8);
    }
    if extended {
        return None;
    }
    Some(match vk {
        VK_END => 1,
        VK_DOWN => 2,
        VK_NEXT => 3,
        VK_LEFT => 4,
        VK_CLEAR => 5,
        VK_RIGHT => 6,
        VK_HOME => 7,
        VK_UP => 8,
        VK_PRIOR => 9,
        _ => return None,
    })
}

/// Digit -> monitor table for the hotkeys that have a monitor.
fn table(hotkeys: &[Hotkey]) -> [Option<u32>; 10] {
    let mut t = [None; 10];
    for h in hotkeys {
        if let (Some(m), Ok(d)) = (h.monitor, h.key.parse::<usize>())
            && (1..=9).contains(&d)
        {
            t[d] = Some(m);
        }
    }
    t
}

/// Install a new hotkey table (startup and after `hotkeys_save`). Nothing is registered with
/// Windows. Returns the current problems (the same list `hotkeys_status` reports).
pub fn register_all(_app: &AppHandle, hotkeys: &[Hotkey]) -> Vec<String> {
    *TABLE.write().unwrap_or_else(|e| e.into_inner()) = table(hotkeys);
    problems()
}

/// Empty = hook installed. Otherwise one line saying why it is not.
fn problems() -> Vec<String> {
    if let Some(e) = HOOK_ERROR.lock().unwrap_or_else(|e| e.into_inner()).clone() {
        return vec![e];
    }
    if !HOOK_INSTALLED.load(Ordering::SeqCst) {
        return vec!["keyboard hook not installed".into()];
    }
    Vec::new()
}

/// Worker: for each queued monitor, wait for Deskflow to bring input home, then centre the cursor.
#[cfg_attr(not(windows), allow(dead_code))]
fn mover(app: AppHandle, rx: std::sync::mpsc::Receiver<u32>) {
    while let Ok(monitor) = rx.recv() {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let list = displays(&app);
        let Some(d) = list.get(monitor as usize) else {
            eprintln!(
                "[shortcuts] monitor {monitor} not found ({} present)",
                list.len()
            );
            continue;
        };
        let (x, y) = centre(d);
        if let Err(e) = set_cursor(x, y) {
            eprintln!("[shortcuts] can't move the cursor: {e}");
        }
    }
}

#[cfg(windows)]
fn set_cursor(x: i32, y: i32) -> Result<(), String> {
    // Tauri runs per-monitor DPI aware, so these are physical pixels like DisplayInfo.
    unsafe { windows::Win32::UI::WindowsAndMessaging::SetCursorPos(x, y) }
        .map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn set_cursor(_x: i32, _y: i32) -> Result<(), String> {
    Err("moving the cursor is only supported on Windows".into())
}

#[cfg(windows)]
mod hook {
    use super::{TABLE, numpad_digit};
    use std::sync::OnceLock;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::mpsc::Sender;
    use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, GetMessageW, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED,
        MSG, SetWindowsHookExW, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    const VK_LWIN: u32 = 0x5B;
    const VK_RWIN: u32 = 0x5C;

    pub static SENDER: OnceLock<Sender<u32>> = OnceLock::new();
    static LWIN: AtomicBool = AtomicBool::new(false);
    static RWIN: AtomicBool = AtomicBool::new(false);
    /// VK currently held down (to skip auto-repeat), 0 = none.
    static HELD: AtomicU32 = AtomicU32::new(0);

    /// Passive: only looks, always passes the key on, never swallows. Must return fast.
    unsafe extern "system" fn proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if code == HC_ACTION as i32 {
            // SAFETY: for WH_KEYBOARD_LL with HC_ACTION, lparam points to a KBDLLHOOKSTRUCT.
            let k = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
            if k.flags.0 & LLKHF_INJECTED.0 == 0 {
                let msg = wparam.0 as u32;
                let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
                let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
                on_key(k.vkCode, k.flags.0 & LLKHF_EXTENDED.0 != 0, down, up);
            }
        }
        unsafe { CallNextHookEx(None, code, wparam, lparam) }
    }

    fn on_key(vk: u32, extended: bool, down: bool, up: bool) {
        match vk {
            VK_LWIN if down || up => LWIN.store(down, Ordering::Relaxed),
            VK_RWIN if down || up => RWIN.store(down, Ordering::Relaxed),
            _ if up => {
                let _ = HELD.compare_exchange(vk, 0, Ordering::Relaxed, Ordering::Relaxed);
            }
            _ if down => {
                if HELD.swap(vk, Ordering::Relaxed) == vk {
                    return; // auto-repeat
                }
                if !(LWIN.load(Ordering::Relaxed) || RWIN.load(Ordering::Relaxed)) {
                    return;
                }
                // The VK class already tells which NumLock state produced it, and NumLock read
                // with GetKeyState on this hook thread can be stale, so accept either state.
                let Some(d) =
                    numpad_digit(vk, extended, true).or(numpad_digit(vk, extended, false))
                else {
                    return;
                };
                let Some(m) = TABLE.read().map(|t| t[d as usize]).unwrap_or(None) else {
                    return;
                };
                if let Some(tx) = SENDER.get() {
                    let _ = tx.send(m);
                }
            }
            _ => {}
        }
    }

    /// Hook thread: install, report the result, then pump messages forever (LL hooks need it).
    pub fn run(ready: Sender<Result<(), String>>) {
        let hmod = unsafe { GetModuleHandleW(None) }.ok().map(|h| h.into());
        if let Err(e) = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc), hmod, 0) } {
            let _ = ready.send(Err(format!("keyboard hook not installed: {e}")));
            return;
        }
        let _ = ready.send(Ok(()));
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {}
    }
}

/// Start the cursor worker and the hook thread; wait (max 2 s) for the install result.
#[cfg(windows)]
fn start_hook(app: &AppHandle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::channel::<u32>();
    let _ = hook::SENDER.set(tx);
    let app = app.clone();
    std::thread::Builder::new()
        .name("monitor-key-mover".into())
        .spawn(move || mover(app, rx))
        .map_err(|e| format!("can't start the cursor thread: {e}"))?;
    let (rtx, rrx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("monitor-key-hook".into())
        .spawn(move || hook::run(rtx))
        .map_err(|e| format!("can't start the keyboard hook thread: {e}"))?;
    rrx.recv_timeout(std::time::Duration::from_secs(2))
        .map_err(|_| "keyboard hook thread did not report".to_string())?
}

#[cfg(not(windows))]
fn start_hook(_app: &AppHandle) -> Result<(), String> {
    Err("monitor keys are only supported on Windows".into())
}

/// Startup: install the hook, then load the saved hotkeys (or server.conf). Never fails the app.
pub fn init(app: &AppHandle) {
    match start_hook(app) {
        Ok(()) => HOOK_INSTALLED.store(true, Ordering::SeqCst),
        Err(e) => {
            eprintln!("[shortcuts] {e}");
            *HOOK_ERROR.lock().unwrap_or_else(|e| e.into_inner()) = Some(e);
        }
    }
    match crate::deskflow::current_hotkeys() {
        Ok(h) => {
            register_all(app, &h);
        }
        Err(e) => eprintln!("[shortcuts] could not read hotkeys: {e}"),
    }
}

#[tauri::command]
pub async fn displays_list(app: AppHandle) -> Result<Vec<DisplayInfo>, String> {
    Ok(displays(&app))
}

/// Empty = the keyboard hook is installed; otherwise the reason it is not.
#[tauri::command]
pub async fn hotkeys_status() -> Result<Vec<String>, String> {
    Ok(problems())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(name: &str, x: i32, w: u32, primary: bool) -> DisplayInfo {
        DisplayInfo {
            index: 9,
            name: name.into(),
            x,
            y: 0,
            width: w,
            height: 1440,
            primary,
        }
    }

    #[test]
    fn displays_sorted_left_to_right() {
        // office-pc: DISPLAY1 primary at 0, DISPLAY2 at 2560, DISPLAY3 at -1920.
        let v = sort_displays(vec![
            d("DISPLAY1", 0, 2560, true),
            d("DISPLAY2", 2560, 2560, false),
            d("DISPLAY3", -1920, 1920, false),
        ]);
        let names: Vec<_> = v.iter().map(|d| (d.index, d.name.as_str())).collect();
        assert_eq!(
            names,
            vec![(0, "DISPLAY3"), (1, "DISPLAY1"), (2, "DISPLAY2")]
        );
        assert_eq!(centre(&v[0]), (-960, 720));
        assert_eq!(centre(&v[1]), (1280, 720));
    }

    fn hk(k: &str, m: Option<u32>) -> Hotkey {
        Hotkey {
            key: k.into(),
            screen: "office-pc".into(),
            monitor: m,
        }
    }

    #[test]
    fn table_only_monitor_keys() {
        let t = table(&[hk("1", Some(0)), hk("4", None), hk("5", Some(2))]);
        assert_eq!(t[1], Some(0));
        assert_eq!(t[4], None);
        assert_eq!(t[5], Some(2));
        assert_eq!(t.iter().filter(|x| x.is_some()).count(), 2);
    }

    #[test]
    fn numlock_on_digits() {
        for n in 1..=9u8 {
            assert_eq!(numpad_digit(VK_NUMPAD0 + n as u32, false, true), Some(n));
        }
        assert_eq!(numpad_digit(VK_NUMPAD0, false, true), None); // Num0
        assert_eq!(numpad_digit(VK_END, false, true), None);
    }

    #[test]
    fn numlock_off_digits_only_without_extended_flag() {
        let vks = [
            VK_END, VK_DOWN, VK_NEXT, VK_LEFT, VK_CLEAR, VK_RIGHT, VK_HOME, VK_UP, VK_PRIOR,
        ];
        let got: Vec<_> = vks.iter().map(|&v| numpad_digit(v, false, false)).collect();
        assert_eq!(got, (1..=9).map(Some).collect::<Vec<_>>());
        // The dedicated arrow/Home/End/PgUp/PgDn keys carry LLKHF_EXTENDED.
        assert!(vks.iter().all(|&v| numpad_digit(v, true, false).is_none()));
        assert_eq!(numpad_digit(VK_NUMPAD0 + 1, false, false), None); // needs NumLock on
        assert_eq!(numpad_digit(0x41, false, false), None); // 'A'
    }
}
