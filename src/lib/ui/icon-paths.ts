/** Rail icons (24x24, 1.5 stroke outline). Keys match rail item `icon` fields. */
export const iconPaths: Record<string, string[]> = {
  monitors: ['M3 5h18v11H3z', 'M8 20h8', 'M12 16v4'],
  rdp: ['M3 4h13v9H3z', 'M8 17h3', 'M9.5 13v4', 'M13 11h8v9h-8z'],
  refresh: ['M20 11a8 8 0 0 0-14.5-4', 'M4 4v4h4', 'M4 13a8 8 0 0 0 14.5 4', 'M20 20v-4h-4'],
  auto: ['M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z', 'M12 8v4l3 2'],
  boot: ['M12 3v8', 'M7 6.5a7 7 0 1 0 10 0'],
  sun: [
    'M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z',
    'M12 2.5v2',
    'M12 19.5v2',
    'M2.5 12h2',
    'M19.5 12h2',
    'M5.3 5.3l1.4 1.4',
    'M17.3 17.3l1.4 1.4',
    'M5.3 18.7l1.4-1.4',
    'M17.3 6.7l1.4-1.4'
  ],
  moon: ['M20 14.5A8 8 0 0 1 9.5 4 8 8 0 1 0 20 14.5z'],
  quit: ['M10 4H5v16h5', 'M15 8l4 4-4 4', 'M19 12H9']
};
