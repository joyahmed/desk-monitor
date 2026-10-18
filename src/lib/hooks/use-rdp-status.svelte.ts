import { untrack } from "svelte";
import { rdpStatus, type Machine, type RdpStatus } from "$lib/api";

export const useRdpStatus = (
  getMachine: () => Machine,
  getTick: () => number,
  onstate?: (id: string, state: string) => void,
) => {
  let status = $state<RdpStatus | null>(null);
  let checking = $state(false);
  let seq = 0;

  const refresh = async () => {
    const n = ++seq;
    const id = getMachine().id;
    checking = true;
    try {
      const r = await rdpStatus(id);
      if (n === seq) status = r;
    } catch (e) {
      if (n === seq) {
        status = {
          id,
          online: false,
          ip: null,
          reason: String(e),
          checked_at: new Date().toISOString(),
        };
      }
    } finally {
      if (n === seq) checking = false;
    }
  };

  $effect(() => {
    getTick();
    getMachine().id;
    untrack(refresh);
  });

  const state = $derived(
    checking && !status
      ? "unknown"
      : status
        ? status.online
          ? "connected"
          : "offline"
        : "unknown",
  );
  $effect(() => {
    onstate?.(getMachine().id, state);
  });
  const sentence = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
  const word = $derived(sentence(state === "unknown" && checking ? "checking" : state));
  const checkedAt = $derived(
    status
      ? new Date(status.checked_at).toLocaleTimeString([], { hour12: false })
      : "",
  );

  return {
    get status() {
      return status;
    },
    get checking() {
      return checking;
    },
    get state() {
      return state;
    },
    get word() {
      return word;
    },
    get checkedAt() {
      return checkedAt;
    },
    /** Online: the band's little screen is lit. */
    get lit() {
      return state === "connected";
    },
    refresh,
  };
};
