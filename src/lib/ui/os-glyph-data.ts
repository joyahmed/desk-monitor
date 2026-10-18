/** Outline paths (24x24 viewBox, 1.5 stroke) per OS family. */
export type OsFamily = 'windows' | 'ubuntu' | 'apple' | 'other';

export const osFamily = (os: string): OsFamily => {
  const s = os.toLowerCase();
  if (s.includes('win')) return 'windows';
  if (s.includes('ubuntu') || s.includes('linux') || s.includes('debian')) return 'ubuntu';
  if (s.includes('mac') || s.includes('apple') || s.includes('darwin') || s.includes('osx'))
    return 'apple';
  return 'other';
};

export const osPaths: Record<OsFamily, string[]> = {
  windows: ['M3.5 5.5 11 4.5V11H3.5z', 'M13 4.2 20.5 3v8H13z', 'M3.5 13H11v6.5l-7.5-1z', 'M13 13h7.5v8L13 19.8z'],
  ubuntu: [
    'M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17z',
    'M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7z',
    'M12 3.5V8.5',
    'M4.6 16.2 8.9 13.7',
    'M19.4 16.2 15.1 13.7'
  ],
  apple: [
    'M12 8.5C10.5 7.5 8.5 7.3 7 8.5c-1.8 1.5-1.5 5-.2 7.2C7.7 17.2 8.6 18.5 9.8 18.500c1 0 1.4-.5 2.2-.5s1.2.5 2.2.5c1.2 0 2.1-1.3 3-2.8-2.1-1-2.6-4.2-.3-5.5-1.1-1.5-3.2-2-4.9-1.200z',
    'M12.3 6.500c0-1.5.9-2.7 2.5-3.1'
  ],
  other: ['M4 5h16v11H4z', 'M9 20h6', 'M12 16v4']
};
