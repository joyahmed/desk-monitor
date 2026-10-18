<script lang="ts">
  import type { Snippet } from 'svelte';
  import { space, type SpaceStep } from './layout';

  interface Props {
    /** Gap between cells, a --space-* step. Default 3 (12px). */
    gap?: SpaceStep;
    /** When every child goes full width: 'sm' below 640 (default), 'xs' at <=420, 'never'. */
    stack?: 'sm' | 'xs' | 'never';
    /** Element to render. Default 'div'. */
    as?: 'div' | 'section' | 'main' | 'ul' | 'ol';
    label?: string;
    children: Snippet;
  }
  let { gap = 3, stack = 'sm', as = 'div', label, children }: Props = $props();
</script>

<svelte:element this={as} class="grid stack-{stack}" style:--grid-gap={space(gap)} aria-label={label}>
  {@render children()}
</svelte:element>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(12, minmax(0, 1fr));
    gap: var(--grid-gap);
    min-width: 0;
    margin: var(--space-0);
    padding: var(--space-0);
    list-style: none;
  }
  /* zero specificity: a plain child is full width; a Block's span overrides it */
  :where(.grid) > :global(*) {
    grid-column: 1 / -1;
    min-width: 0;
  }
  @media (max-width: 639.98px) {
    .grid.stack-sm > :global(*) {
      grid-column: 1 / -1;
    }
  }
  @media (max-width: 420px) {
    .grid.stack-xs > :global(*) {
      grid-column: 1 / -1;
    }
  }
</style>
