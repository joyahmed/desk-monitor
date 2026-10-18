<script lang="ts">
  interface Props {
    text: string;
    count?: number;
  }
  let { text, count }: Props = $props();
</script>

<!-- Kicker: 11px caps, +0.16em, muted. Count in --ink. -->
<h2 class="label">
  {text}{#if count !== undefined}<span class="count">{count}</span>{/if}
</h2>

<style>
  .label {
    margin: var(--space-0);
    font-family: var(--font-body);
    font-size: var(--fs-kicker);
    font-weight: 600;
    line-height: 1.2;
    text-transform: uppercase;
    letter-spacing: var(--ls-kicker);
    color: var(--ink-dim);
  }
  .count {
    margin-left: var(--space-2);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0;
    color: var(--ink);
  }
</style>
