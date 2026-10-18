import {
  displaysList,
  hotkeysGet,
  type ClientStatus,
  type Config,
  type DisplayInfo,
  type Hotkey,
  type Machine,
  type Role
} from '$lib/api';
import { useMonitorStatus } from './use-monitor-status.svelte';
import { useMonitorActions } from './use-monitor-actions.svelte';
import { useMonitorSections } from './use-monitor-sections.svelte';
import { useLayoutEditor } from './use-layout-editor.svelte';
import { MONITOR_ACTIONS, checkedTime } from '$lib/monitors/monitor-actions';
import {
  EDGE_DROP,
  deskGeometry,
  deskflowName,
  displayNameOf,
  fitDesk,
  isDeskflow,
  type GeoBlock,
  type GeoScreen,
  type PxRect
} from '$lib/monitors/desk-map-geometry';
import {
  OS_LABEL,
  THIS_PC,
  hotkeyChips,
  hotkeysFor,
  screenLabel,
  sizeTier,
  stateWord,
  summaryChips,
  toneOf,
  type Tier
} from '$lib/monitors/desk-map-view';

export interface ScreenView {
  key: string;
  name: string;
  /** What fits on the drawn screen: full name, short name, or '' (glyph only). */
  label: string;
  sub: string;
  os: string;
  state: string;
  /** Token name for the dot: ok / warn / bad / info / off. */
  tone: string;
  word: string;
  tier: Tier;
  /** How the screen is drawn: dim (connected), login (box outline), asleep (LED), hung, off. */
  look: Look;
  glyph: number;
  cls: string;
  style: string;
  title: string;
  selected: boolean;
}

/** dark = no data yet; the selected screen is lit on top of any look. */
export type Look = 'dim' | 'login' | 'asleep' | 'hung' | 'off' | 'dark';

const CLIENT_LOOK: Record<string, Look> = {
  connected: 'dim',
  disconnected: 'login',
  asleep: 'asleep',
  hung: 'hung',
  offline: 'off',
  unknown: 'off'
};

export interface BlockView {
  key: string;
  placed: boolean;
  style: string;
  label: string;
  title: string;
  /** The name plate under the block, below the desk edge: name, then a muted note. */
  plate: { name: string; note: string; tone: string; lit: boolean; style: string };
  screens: ScreenView[];
}

const BEZEL = 3; // px gap drawn between neighbouring monitors
const FRAME_Y = 56; // the map frame's own top + bottom padding (DeskMap .frame)
const MAP_SHARE = 0.42; // the map frame may take this much of the window height
const POOL_SPREAD = 2.2; // the light pool is this much wider than the lit screen
const MIN_DESK_H = 120;
const PLATE_GAP = 8; // the name plate sits this far under the desk edge
const PLATE_NOTE_W = 150; // a plate narrower than this shows the name only
const rectStyle = (r: PxRect) =>
  `left:${r.left}px;top:${r.top}px;width:${r.width}px;height:${r.height}px`;

