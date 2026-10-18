/** Maps a machine state word to its token name. Token vars: --<name>, glow: --glow-<name>. */
const colours: Record<string, string> = {
  connected: 'ok',
  online: 'ok',
  hung: 'bad',
  error: 'bad',
  disconnected: 'warn',
  asleep: 'info',
  offline: 'off',
  unknown: 'off'
};

export const stateColour = (state: string): string => colours[state] ?? 'off';
