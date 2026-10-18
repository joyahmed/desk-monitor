<script lang="ts">
  import type { ChipView } from './desk-map-view';
  import Button from '$lib/ui/Button.svelte';
  import Cluster from '$lib/ui/Cluster.svelte';
  import HotkeyChip from './desk-map-parts/HotkeyChip.svelte';

  interface Props {
    chips: ChipView[];
    open: boolean;
    ontoggle: () => void;
  }
  let { chips, open, ontoggle }: Props = $props();
</script>

<!-- The footer line: every saved hotkey, then the Edit keys link. -->
<section class="keys" aria-label="Hotkeys">
  <Cluster gap={5}>
    {#each chips as c (c.key)}<HotkeyChip chip={c} />{/each}
    {#if chips.length === 0}<span class="dim">No hotkeys saved</span>{/if}
  </Cluster>
  <Button label={open ? 'Close keys' : 'Edit keys'} size="sm" onclick={ontoggle} />
</section>

<style>
  .keys {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3) var(--space-5);
    min-width: 0;
    padding: var(--space-4) var(--gutter);
    border-top: var(--border);
  }
  .dim { color: var(--ink-dim); font-size: var(--fs-small); }
</style>
