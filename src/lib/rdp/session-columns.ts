import type { RdpSession } from '$lib/api';

export interface SessionColumn {
  key: 'user' | 'id' | 'state' | 'name' | 'act';
  label: string;
  value?: (s: RdpSession) => string;
}

export const sessionColumns: SessionColumn[] = [
  { key: 'user', label: 'User', value: (s) => s.user || '-' },
  { key: 'id', label: 'Id', value: (s) => s.id },
  { key: 'state', label: 'State', value: (s) => s.state },
  { key: 'name', label: 'Session', value: (s) => s.name },
  { key: 'act', label: 'Actions' }
];
