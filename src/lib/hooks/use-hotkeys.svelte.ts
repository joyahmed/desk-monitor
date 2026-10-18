import {
  displaysList,
  hotkeysDefault,
  hotkeysGet,
  hotkeysSave,
  hotkeysStatus,
  type Config,
  type DisplayInfo,
  type Hotkey,
} from "$lib/api";
import {
  HOTKEY_KEYS,
  displayOptions,
  primaryIndex,
  type Option,
} from "$lib/monitors/hotkey-data";

export interface HotkeyRow {
  id: number;
  key: string;
  screen: string;
  monitor: number | null;
}

export const useHotkeys = (getConfig: () => Config, onsaved?: () => void) => {
  let rows = $state<HotkeyRow[]>([]);
  let savedJson = $state("[]");
  let displays = $state<DisplayInfo[]>([]);
  let problems = $state<string[]>([]);
  let loaded = $state(false);
  let loadError = $state("");
  let saving = $state(false);
  let confirming = $state(false);
  let result = $state<{ ok: boolean; text: string } | null>(null);
  let nextId = 1;

  const serverScreen = $derived.by(() => {
    const m = getConfig().machines.find((x) => x.role === "deskflow-server");
    return m ? (m.deskflow_name ?? m.name) : "";
  });
  const screens = $derived(
    getConfig()
      .machines.filter(
        (m) => m.role === "deskflow-server" || m.role === "deskflow-client",
      )
      .map((m) => m.deskflow_name ?? m.name),
  );
  const screenOptions = $derived<Option[]>(
    getConfig()
      .machines.filter(
        (m) => m.role === "deskflow-server" || m.role === "deskflow-client",
      )
      .map((m) => ({ value: m.deskflow_name ?? m.name, label: m.name })),
  );
  const monitorOptions = $derived(displayOptions(displays));

  const isServer = (screen: string) => screen !== "" && screen === serverScreen;

  const toHotkeys = (list: HotkeyRow[]): Hotkey[] =>
    list
      .slice()
      .sort((a, b) => a.key.localeCompare(b.key))
      .map((r) => ({
        key: r.key,
        screen: r.screen,
        ...(isServer(r.screen) ? { monitor: r.monitor } : {}),
      }));

  const fromHotkeys = (list: Hotkey[]): HotkeyRow[] =>
    list.map((h) => ({
      id: nextId++,
      key: h.key,
      screen: h.screen,
      monitor: isServer(h.screen)
        ? (h.monitor ?? primaryIndex(displays))
        : null,
    }));

  const dirty = $derived(JSON.stringify(toHotkeys(rows)) !== savedJson);
  const usedKeys = $derived(new Set(rows.map((r) => r.key)));
  const canAdd = $derived(
    rows.length < HOTKEY_KEYS.length && screens.length > 0,
  );

  const keyOptions = (row: HotkeyRow): Option[] =>
    HOTKEY_KEYS.filter((k) => k === row.key || !usedKeys.has(k)).map((k) => ({
      value: k,
      label: `Win + Num${k}`,
    }));

  const refreshProblems = async () => {
    try {
      problems = await hotkeysStatus();
    } catch (e) {
      problems = [String(e)];
    }
  };

  const load = async () => {
    try {
      const [list, d] = await Promise.all([hotkeysGet(), displaysList()]);
      displays = d;
      rows = fromHotkeys(list);
      savedJson = JSON.stringify(toHotkeys(rows));
      await refreshProblems();
    } catch (e) {
      loadError = String(e);
    } finally {
      loaded = true;
    }
  };
  void load();

  const touch = () => {
    result = null;
    confirming = false;
  };

  const add = () => {
    const key = HOTKEY_KEYS.find((k) => !usedKeys.has(k));
    if (!key || !canAdd) return;
    const screen = screens[0];
    rows.push({
      id: nextId++,
      key,
      screen,
      monitor: isServer(screen) ? primaryIndex(displays) : null,
    });
    touch();
  };

  const remove = (id: number) => {
    rows = rows.filter((r) => r.id !== id);
    touch();
  };

  const setKey = (row: HotkeyRow, key: string) => {
    row.key = key;
    touch();
  };

  const setScreen = (row: HotkeyRow, screen: string) => {
    row.screen = screen;
    row.monitor = isServer(screen)
      ? (row.monitor ?? primaryIndex(displays))
      : null;
    touch();
  };

  const setMonitor = (row: HotkeyRow, value: string) => {
    row.monitor = Number(value);
    touch();
  };

  // The suggestion is not on disk yet, so it counts as a change (saved stays as loaded).
  const useSuggested = async () => {
    try {
      rows = fromHotkeys(await hotkeysDefault());
      touch();
    } catch (e) {
      result = { ok: false, text: String(e) };
    }
  };

  const askSave = () => {
    if (dirty && !saving) confirming = true;
  };
  const cancelSave = () => {
    confirming = false;
  };

  const save = async () => {
    confirming = false;
    saving = true;
    result = null;
    try {
      const list = toHotkeys(rows);
      const text = await hotkeysSave(list);
      result = { ok: true, text: text || "Saved." };
      savedJson = JSON.stringify(list);
      onsaved?.();
      await refreshProblems();
    } catch (e) {
      result = { ok: false, text: String(e) };
    } finally {
      saving = false;
    }
  };

  return {
    get rows() {
      return rows;
    },
    get loaded() {
      return loaded;
    },
    get loadError() {
      return loadError;
    },
    get saving() {
      return saving;
    },
    get confirming() {
      return confirming;
    },
    get result() {
      return result;
    },
    get dirty() {
      return dirty;
    },
    get canAdd() {
      return canAdd;
    },
    get problems() {
      return problems;
    },
    get screenOptions() {
      return screenOptions;
    },
    get monitorOptions() {
      return monitorOptions;
    },
    isServer,
    keyOptions,
    add,
    remove,
    setKey,
    setScreen,
    setMonitor,
    useSuggested,
    askSave,
    cancelSave,
    save,
  };
};