export const useDeskMap = (
  getConfig: () => Config,
  getTick: () => number,
  onedit: (m: Machine | null, role: Role) => void
) => {
  const machines = $derived(getConfig().machines.filter(isDeskflow));
  const clients = $derived(machines.filter((m) => m.role === 'deskflow-client'));
  const nameOf = (screen: string) => displayNameOf(machines, screen);
  const serverName = $derived.by(() => {
    const s = machines.find((m) => m.role === 'deskflow-server');
    return s ? deskflowName(s) : '';
  });

  // The server is polled too: the backend reports connected while deskflow-core runs.
  const status = useMonitorStatus(() => machines, getTick);
  const actions = useMonitorActions((m) => void status.refreshOne(m));
  const layout = useLayoutEditor();
  const sections = useMonitorSections();

  let displays = $state<DisplayInfo[]>([]);
  let hotkeys = $state<Hotkey[]>([]);
  let editing = $state(false);
  let selectedKey = $state<string | null>(null);
  let width = $state(0);
  let viewH = $state(800);

  const loadDisplays = async () => {
    try {
      displays = await displaysList();
    } catch {
      displays = []; // server falls back to one nominal screen
    }
  };
  const reloadHotkeys = async () => {
    try {
      hotkeys = await hotkeysGet();
    } catch {
      hotkeys = [];
    }
  };
  void loadDisplays();
  void reloadHotkeys();

  /** Action: watches the map frame's width (and the window height) to scale the desk. */
  const measure = (node: HTMLElement) => {
    const ro = new ResizeObserver(([e]) => {
      width = e.contentRect.width;
    });
    ro.observe(node);
    const onresize = () => {
      viewH = window.innerHeight;
    };
    onresize();
    window.addEventListener('resize', onresize);
    return {
      destroy: () => {
        ro.disconnect();
        window.removeEventListener('resize', onresize);
      }
    };
  };

  // Before the layout loads (or if it fails) the config order stands in.
  const order = $derived(
    layout.loaded && !layout.loadError && layout.order.length > 0
      ? layout.order
      : machines.map(deskflowName)
  );
  const geo = $derived(deskGeometry(order, displays, machines));
  // Sized from both the frame's width and the height the map may take; edit mode keeps one
  // row because dragging reorders left/right.
  const fit = $derived(
    fitDesk(geo, width, Math.max(MIN_DESK_H, viewH * MAP_SHARE - FRAME_Y), editing)
  );

  const statusOf = (m: Machine | null): ClientStatus | null =>
    m ? (status.statuses[m.id] ?? null) : null;
  const stateOf = (m: Machine | null) => statusOf(m)?.state ?? 'unknown';
  const wordOf = (m: Machine | null) =>
    stateWord(
      m,
      statusOf(m),
      m ? (status.checking[m.id] ?? false) : false,
      m ? (actions.busy[m.id] ?? null) : null
    );

  const isServer = (m: Machine | null) => m?.role === 'deskflow-server';
  const lookOf = (m: Machine | null, state: string): Look => {
    if (!m || !statusOf(m)) return 'dark';
    if (isServer(m)) return state === 'connected' ? 'dim' : 'hung';
    return CLIENT_LOOK[state] ?? 'off';
  };
  /** The ring action pulses only when the machine needs the user now. */
  const needsYou = (state: string) => state === 'disconnected' || state === 'asleep';
  const subOf = (b: GeoBlock, s: GeoScreen) =>
    isServer(b.machine) ? [THIS_PC, s.sub].filter(Boolean).join(' · ') : s.sub;

  const screenView = (b: GeoBlock, s: GeoScreen, r: PxRect): ScreenView => {
    const w = r.width - BEZEL * 2;
    const h = r.height - BEZEL * 2;
    const tier = sizeTier(w, h);
    const state = stateOf(b.machine);
    const word = wordOf(b.machine);
    const tone = toneOf(b.machine, state);
    const selected = s.key === selectedKey;
    const look = lookOf(b.machine, state);
    const sub = subOf(b, s);
    return {
      key: s.key,
      name: b.display,
      label: screenLabel(b.display, w, tier),
      sub,
      os: b.machine?.os ?? 'other',
      state,
      tone,
      word,
      tier,
      look,
      glyph: tier === 'full' ? 20 : 16,
      cls: `screen look-${look} ${tier}${selected ? ' selected' : ''}`,
      style: rectStyle({ left: r.left + BEZEL, top: r.top + BEZEL, width: w, height: h }),
      title: [b.display, b.display !== b.name ? b.name : '', sub, word]
        .filter(Boolean)
        .join(' - '),
      selected
    };
  };

  const tip = (b: GeoBlock) => (b.display !== b.name ? `${b.display} (${b.name})` : b.display);

  const plateOf = (b: GeoBlock, rect: PxRect, views: ScreenView[]) => {
    const state = stateOf(b.machine);
    return {
      name: b.display,
      // a narrow plate keeps just the name; the screen's tooltip and the detail say the rest
      note:
        rect.width >= PLATE_NOTE_W
          ? [isServer(b.machine) ? THIS_PC : '', wordOf(b.machine)].filter(Boolean).join(' · ')
          : '',
      tone: toneOf(b.machine, state),
      lit: views.some((v) => v.selected),
      style: `left:${rect.left}px;top:${rect.top + rect.height + EDGE_DROP + PLATE_GAP}px;width:${rect.width}px`
    };
  };

  const blocks = $derived<BlockView[]>(
    fit.blocks.map(({ block: b, rect, screens }, i) => {
      const views = b.screens.map((s, j) => screenView(b, s, screens[j]));
      return {
        key: b.name,
        placed: b.placed,
        style: rectStyle(rect),
        label: `${b.display}, position ${i + 1} of ${layout.order.length}. Arrow keys or drag move it.`,
        title: b.placed ? tip(b) : `${b.display} - not in the Deskflow layout yet`,
        plate: plateOf(b, rect, views),
        screens: views
      };
    })
  );
  const edges = $derived(fit.edges.map((e, i) => ({ key: i, style: rectStyle(e) })));

  // The light pool under the lit (selected) screen, on its row's desk edge. With nothing
  // selected it fades out where it was, so the next pick slides it from there.
  let lastPool = '';
  const pool = $derived.by(() => {
    for (const { rect, screens, block } of fit.blocks) {
      const i = block.screens.findIndex((s) => s.key === selectedKey);
      if (i < 0) continue;
      const r = screens[i];
      const w = Math.round(r.width * POOL_SPREAD);
      const cx = rect.left + r.left + r.width / 2;
      lastPool = `left:${Math.round(cx - w / 2)}px;top:${rect.top + rect.height + EDGE_DROP}px;width:${w}px`;
      return { on: true, style: lastPool };
    }
    return { on: false, style: lastPool };
  });

  const summary = $derived(
    summaryChips(
      clients.map((m) => status.statuses[m.id]).filter((s): s is ClientStatus => !!s),
      statusOf(machines.find(isServer) ?? null)
    )
  );
  const keyChips = $derived(hotkeyChips(hotkeys, displays, serverName, nameOf));

  const picked = $derived.by(() => {
    for (const b of geo.blocks)
      for (const s of b.screens) if (s.key === selectedKey) return { b, s };
    return null;
  });

  const detail = $derived.by(() => {
    if (!picked) return null;
    const { b, s } = picked;
    const m = b.machine;
    const st = statusOf(m);
    const busy = m ? (actions.busy[m.id] ?? null) : null;
    const asked = m ? MONITOR_ACTIONS.find((a) => a.key === actions.asking[m.id]) : undefined;
    const state = stateOf(m);
    const at = st ? checkedTime(st.checked_at) : '';
    const buttons = m
      ? MONITOR_ACTIONS.filter((a) => a.showFor(m)).map((a) => ({
          key: a.key,
          label: a.label,
          variant: a.variant,
          busy: busy === a.key,
          run: () => (a.confirm || a.password ? actions.ask(m, a.key) : void actions.run(m, a.key))
        }))
      : [];
    return {
      key: s.key,
      name: b.display,
      /** The technical Deskflow / ssh name, shown small; '' when it equals the display name. */
      techName: b.display !== b.name ? b.name : '',
      label: m?.label ?? '',
      os: m?.os ?? 'other',
      osLabel: m ? (OS_LABEL[m.os] ?? m.os) : 'Not in machines.json',
      sub: subOf(b, s),
      host: m ? m.host : '',
      state,
      tone: toneOf(m, state),
      word: wordOf(m),
      reason: st && !busy ? st.reason : '',
      checked: at,
      /** The meta line after the state word: OS, label, tech name, sub, host, checked time. */
      meta: [
        m ? (OS_LABEL[m.os] ?? m.os) : 'Not in machines.json',
        m?.label ?? '',
        b.display !== b.name ? b.name : '',
        subOf(b, s),
        m ? m.host : '',
        at ? `checked ${at}` : ''
      ].filter(Boolean),
      result: m ? (actions.results[m.id] ?? null) : null,
      busy: busy !== null,
      /** The ONE round action (Log in), or null; `links` are the rest, as text links. */
      ring: buttons.find((a) => a.variant === 'ring') ?? null,
      links: buttons.filter((a) => a.variant !== 'ring'),
      pulse: needsYou(state),
      confirm:
        m && asked?.confirm
          ? {
              text: asked.confirm,
              yes: () => void actions.run(m, asked.key),
              no: () => actions.cancel(m)
            }
          : null,
      password:
        m && asked?.password
          ? {
              label: asked.password,
              value: actions.secret[m.id] ?? '',
              oninput: (v: string) => actions.setSecret(m, v),
              yes: () => void actions.run(m, asked.key),
              no: () => actions.cancel(m)
            }
          : null,
      chips: hotkeyChips(hotkeysFor(hotkeys, b.name, s.monitor), displays, serverName, nameOf),
      edit: m ? () => onedit(m, m.role) : null,
      add: () => onedit(null, 'deskflow-client')
    };
  });

  // Leaving a machine drops a half-typed password and any open question, so coming back
  // never finds a pre-filled password one Enter away from logging in.
  const leave = () => {
    const m = picked?.b.machine;
    if (m) actions.cancel(m);
  };

  const select = (key: string) => {
    leave();
    selectedKey = selectedKey === key ? null : key;
  };

  const toggleEdit = () => {
    leave();
    if (editing) layout.revert();
    editing = !editing;
    selectedKey = null;
  };

  return {
    get machineCount() {
      return machines.length;
    },
    get editing() {
      return editing;
    },
    get editLabel() {
      return editing ? (layout.dirty ? 'Discard' : 'Done') : 'Edit layout';
    },
    get canEdit() {
      return layout.loaded && !layout.loadError && layout.order.length > 0;
    },
    get deskWidth() {
      return fit.width;
    },
    get deskHeight() {
      return fit.height;
    },
    get blocks() {
      return blocks;
    },
    get edges() {
      return edges;
    },
    get pool() {
      return pool;
    },
    get summary() {
      return summary;
    },
    get hotkeyChips() {
      return keyChips;
    },
    get detail() {
      return detail;
    },
    get hotkeysOpen() {
      return sections.hotkeysOpen;
    },
    get idleNote() {
      return machines.length > 0
        ? 'Click a screen for its details and fixes.'
        : 'No Deskflow machines yet. Add one.';
    },
    get layoutNote() {
      if (!layout.loaded) return 'Loading layout...';
      if (layout.loadError) return `Layout not read: ${layout.loadError}`;
      return '';
    },
    layout,
    measure,
    select,
    toggleEdit,
    toggleHotkeys: sections.toggleHotkeys,
    reloadHotkeys,
    addMonitor: () => onedit(null, 'deskflow-client')
  };
};

export type DeskMapHook = ReturnType<typeof useDeskMap>;
export type DetailView = NonNullable<DeskMapHook['detail']>;
export type LayoutHook = DeskMapHook['layout'];
