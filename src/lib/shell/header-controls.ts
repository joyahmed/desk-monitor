export type HeaderControl = {
  key: string;
  label: string;
  title: string;
  icon: string;
  pressed?: boolean;
  disabled?: boolean;
  primary?: boolean;
  onclick: () => void;
};

type Ctx = {
  ready: boolean;
  auto: boolean;
  autostart: boolean;
  autostartBusy: boolean;
  theme: 'light' | 'dark';
  refresh: () => void;
  toggleAuto: () => void;
  toggleAutostart: () => void;
  quit: () => void;
  toggleTheme: () => void;
};

/** Rail bottom items (icon only; label is the aria-label). */
export const headerControls = (c: Ctx): HeaderControl[] => [
  {
    key: 'refresh',
    label: 'Refresh',
    title: 'Refresh all now',
    icon: 'refresh',
    primary: true,
    disabled: !c.ready,
    onclick: c.refresh
  },
  {
    key: 'auto',
    label: 'Auto 15s',
    title: 'Auto refresh every 15 seconds',
    icon: 'auto',
    pressed: c.auto,
    onclick: c.toggleAuto
  },
  {
    key: 'autostart',
    label: 'Start with Windows',
    title: 'Start with Windows: open desk-monitor when you sign in',
    icon: 'boot',
    pressed: c.autostart,
    disabled: c.autostartBusy,
    onclick: c.toggleAutostart
  },
  {
    key: 'theme',
    label: c.theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme',
    title: c.theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme',
    icon: c.theme === 'dark' ? 'sun' : 'moon',
    onclick: c.toggleTheme
  },
  {
    key: 'quit',
    label: 'Quit',
    title: 'End desk-monitor completely',
    icon: 'quit',
    onclick: c.quit
  }
];
