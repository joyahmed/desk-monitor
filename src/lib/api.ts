import { invoke } from '@tauri-apps/api/core';

export type Os = 'windows' | 'linux' | 'mac';
export type Role = 'deskflow-server' | 'deskflow-client' | 'rdp';

export interface Machine {
  id: string;
  name: string;
  label?: string;
  os: Os;
  role: Role;
  host: string;
  ssh_alias?: string;
  deskflow_name?: string;
  /** Deskflow client: local .ps1 that wakes it. Set = Wake shows. */
  wake_script?: string;
  /** Deskflow client: ssh command, password on stdin line 1; exit 0 ok, 3 wrong, 4 busy. Set = Log in shows. */
  login_command?: string;
  /** Deskflow client: ssh command that restarts its Deskflow client. Set = Restart client shows. */
  client_restart_command?: string;
}

export interface Config {
  machines: Machine[];
  /** Saved hotkeys; leave out to keep the stored ones (save_config never wipes them). */
  hotkeys?: Hotkey[] | null;
}

export type ClientState = 'connected' | 'hung' | 'disconnected' | 'asleep' | 'offline' | 'unknown';

export interface ClientStatus {
  id: string;
  state: ClientState;
  reason: string;
  checked_at: string; // RFC3339
}

export type DeskflowAction =
  | 'restart_client'
  | 'login_server'
  | 'wake'
  | 'set_remote_host'
  | 'restart_server';

export interface RdpStatus {
  id: string;
  online: boolean;
  ip: string | null;
  reason: string;
  checked_at: string; // RFC3339
}

export interface RdpSession {
  name: string;
  user: string;
  id: string;
  state: string;
}

export const getConfig = () => invoke<Config>('get_config');
export const saveConfig = (config: Config) => invoke<void>('save_config', { config });

export const deskflowStatus = (id: string) => invoke<ClientStatus>('deskflow_status', { id });
export const deskflowAction = (id: string, action: DeskflowAction) =>
  invoke<string>('deskflow_action', { id, action });
/** Log in on a client's own screen via its login_command; the client checks the password (never stored). */
export const loginServer = (id: string, password: string) =>
  invoke<string>('login_server', { id, password });
export const layoutGet = () => invoke<string[]>('layout_get');
export const layoutSave = (order: string[]) => invoke<string>('layout_save', { order });

export const rdpStatus = (id: string) => invoke<RdpStatus>('rdp_status', { id });
export const rdpConnect = (id: string) => invoke<void>('rdp_connect', { id });
export const rdpSessions = (id: string) => invoke<RdpSession[]>('rdp_sessions', { id });
// Tauri maps JS camelCase keys to Rust snake_case args, so `sessionId` -> `session_id`.
export const rdpLogoff = (id: string, sessionId: string) =>
  invoke<string>('rdp_logoff', { id, sessionId });
export const rdpFixPaste = (id: string) => invoke<string>('rdp_fix_paste', { id });

// Win + numpad hotkeys in Deskflow server.conf. `key` is "1".."9" (numpad digit).
export interface Hotkey {
  key: string;
  screen: string; // Deskflow screen name, must exist in server.conf
  /** Index into displaysList() (0 = leftmost). Only when `screen` is the Deskflow server. */
  monitor?: number | null;
}

/** One monitor of this PC, physical pixels. Sorted by x, index 0 = leftmost. */
export interface DisplayInfo {
  index: number;
  name: string;
  x: number;
  y: number;
  width: number;
  height: number;
  primary: boolean;
}

/** Saved list from machines.json; falls back to parsing server.conf. */
export const hotkeysGet = () => invoke<Hotkey[]>('hotkeys_get');
/** Suggestion (1-3 server monitors left/middle/right, 4-9 the Deskflow clients in list order). Writes nothing. */
export const hotkeysDefault = () => invoke<Hotkey[]>('hotkeys_default');
/** Saves to machines.json, rewrites the Win+numpad lines in server.conf (backup first),
 * restarts the server and re-registers the app's monitor hotkeys. Returns what happened. */
export const hotkeysSave = (hotkeys: Hotkey[]) => invoke<string>('hotkeys_save', { hotkeys });
/** Monitors of this PC, sorted left to right. */
export const displaysList = () => invoke<DisplayInfo[]>('displays_list');
/** Problems from registering the app's own global hotkeys (empty = all fine). */
export const hotkeysStatus = () => invoke<string[]>('hotkeys_status');

// Start with Windows (off unless the user turns it on) and a real quit.
export const autostartGet = () => invoke<boolean>('autostart_get');
export const autostartSet = (enabled: boolean) => invoke<void>('autostart_set', { enabled });
/** Ends the whole app (the window close button only hides to the tray). */
export const appQuit = () => invoke<void>('app_quit');
