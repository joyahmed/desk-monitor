import type { DisplayInfo } from '$lib/api';

export interface Option {
  value: string;
  label: string;
}

export const HOTKEY_KEYS = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

export const HOTKEY_HINT =
  'Num keys for Windows monitors need desk-monitor running. Mac and Ubuntu keys always work.';

const POSITION_NAMES: Record<number, string[]> = {
  1: ['only'],
  2: ['left', 'right'],
  3: ['left', 'middle', 'right']
};

/** Names a monitor by its place left to right: "left / middle / right", else "#1, #2, ...". */
export const displayOptions = (displays: DisplayInfo[]): Option[] => {
  const sorted = displays.slice().sort((a, b) => a.x - b.x);
  const names = POSITION_NAMES[sorted.length];
  return sorted.map((d, i) => ({
    value: String(d.index),
    label: `${names ? names[i] : `#${i + 1}`} ${d.width}x${d.height}${d.primary ? ' (primary)' : ''}`
  }));
};

export const primaryIndex = (displays: DisplayInfo[]): number | null =>
  (displays.find((d) => d.primary) ?? displays[0])?.index ?? null;

/** "left" / "middle" / "right" (or "#n") for a display index, by its place left to right. */
export const positionName = (displays: DisplayInfo[], index: number): string => {
  const sorted = displays.slice().sort((a, b) => a.x - b.x);
  const i = sorted.findIndex((d) => d.index === index);
  if (i < 0) return `#${index + 1}`;
  const names = POSITION_NAMES[sorted.length];
  return names ? names[i] : `#${i + 1}`;
};
