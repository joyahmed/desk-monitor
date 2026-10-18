import { untrack } from "svelte";
import {
  rdpSessions,
  rdpLogoff,
  type Machine,
  type RdpSession,
} from "$lib/api";

export const useSessions = (getMachine: () => Machine) => {
  let sessions = $state<RdpSession[]>([]);
  let loading = $state(true);
  let error = $state("");
  let note = $state("");
  let confirmId = $state<string | null>(null);
  let busyId = $state<string | null>(null);
  let seq = 0;

  const load = async () => {
    const n = ++seq;
    loading = true;
    error = "";
    try {
      const r = await rdpSessions(getMachine().id);
      if (n === seq) sessions = r;
    } catch (e) {
      if (n === seq) {
        error = String(e);
        sessions = [];
      }
    } finally {
      if (n === seq) loading = false;
    }
  };

  const logoff = async (s: RdpSession) => {
    busyId = s.id;
    note = "";
    let failed = "";
    try {
      note = await rdpLogoff(getMachine().id, s.id);
    } catch (e) {
      failed = String(e);
    } finally {
      busyId = null;
      confirmId = null;
    }
    await load();
    // set after the reload so the reload cannot wipe the logoff error
    if (failed) error = failed;
  };

  const ask = (id: string | null) => {
    confirmId = id;
  };

  $effect(() => {
    getMachine().id;
    untrack(load);
  });

  return {
    get sessions() {
      return sessions;
    },
    get loading() {
      return loading;
    },
    get error() {
      return error;
    },
    get note() {
      return note;
    },
    get confirmId() {
      return confirmId;
    },
    get busyId() {
      return busyId;
    },
    ask,
    load,
    logoff,
  };
};
