import { deskflowStatus, type ClientStatus, type Machine } from "$lib/api";

export const useMonitorStatus = (
  getClients: () => Machine[],
  getTick: () => number,
) => {
  let statuses = $state<Record<string, ClientStatus>>({});
  let checking = $state<Record<string, boolean>>({});
  const seq: Record<string, number> = {};

  const refreshOne = async (m: Machine) => {
    const n = (seq[m.id] = (seq[m.id] ?? 0) + 1);
    checking[m.id] = true;
    try {
      const s = await deskflowStatus(m.id);
      if (seq[m.id] === n) statuses[m.id] = s;
    } catch (e) {
      if (seq[m.id] === n)
        statuses[m.id] = {
          id: m.id,
          state: "unknown",
          reason: String(e),
          checked_at: new Date().toISOString(),
        };
    } finally {
      if (seq[m.id] === n) checking[m.id] = false;
    }
  };

  // Runs on mount, on every tick change, and when the client list changes.
  $effect(() => {
    getTick();
    getClients().forEach((m) => void refreshOne(m));
  });

  return {
    get statuses() {
      return statuses;
    },
    get checking() {
      return checking;
    },
    refreshOne,
  };
};
