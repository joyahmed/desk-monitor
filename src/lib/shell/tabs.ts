export type TabKey = 'monitors' | 'rdp';

/** Rail top items. `rail` is the tiny label under the icon; `label` is the full name. */
export const tabs: { key: TabKey; label: string; rail: string; icon: string }[] = [
  { key: 'monitors', label: 'Monitors', rail: 'Monitors', icon: 'monitors' },
  { key: 'rdp', label: 'Remote Desktop', rail: 'Remote', icon: 'rdp' }
];
