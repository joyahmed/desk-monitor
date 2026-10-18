export interface SummaryChip {
  key: string;
  label: string;
  state: "connected" | "offline" | "unknown";
}

interface ChipRule {
  key: string;
  word: string;
  state: SummaryChip["state"];
  match: (state: string) => boolean;
}

const rules: ChipRule[] = [
  { key: "online", word: "online", state: "connected", match: (s) => s === "connected" },
  { key: "offline", word: "offline", state: "offline", match: (s) => s === "offline" },
  { key: "unknown", word: "checking", state: "unknown", match: (s) => s === "unknown" },
];

/** "2 online", "1 offline"... only the groups that are not empty. */
export const summaryChips = (states: string[]): SummaryChip[] =>
  rules
    .map((r) => ({ r, n: states.filter(r.match).length }))
    .filter(({ n }) => n > 0)
    .map(({ r, n }) => ({ key: r.key, label: `${n} ${r.word}`, state: r.state }));
