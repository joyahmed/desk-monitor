import {
  appQuit,
  autostartGet,
  autostartSet,
  getConfig,
  type Config,
  type Machine,
  type Role
} from '$lib/api';
import type { TabKey } from '$lib/shell/tabs';

export type Editing = { machine: Machine | null; role: Role };

const AUTO_MS = 15000;

class AppState {
  tab = $state<TabKey>('monitors');
  config = $state<Config | null>(null);
  loadError = $state<string | null>(null);
  tick = $state(0);
  auto = $state(false);
  editing = $state<Editing | null>(null);
  autostart = $state(false);
  autostartBusy = $state(false);
  actionError = $state<string | null>(null);

  private loadSeq = 0;
  private timer: ReturnType<typeof setInterval> | null = null;

  load = async () => {
    const seq = ++this.loadSeq;
    try {
      const c = await getConfig();
      if (seq !== this.loadSeq) return;
      this.config = c;
      this.loadError = null;
    } catch (e) {
      if (seq !== this.loadSeq) return;
      this.loadError = String(e);
    }
  };

  /** Call once on mount: config and the Start-with-Windows flag. */
  init = async () => {
    void this.load();
    try {
      this.autostart = await autostartGet();
    } catch (e) {
      this.actionError = `Could not read "Start with Windows": ${String(e)}`;
    }
  };

  refresh = () => {
    this.tick++;
  };

  toggleAuto = () => {
    this.auto = !this.auto;
    if (this.timer) clearInterval(this.timer);
    this.timer = this.auto ? setInterval(this.refresh, AUTO_MS) : null;
  };

  toggleAutostart = async () => {
    if (this.autostartBusy) return;
    this.autostartBusy = true;
    const want = !this.autostart;
    try {
      await autostartSet(want);
      this.autostart = want;
      this.actionError = null;
    } catch (e) {
      this.actionError = `Could not change "Start with Windows": ${String(e)}`;
    } finally {
      this.autostartBusy = false;
    }
  };

  quit = async () => {
    try {
      await appQuit();
    } catch (e) {
      this.actionError = `Could not quit: ${String(e)}`;
    }
  };

  dismissError = () => {
    this.actionError = null;
  };

  setTab = (t: TabKey) => {
    this.tab = t;
  };
  onedit = (machine: Machine | null, role: Role) => {
    this.editing = { machine, role };
  };
  onsaved = (c: Config) => {
    this.loadSeq++; // drop any load still in flight
    this.config = c;
    this.editing = null;
    this.tick++;
  };
  oncancel = () => {
    this.editing = null;
  };
}

export const app = new AppState();
