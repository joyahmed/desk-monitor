<script lang="ts">
  import type { Snippet } from 'svelte';
  import Stack from './Stack.svelte';
  import type { SpaceStep } from './layout';

  /** A wrapping row of small things (chips, buttons): Stack row + wrap + align center. */
  interface Props {
    /** A --space-* step. Default 2 (8px). */
    gap?: SpaceStep;
    /** Default 'start'. */
    justify?: 'start' | 'center' | 'end' | 'between';
    /** Default 'center'. */
    align?: 'start' | 'center' | 'end' | 'baseline';
    /** Element to render. Default 'div'. */
    as?: 'div' | 'ul' | 'span' | 'nav' | 'header' | 'footer';
    label?: string;
    children: Snippet;
  }
  let { gap = 2, justify = 'start', align = 'center', as = 'div', label, children }: Props = $props();
</script>

<Stack direction="row" wrap {gap} {justify} {align} {as} {label}>
  {@render children()}
</Stack>
