export type Theme = 'light' | 'dark';
const THEME_KEY = 'desk-monitor-theme';

class ThemeState {
  theme = $state<Theme>('dark');

  private apply = (t: Theme) => {
    this.theme = t;
    document.documentElement.setAttribute('data-theme', t);
  };

  /** Call once on mount: restore the saved theme; dark is the default. */
  init = () => {
    let saved: string | null = null;
    try {
      saved = localStorage.getItem(THEME_KEY);
    } catch {
      /* ignore */
    }
    this.apply(saved === 'light' ? 'light' : 'dark');
  };

  toggle = () => {
    const next: Theme = this.theme === 'dark' ? 'light' : 'dark';
    this.apply(next);
    try {
      localStorage.setItem(THEME_KEY, next);
    } catch {
      /* storage unavailable: theme still applies for this session */
    }
  };
}

export const theme = new ThemeState();
