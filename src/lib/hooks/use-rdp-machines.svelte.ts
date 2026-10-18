import type { Config } from "$lib/api";
import { summaryChips } from "$lib/rdp/summary-chips";

export const useRdpMachines = (getConfig: () => Config) => {
  const machines = $derived(
    getConfig().machines.filter((m) => m.role === "rdp"),
  );

  // Each card reports its state here so the header chips can add them up.
  let states = $state<Record<string, string>>({});
  const report = (id: string, state: string) => {
    if (states[id] !== state) states[id] = state;
  };

  const chips = $derived(
    summaryChips(machines.map((m) => states[m.id] ?? "unknown")),
  );

  return {
    get machines() {
      return machines;
    },
    get chips() {
      return chips;
    },
    report,
  };
};
