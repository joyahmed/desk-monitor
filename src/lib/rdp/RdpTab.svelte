<script lang="ts">
  import type { Config, Machine, Role } from '$lib/api';
  import { useRdpMachines } from '$lib/hooks/use-rdp-machines.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Chip from '$lib/ui/Chip.svelte';
  import RdpCard from './RdpCard.svelte';

  interface Props {
    config: Config;
    tick: number;
    onedit: (m: Machine | null, role: Role) => void;
  }

  let { config, tick, onedit }: Props = $props();

  const list = useRdpMachines(() => config);
</script>

<!-- The remote room: a kicker line on top, then one band per PC in a 1/2/3-column grid. -->
<section class="room" aria-label="Remote desktop">
  <header class="head">
    <div class="lead">
      <SectionLabel text="Remote desktop" count={list.machines.length} />
      {#each list.chips as c (c.key)}
        <Chip label={c.label} state={c.state} />
      {/each}
    </div>
    <Button label="Add PC" variant="primary" onclick={() => onedit(null, 'rdp')} />
  </header>

  {#if list.machines.length === 0}
    <div class="empty">
      <span class="screen" aria-hidden="true"></span>
      <p class="title">No remote PCs yet</p>
      <p class="hint">
        Add a Windows PC you reach over Remote Desktop. It shows here with its status and a Connect link.
      </p>
    </div>
  {:else}
    <ul class="bands">
      {#each list.machines as m (m.id)}
        <li><RdpCard machine={m} {tick} {onedit} onstate={list.report} /></li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .room {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-5) var(--gutter);
    min-width: 0;
    container-type: inline-size;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2) var(--space-5);
    min-height: var(--size-md);
  }
  .lead {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-4);
    min-width: 0;
  }
  /* bands: no cards, a 1px rule on top of each; columns follow the room's own width */
  .bands {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    align-items: start;
    column-gap: var(--space-6);
    row-gap: var(--space-0);
    margin: var(--space-0);
    padding: var(--space-0);
    list-style: none;
  }
  .bands > li {
    min-width: 0;
  }
  @container (min-width: 600px) {
    .bands {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @container (min-width: 1000px) {
    .bands {
      grid-template-columns: repeat(3, minmax(0, 1fr));
    }
  }

  /* empty: one dark screen waiting to be lit */
  .empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-6) var(--space-0);
    border-top: var(--border);
  }
  .screen {
    width: var(--size-xl);
    height: calc(var(--size-xl) * 0.625);
    border: var(--border-w) solid var(--bezel);
    border-radius: var(--radius-screen);
    background: var(--screen-off);
  }
  .title {
    margin: var(--space-0);
    font-family: var(--font-display);
    font-size: var(--fs-h1);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.01em;
    line-height: 1;
  }
  .hint {
    margin: var(--space-0);
    max-width: 46ch;
    color: var(--ink-dim);
    font-size: var(--fs-small);
  }
</style>
