<script lang="ts">
  import type { ChipView } from '../desk-map-view';

  interface Props {
    chip: ChipView;
  }
  let { chip }: Props = $props();
</script>

<!-- "WIN 1  HomeServer": the key in the display font, where it goes in muted body text. -->
<span class="hk" title={chip.title}>
  <span class="key">{chip.keyText ?? chip.label}</span>
  {#if chip.target}<span class="target">{chip.target}</span>{/if}
</span>

<style>
  .hk {
    display: inline-flex;
    align-items: baseline;
    gap: var(--space-2);
    max-width: 100%;
    white-space: nowrap;
  }
  .key {
    flex: none;
    font-family: var(--font-display);
    font-weight: 800;
    font-size: var(--fs-hotkey);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--ink);
  }
  .target {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: var(--fs-small);
    color: var(--ink-dim);
  }
</style>
