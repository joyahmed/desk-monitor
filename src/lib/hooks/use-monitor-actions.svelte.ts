import { deskflowAction, loginServer, type DeskflowAction, type Machine } from "$lib/api";

export interface ActionResult {
  ok: boolean;
  text: string;
}

export const useMonitorActions = (onafter: (m: Machine) => void) => {
  let busy = $state<Record<string, DeskflowAction | null>>({});
  let results = $state<Record<string, ActionResult | null>>({});
  // machine id -> action waiting for Yes/No
  let asking = $state<Record<string, DeskflowAction | null>>({});
  // machine id -> password being typed for Log in; cleared after every try, never saved
  let secret = $state<Record<string, string>>({});

  const run = async (m: Machine, action: DeskflowAction) => {
    asking[m.id] = null;
    busy[m.id] = action;
    results[m.id] = null;
    const password = secret[m.id] ?? "";
    secret[m.id] = "";
    try {
      const text =
        action === "login_server"
          ? await loginServer(m.id, password)
          : await deskflowAction(m.id, action);
      results[m.id] = { ok: true, text: text || "Done." };
    } catch (e) {
      results[m.id] = { ok: false, text: String(e) };
    } finally {
      busy[m.id] = null;
    }
    onafter(m);
  };

  const ask = (m: Machine, action: DeskflowAction) => {
    results[m.id] = null;
    asking[m.id] = action;
  };
  const cancel = (m: Machine) => {
    asking[m.id] = null;
    secret[m.id] = "";
  };
  const setSecret = (m: Machine, value: string) => {
    secret[m.id] = value;
  };

  return {
    get busy() {
      return busy;
    },
    get results() {
      return results;
    },
    get asking() {
      return asking;
    },
    get secret() {
      return secret;
    },
    run,
    setSecret,
    ask,
    cancel,
  };
};
