<script lang="ts">
  import type { Config, Machine, Role } from '$lib/api';
  import { useDeskMap } from '$lib/hooks/use-desk-map.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Chip from '$lib/ui/Chip.svelte';
  import Cluster from '$lib/ui/Cluster.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';
  import DeskMap from './DeskMap.svelte';
  import DetailStrip from './DetailStrip.svelte';
  import HotkeyChips from './HotkeyChips.svelte';
  import Headline from './desk-map-parts/Headline.svelte';
  import LayoutBar from './desk-map-parts/LayoutBar.svelte';

  interface Props {
    config: Config;
    tick: number;
    onedit: (m: Machine | null, role: Role) => void;
  }
  let { config, tick, onedit }: Props = $props();

  const map = useDeskMap(
    () => config,
    () => tick,
    (m, role) => onedit(m, role)
  );
</script>

<div class="tab">
  <!-- the room: the screens on the wall and the desk edge they stand on -->
  <section class="room" aria-label="Desk map">
    <header class="head">
      <Cluster gap={4}>
        <SectionLabel text="Your desk tonight" />
        {#each map.summary as c (c.key)}<Chip label={c.label} state={c.state} title={c.title} />{/each}
      </Cluster>
      <Cluster gap={5}>
        <Button
          label={map.editLabel}
          variant={map.editing && !map.layout.dirty ? 'primary' : 'ghost'}
          disabled={!map.canEdit}
          onclick={map.toggleEdit}
        />
        <Button label="Add monitor" onclick={map.addMonitor} />
      </Cluster>
    </header>

    <DeskMap {map} />

    {#if map.layoutNote}<p class="note" role="status">{map.layoutNote}</p>{/if}
  </section>

  {#if map.editing}
    <LayoutBar ed={map.layout} />
  {:else if map.detail}
    {#key map.detail.key}<DetailStrip detail={map.detail} />{/key}
  {:else}
    <section class="idle" aria-label="Your desk">
      <Headline text="Your desk" />
      <p class="note">{map.idleNote}</p>
    </section>
  {/if}

  <HotkeyChips chips={map.hotkeyChips} open={map.hotkeysOpen} ontoggle={map.toggleHotkeys} />
  {#if map.hotkeysOpen}
    {#await import('./HotkeysEditor.svelte')}
      <div class="skeleton" aria-busy="true"></div>
    {:then mod}
      <mod.default {config} onsaved={map.reloadHotkeys} />
    {/await}
  {/if}
</div>

<style>
  .tab { display: flex; flex-direction: column; min-width: 0; }
  /* the room: a faint floor rising to the desk, a rule where the desk ends */
  .room {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    min-width: 0;
    padding: var(--space-4) var(--gutter) var(--space-0);
    border-bottom: var(--border);
    background: linear-gradient(to bottom, transparent, var(--surface));
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-5);
    min-width: 0;
  }
  .idle {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-6) var(--gutter) var(--space-5);
  }
  .note { margin: 0; color: var(--ink-dim); font-size: var(--fs-small); overflow-wrap: anywhere; }
  .skeleton {
    height: var(--size-xl);
    margin: var(--space-0) var(--gutter);
    background: var(--surface);
  }
</style>
