<script lang="ts">
  import type { Snippet } from 'svelte';
  import { space, type SpaceStep } from './layout';

  interface Props {
    /** Default 'column'. */
    direction?: 'column' | 'row';
    /** A --space-* step. Default 3 (12px). */
    gap?: SpaceStep;
    /** Cross-axis alignment. Default 'stretch'. */
    align?: 'start' | 'center' | 'end' | 'stretch' | 'baseline';
    /** Main-axis distribution. Default 'start'. */
    justify?: 'start' | 'center' | 'end' | 'between';
    /** Let items wrap onto new lines. Default false. */
    wrap?: boolean;
    /** Element to render. Default 'div'. */
    as?: 'div' | 'section' | 'main' | 'header' | 'footer' | 'nav' | 'ul' | 'ol' | 'li' | 'form' | 'span';
    label?: string;
    children: Snippet;
  }
  let {
    direction = 'column',
    gap = 3,
    align = 'stretch',
    justify = 'start',
    wrap = false,
    as = 'div',
    label,
    children
  }: Props = $props();
</script>

<svelte:element
  this={as}
  class="stack {direction} a-{align} j-{justify}"
  class:wrap
  style:--stack-gap={space(gap)}
  aria-label={label}
>
  {@render children()}
</svelte:element>

<style>
  .stack {
    display: flex;
    gap: var(--stack-gap);
    min-width: 0;
    margin: var(--space-0);
    padding: var(--space-0);
    list-style: none;
  }
  .column {
    flex-direction: column;
  }
  .row {
    flex-direction: row;
  }
  .wrap {
    flex-wrap: wrap;
  }
  .a-start {
    align-items: flex-start;
  }
  .a-center {
    align-items: center;
  }
  .a-end {
    align-items: flex-end;
  }
  .a-stretch {
    align-items: stretch;
  }
  .a-baseline {
    align-items: baseline;
  }
  .j-start {
    justify-content: flex-start;
  }
  .j-center {
    justify-content: center;
  }
  .j-end {
    justify-content: flex-end;
  }
  .j-between {
    justify-content: space-between;
  }
</style>
