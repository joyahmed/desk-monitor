/** Block-system types and pure helpers (DESIGN.md "Block system"). No runes, no DOM. */

/** Index into the 4px spacing scale: 0=0 1=4 2=8 3=12 4=16 5=24 6=32 7=48 8=64 (px). */
export type SpaceStep = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

/** Control size: sm 24px, md 32px (default), lg 40px tall. */
export type Size = 'sm' | 'md' | 'lg';

/** Columns of the 12-column grid. */
export type Cols = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12;

/**
 * Grid span per breakpoint, mobile-first: base 0+, sm >=420, md >=640, lg >=880.
 * A missing tier inherits the one below it; a bare number is { base: n }.
 * Inside a <Grid stack="sm"> (default) every child is full width below 640 anyway.
 */
export type Span = Cols | { base?: Cols; sm?: Cols; md?: Cols; lg?: Cols };

/** `var(--space-n)` for a step. */
export const space = (step: SpaceStep): string => `var(--space-${step})`;

/** Inline CSS vars --span-base/sm/md/lg for a Span, every tier resolved (default 12). */
export const spanVars = (span: Span = 12): string => {
  const s = typeof span === 'number' ? { base: span } : span;
  const base = s.base ?? 12;
  const sm = s.sm ?? base;
  const md = s.md ?? sm;
  const lg = s.lg ?? md;
  return `--span-base:${base};--span-sm:${sm};--span-md:${md};--span-lg:${lg}`;
};
