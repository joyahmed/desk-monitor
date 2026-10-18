import type { DisplayInfo, Machine } from '$lib/api';
import { positionName } from './hotkey-data';

/** One physical screen, in desk units (physical px), relative to its block. */
export interface GeoScreen {
  key: string;
  /** Index into displaysList() for the Deskflow server's monitors, else null. */
  monitor: number | null;
  sub: string;
  primary: boolean;
  x: number;
  y: number;
  w: number;
  h: number;
}

/** One Deskflow screen name: the server (its real monitors) or a client (one nominal screen). */
export interface GeoBlock {
  /** The Deskflow screen name: the key sent to the backend. */
  name: string;
  /** What the UI shows: the machine's display name. */
  display: string;
  machine: Machine | null;
  /** false = machine is in machines.json but not in the server.conf layout. */
  placed: boolean;
  x: number;
  y: number;
  w: number;
  h: number;
  screens: GeoScreen[];
}

export interface DeskGeometry {
  blocks: GeoBlock[];
  width: number;
  height: number;
}

const NOMINAL_H = 1080;
const RATIO = 16 / 9;
const GAP = 0.08; // gap between machines, as a share of the unit height

export const isDeskflow = (m: Machine) =>
  m.role === 'deskflow-server' || m.role === 'deskflow-client';

export const deskflowName = (m: Machine) => m.deskflow_name || m.name;

/** The name to show for a Deskflow screen name: the machine's display `name`, else the screen name. */
export const displayNameOf = (machines: Machine[], screen: string) =>
  machines.find((m) => isDeskflow(m) && deskflowName(m) === screen)?.name ?? screen;

const single = (name: string, h: number) => {
  const w = Math.round(h * RATIO);
  const screens: GeoScreen[] = [
    { key: name, monitor: null, sub: '', primary: false, x: 0, y: 0, w, h }
  ];
  return { screens, w, h };
};

const serverScreens = (name: string, displays: DisplayInfo[], unit: number) => {
  if (displays.length === 0) return single(name, unit);
  // Side by side in their real left-to-right order, true size, bottoms on one baseline.
  // Real y offsets are ignored on purpose: on a desk the screens stand on the same surface.
  let x = 0;
  const screens: GeoScreen[] = displays
    .slice()
    .sort((a, b) => a.x - b.x)
    .map((d) => {
      const s = {
        key: `${name}#${d.index}`,
        monitor: d.index,
        sub: positionName(displays, d.index),
        primary: d.primary,
        x,
        y: 0,
        w: d.width,
        h: d.height
      };
      x += d.width;
      return s;
    });
  const h = Math.max(...screens.map((s) => s.h));
  return {
    screens: screens.map((s) => ({ ...s, y: h - s.h })),
    w: x,
    h
  };
};

/**
 * Lays the desk out left to right in `order` (deskflow names, from layoutGet()).
 * The server is drawn as its real monitors; each client is one 16:9 screen as tall as the
 * server's primary monitor. Machines missing from `order` go on the right, `placed: false`.
 * All blocks stand on one baseline (bottom aligned), like screens on a desk.
 */
export const deskGeometry = (
  order: string[],
  displays: DisplayInfo[],
  machines: Machine[]
): DeskGeometry => {
  const deskflow = machines.filter(isDeskflow);
  const byName = new Map(deskflow.map((m) => [deskflowName(m), m]));
  const unit = (displays.find((d) => d.primary) ?? displays[0])?.height ?? NOMINAL_H;
  const gap = Math.round(unit * GAP);

  const names = [
    ...order.map((name) => ({ name, placed: true })),
    ...deskflow
      .map(deskflowName)
      .filter((n) => !order.includes(n))
      .map((name) => ({ name, placed: false }))
  ];

  let x = 0;
  const raw = names.map(({ name, placed }) => {
    const machine = byName.get(name) ?? null;
    const s =
      machine?.role === 'deskflow-server'
        ? serverScreens(name, displays, unit)
        : single(name, unit);
    const block: GeoBlock = { name, display: machine?.name ?? name, machine, placed, x, y: 0, w: s.w, h: s.h, screens: s.screens };
    x += s.w + gap;
    return block;
  });

  const height = raw.length ? Math.max(...raw.map((b) => b.h)) : 0;
  return {
    blocks: raw.map((b) => ({ ...b, y: height - b.h })),
    width: raw.length ? x - gap : 0,
    height
  };
};

// ---------------------------------------------------------------- fit to the frame (px)

export interface PxRect {
  left: number;
  top: number;
  width: number;
  height: number;
}

export interface PlacedBlock {
  block: GeoBlock;
  /** Relative to the desk. */
  rect: PxRect;
  /** One per block.screens entry, relative to the block. */
  screens: PxRect[];
}

