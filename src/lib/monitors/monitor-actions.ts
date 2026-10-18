import type { DeskflowAction, Machine } from '$lib/api';

export interface MonitorAction {
  key: DeskflowAction;
  label: string;
  busyLabel: string;
  /** ring = the ONE round action of a machine (Log in); everything else is a text link. */
  variant: 'primary' | 'ghost' | 'danger' | 'ring';
  showFor: (m: Machine) => boolean;
  /** When set, the action asks this question in place before it runs. */
  confirm?: string;
  /** When set, the action asks for a password in place (this is its label) before it runs. */
  password?: string;
}

const isServer = (m: Machine) => m.role === 'deskflow-server';
const isClient = (m: Machine) => m.role === 'deskflow-client';
/** An optional machine setting counts only when it has text. */
const has = (v: string | undefined) => !!v?.trim();

export const MONITOR_ACTIONS: MonitorAction[] = [
  {
    key: 'restart_server',
    label: 'Restart Deskflow server',
    busyLabel: 'restarting server...',
    variant: 'danger',
    showFor: isServer,
    confirm: 'Restart the Deskflow server? All clients reconnect.'
  },
  {
    key: 'restart_client',
    label: 'Restart client',
    busyLabel: 'restarting client...',
    variant: 'ghost',
    showFor: (m) => isClient(m) && has(m.client_restart_command)
  },
  {
    key: 'login_server',
    label: 'Log in',
    busyLabel: 'logging in...',
    variant: 'ring',
    showFor: (m) => isClient(m) && has(m.login_command),
    password: 'Password for this machine'
  },
  {
    key: 'wake',
    label: 'Wake',
    busyLabel: 'starting the wake script...',
    variant: 'ring',
    showFor: (m) => isClient(m) && has(m.wake_script)
  },
  {
    key: 'set_remote_host',
    label: 'Fix server IP',
    busyLabel: 'fixing server IP...',
    variant: 'ghost',
    showFor: (m) => isClient(m) && m.os !== 'windows'
  }
];

export const checkedTime = (iso: string | undefined) => {
  if (!iso) return '';
  const d = new Date(iso);
  if (isNaN(d.getTime())) return iso;
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
};
