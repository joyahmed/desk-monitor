<script lang="ts">
  import type { Snippet } from 'svelte';
  import SectionLabel from './SectionLabel.svelte';
  import { spanVars, type Span } from './layout';
  import { stateColour } from './state-colour';

  interface Props {
    /** Header kicker (small caps). Ignored when `header` is given. */
    title?: string;
    /** Number shown after the title. */
    count?: number;
    /** panel (default): a band - no box, a 1px rule on top, vertical padding only.
     *  raised: a band on --surface with padding all round. plain: no chrome, no padding. */
    variant?: 'panel' | 'raised' | 'plain';
    /** Columns in a parent Grid, per breakpoint: 6 or { base: 12, md: 6, lg: 4 }. Default 12. */
    span?: Span;
    /** Machine state word (hung, online, ...): the top rule takes the state colour. */
    state?: string;
    /** aria-label when there is no title. */
    label?: string;
    /** Full-bleed band: padding-inline var(--gutter), so the rule and floor run edge to edge. */
    bleed?: boolean;
    /** Replaces the title on the left of the header row. */
    header?: Snippet;
    /** Right side of the header row (links, chips). */
    actions?: Snippet;
    children: Snippet;
    /** Bottom row under a rule (links, confirm). */
    footer?: Snippet;
  }
  let {
    title,
    count,
    variant = 'panel',
    span = 12,
    state,
    label,
    bleed = false,
    header,
    actions,
    children,
    footer
  }: Props = $props();
  const tone = $derived(state ? stateColour(state) : undefined);
</script>

<section
  class="block {variant}"
  class:lit={!!tone}
  class:bleed
  style={spanVars(span)}
  style:--block-edge={tone ? `var(--${tone})` : undefined}
  aria-label={title ? undefined : label}
>
  {#if header || title || actions}
    <header class="head">
      <div class="lead">
        {#if header}{@render header()}{:else if title}<SectionLabel text={title} {count} />{/if}
      </div>
      {#if actions}<div class="actions">{@render actions()}</div>{/if}
    </header>
  {/if}
  <div class="body">{@render children()}</div>
  {#if footer}<footer class="foot">{@render footer()}</footer>{/if}
</section>

<style>
  .block {
    grid-column: span var(--span-base);
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    transition: border-color var(--t-state) ease;
  }
  /* band: the room is one surface; sections are separated by a rule, not boxed */
  .panel {
    padding: var(--space-5) var(--space-0);
    border-top: var(--border);
  }
  .raised {
    padding: var(--space-5);
    background: var(--surface);
  }
  .plain {
    padding: var(--space-0);
  }
  .bleed {
    padding-inline: var(--gutter);
  }
  .lit {
    border-top: var(--border-w) solid var(--block-edge);
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
    min-height: var(--size-md);
    min-width: 0;
  }
  .lead {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-4);
    min-width: 0;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-5);
    margin-left: auto;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-5);
    padding-top: var(--space-3);
    border-top: var(--border);
  }
  @media (max-width: 420px) {
    .raised {
      padding: var(--space-4) var(--gutter);
    }
  }
  @media (min-width: 420px) {
    .block {
      grid-column: span var(--span-sm);
    }
  }
  @media (min-width: 640px) {
    .block {
      grid-column: span var(--span-md);
    }
  }
  @media (min-width: 880px) {
    .block {
      grid-column: span var(--span-lg);
    }
  }
</style>