export interface DeskLayout {
  scale: number;
  width: number;
  height: number;
  blocks: PlacedBlock[];
  /** The desk edge under each row (relative to the desk). */
  edges: PxRect[];
}

/** px between two rows of screens; the desk edge of the upper row sits in it. */
export const ROW_GAP = 28;
export const EDGE_DROP = 9; // desk edge this far under a row's baseline
/** Room under each row for the machine names, which stand below the desk edge. */
export const LABEL_H = 30;
const EDGE_OUT = 12; // and this far past its ends
const MAX_ROWS = 3;
const ROW_GAIN = 1.1; // another row must make screens 10% bigger to be worth it

const rowWidth = (bs: GeoBlock[], gap: number) =>
  bs.reduce((a, b) => a + b.w, 0) + gap * Math.max(0, bs.length - 1);

/** Splits blocks (in order) into `rows` contiguous rows with the narrowest widest row. */
const splitRows = (bs: GeoBlock[], rows: number, gap: number): GeoBlock[][] => {
  if (rows <= 1 || bs.length <= 1) return [bs];
  let best: GeoBlock[][] = [bs];
  let bestW = Infinity;
  for (let i = 1; i < bs.length; i++) {
    const rest = splitRows(bs.slice(i), rows - 1, gap);
    const w = Math.max(rowWidth(bs.slice(0, i), gap), ...rest.map((r) => rowWidth(r, gap)));
    if (w < bestW) {
      bestW = w;
      best = [bs.slice(0, i), ...rest];
    }
  }
  return best;
};

const fitScale = (rows: GeoBlock[][], gap: number, availW: number, availH: number) => {
  const wide = Math.max(...rows.map((r) => rowWidth(r, gap)));
  const tall = rows.reduce((a, r) => a + Math.max(...r.map((b) => b.h)), 0);
  return Math.min(
    availW / wide,
    (availH - ROW_GAP * (rows.length - 1) - LABEL_H * rows.length) / tall
  );
};

/**
 * Fits the desk into the frame using both its width and the height it may take. The machines
 * keep their order (reading order: left to right, then the next row). More rows are used only
 * when that makes the screens clearly bigger; `oneRow` (edit mode: dragging is left/right only)
 * keeps a single row. Every edge is rounded once in desk px, so neighbours share pixels and
 * every screen in a row stands on the same baseline.
 */
export const fitDesk = (
  geo: DeskGeometry,
  availW: number,
  availH: number,
  oneRow: boolean
): DeskLayout => {
  const empty: DeskLayout = { scale: 0, width: 0, height: 0, blocks: [], edges: [] };
  if (!geo.blocks.length || availW <= 0 || availH <= 0) return empty;
  const first = geo.blocks[0];
  const gap = geo.blocks.length > 1 ? geo.blocks[1].x - first.x - first.w : 0;

  let rows = [geo.blocks];
  let scale = fitScale(rows, gap, availW, availH);
  const most = oneRow ? 1 : Math.min(MAX_ROWS, geo.blocks.length);
  for (let n = 2; n <= most; n++) {
    const r = splitRows(geo.blocks, n, gap);
    const s = fitScale(r, gap, availW, availH);
    if (s > scale * ROW_GAIN) {
      rows = r;
      scale = s;
    }
  }
  if (!(scale > 0)) return empty;

  const width = Math.round(Math.max(...rows.map((r) => rowWidth(r, gap))) * scale);
  const blocks: PlacedBlock[] = [];
  const edges: PxRect[] = [];
  let top = 0;
  for (const row of rows) {
    const rowH = Math.round(Math.max(...row.map((b) => b.h)) * scale);
    const baseline = top + rowH;
    const span = rowWidth(row, gap) * scale;
    const offset = (width - span) / 2; // centre a shorter row
    let x = 0;
    for (const b of row) {
      const x0 = Math.round(offset + x * scale);
      const x1 = Math.round(offset + (x + b.w) * scale);
      const h = Math.round(b.h * scale);
      const screens = b.screens.map((s) => {
        const sx0 = Math.round(offset + (x + s.x) * scale) - x0;
        const sx1 = Math.round(offset + (x + s.x + s.w) * scale) - x0;
        const sh = Math.round(s.h * scale);
        return { left: sx0, top: h - sh, width: sx1 - sx0, height: sh };
      });
      blocks.push({ block: b, rect: { left: x0, top: baseline - h, width: x1 - x0, height: h }, screens });
      x += b.w + gap;
    }
    edges.push({
      left: Math.round(offset) - EDGE_OUT,
      top: baseline + EDGE_DROP,
      width: Math.round(span) + EDGE_OUT * 2,
      height: 2
    });
    top = baseline + LABEL_H + ROW_GAP;
  }
  return { scale, width, height: top - ROW_GAP, blocks, edges };
};
