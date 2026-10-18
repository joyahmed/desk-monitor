<script lang="ts">
  import StatusDot from './StatusDot.svelte';
  import type { Size } from './layout';

  interface Props {
    label: string;
    /** Machine state word: shows a StatusDot before the label. */
    state?: string;
    title?: string;
    /** sm 24px (default, as before), md 32px, lg 40px. */
    size?: Size;
    /** Label in --ink instead of muted --ink-dim. Default false. */
    strong?: boolean;
  }
  let { label, state, title, size = 'sm', strong = false }: Props = $props();
</script>

<!-- A dot and a muted word: no pill, no box. -->
<span class="chip {size}" class:strong {title}>
  {#if state}<StatusDot {state} size={size === 'lg' ? 'md' : 'sm'} />{/if}
  <span class="label">{label}</span>
</span>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    color: var(--ink-dim);
    font-weight: 500;
    white-space: nowrap;
  }
  .strong {
    color: var(--ink);
  }
  .sm {
    gap: var(--size-sm-gap);
    min-height: var(--size-sm);
    font-size: var(--fs-small);
  }
  .md {
    gap: var(--size-md-gap);
    min-height: var(--size-md);
    font-size: var(--size-md-fs);
  }
  .lg {
    gap: var(--size-lg-gap);
    min-height: var(--size-lg);
    font-size: var(--size-lg-fs);
  }
  .label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: var(--font-body);
  }
</style>
