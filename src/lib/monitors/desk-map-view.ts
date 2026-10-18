import type { ClientState, ClientStatus, DeskflowAction, DisplayInfo, Hotkey, Machine } from '$lib/api';
import { MONITOR_ACTIONS } from './monitor-actions';
import { positionName } from './hotkey-data';
import { stateColour } from '$lib/ui/state-colour';

/** Worst first, so the summary chips lead with what hurts. */
const SUMMARY_ORDER: { state: ClientState; word: string }[] = [
  { state: 'hung', word: 'hung' },
  { state: 'disconnected', word: 'disconnected' },
  { state: 'asleep', word: 'asleep' },
  { state: 'offline', word: 'offline' },
  { state: 'unknown', word: 'unknown' },
  { state: 'connected', word: 'ok' }
];

export const OS_LABEL: Record<string, string> = {
  windows: 'Windows',
  linux: 'Ubuntu',
  mac: 'macOS'
};

export interface ChipView {
  key: string;
  label: string;
  state?: string;
  title?: string;
  /** Hotkey chips only: the key part ("Win 1", display font) and where it goes (body font). */
  keyText?: string;
  target?: string;
}

/** Clients by state, worst first. The server (this PC) counts as ok while running and gets
 * its own "server stopped" chip otherwise, so the chips always add up to the machine count. */
export const summaryChips = (list: ClientStatus[], server: ClientStatus | null = null): ChipView[] => {
  const all = server?.state === 'connected' ? [...list, server] : list;
  const stopped: ChipView[] =
    server && server.state !== 'connected'
      ? [{ key: 'server', label: 'server stopped', state: 'hung', title: 'Deskflow server is not running' }]
      : [];
  return [
    ...stopped,
    ...SUMMARY_ORDER.map(({ state, word }) => ({
      state,
      word,
      n: all.filter((s) => s.state === state).length
    }))
      .filter((c) => c.n > 0)
      .map((c) => ({ key: c.state, label: `${c.n} ${c.word}`, state: c.state, title: `${c.n} ${c.state}` }))
  ];
};

/** Maps a Deskflow screen name to the label shown for it. */
export type NameOf = (screen: string) => string;

const hotkeyTarget = (h: Hotkey, displays: DisplayInfo[], serverName: string, nameOf: NameOf) =>
  h.screen === serverName && h.monitor != null
    ? `${nameOf(h.screen)} ${positionName(displays, h.monitor)}`
    : nameOf(h.screen);

export const hotkeyChips = (
  hotkeys: Hotkey[],
  displays: DisplayInfo[],
  serverName: string,
  nameOf: NameOf = (s) => s
): ChipView[] =>
  hotkeys
    .slice()
    .sort((a, b) => a.key.localeCompare(b.key))
    .map((h) => {
      const target = hotkeyTarget(h, displays, serverName, nameOf);
      const label = `Win+Num${h.key} → ${target}`;
      return { key: h.key, label, title: label, keyText: `Win ${h.key}`, target };
    });

/** Hotkeys that land on this screen (a server monitor, or a whole client). */
export const hotkeysFor = (hotkeys: Hotkey[], name: string, monitor: number | null) =>
  hotkeys.filter(
    (h) => h.screen === name && (monitor === null || h.monitor == null || h.monitor === monitor)
  );

/** How much text a drawn screen has room for. */
export type Tier = 'full' | 'mid' | 'tiny';
export const sizeTier = (w: number, h: number): Tier =>
  w >= 150 && h >= 80 ? 'full' : w >= 84 && h >= 48 ? 'mid' : 'tiny';

/** What a tiny screen shows next to its glyph; the full name stays in the title. */
export const shortName = (name: string) => (name.length <= 6 ? name : name.slice(0, 5));

/** Room per tier, from MapScreen's CSS: inline padding, glyph, glyph gap, name font size. */
const TIER_ROOM: Record<Tier, { pad: number; glyph: number; gap: number; font: number }> = {
  full: { pad: 12, glyph: 20, gap: 8, font: 20 },
  mid: { pad: 8, glyph: 16, gap: 8, font: 14 },
  tiny: { pad: 4, glyph: 16, gap: 4, font: 12 }
};
/** Plex Sans 600 measures about 0.5em per character ("homeserver" = 70px at 14px); 0.55 leaves margin. */
const CHAR_EM = 0.55;

/** The name drawn on a screen of width `w`: the full name if it fits, else the short one,
 * else nothing (glyph + dot only; the name stays in the title). Never an ellipsis stub. */
export const screenLabel = (name: string, w: number, tier: Tier) => {
  const r = TIER_ROOM[tier];
  const room = w - r.pad * 2 - 2 - r.glyph - r.gap; // 2 = the 1px border each side
  const fits = (t: string) => t.length * r.font * CHAR_EM <= room;
  const short = shortName(name);
  return fits(name) ? name : fits(short) ? short : '';
};

/** Sub-label of the Deskflow server's screens: it is the PC this app runs on. */
export const THIS_PC = 'This PC';

const SERVER_WORD: Partial<Record<ClientState, string>> = {
  connected: 'running',
  disconnected: 'stopped'
};
const SERVER_TONE: Partial<Record<ClientState, string>> = {
  connected: 'ok',
  disconnected: 'bad'
};

const isServerMachine = (m: Machine | null) => m?.role === 'deskflow-server';

/** Token name (ok / warn / bad / info / off) for a screen's glow and dot. */
export const toneOf = (machine: Machine | null, state: ClientState) =>
  isServerMachine(machine) ? (SERVER_TONE[state] ?? 'off') : stateColour(state);

export const busyLabel = (a: DeskflowAction | null) =>
  a ? (MONITOR_ACTIONS.find((x) => x.key === a)?.busyLabel ?? 'working...') : '';

/** The state word drawn on a screen and in the detail strip. */
export const stateWord = (
  machine: Machine | null,
  status: ClientStatus | null,
  checking: boolean,
  busy: DeskflowAction | null
) => {
  if (busy) return busyLabel(busy);
  if (!machine) return 'not in app';
  if (!status) return checking ? 'checking...' : 'no data';
  if (isServerMachine(machine)) return SERVER_WORD[status.state] ?? status.state;
  return status.state;
};
