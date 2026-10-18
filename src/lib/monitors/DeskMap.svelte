<script lang="ts">
  import type { BlockView, DeskMapHook } from '$lib/hooks/use-desk-map.svelte';
  import MapScreen from './desk-map-parts/MapScreen.svelte';

  interface Props {
    map: DeskMapHook;
  }
  let { map }: Props = $props();
  const ed = $derived(map.layout);
</script>

{#snippet screens(b: BlockView)}
  {#each b.screens as s (s.key)}
    <MapScreen screen={s} interactive={!map.editing} onselect={map.select} />
  {/each}
{/snippet}

<div class="frame" class:editing={map.editing} use:map.measure>
  <div
    class="desk"
    style:width="{map.deskWidth}px"
    style:height="{map.deskHeight}px"
    role="group"
    aria-label="Desk map, left to right"
  >
    <span class="pool" class:on={map.pool.on && !map.editing} style={map.pool.style} aria-hidden="true"></span>
    {#each map.edges as e (e.key)}<span class="edge" style={e.style} aria-hidden="true"></span>{/each}
    {#each map.blocks as b (b.key)}
      {#if map.editing && b.placed}
        <div
          class="block drag"
          class:dragging={ed.dragName === b.key}
          style={b.style}
          style:transform={ed.dragName === b.key ? `translateX(${ed.dragDx}px)` : null}
          role="button"
          tabindex="0"
          title={b.title}
          aria-label={b.label}
          use:ed.tile={b.key}
          onpointerdown={(e) => ed.down(e, b.key)}
          onpointermove={ed.onmove}
          onpointerup={ed.onup}
          onpointercancel={ed.onup}
          onkeydown={(e) => ed.onkey(e, b.key)}
        >
          {@render screens(b)}
        </div>
      {:else}
        <div class="block" class:unplaced={!b.placed} style={b.style} title={b.title}>
          {@render screens(b)}
        </div>
      {/if}
    {/each}
    {#each map.blocks as b (b.key)}
      <span class="plate" class:lit={b.plate.lit} style={b.plate.style} aria-hidden="true">
        <span class="pname">{b.plate.name}</span>{#if b.plate.note}<span class="pnote tone-{b.plate.tone}">{' · '}{b.plate.note}</span>{/if}
      </span>
    {/each}
  </div>
  {#if map.editing}
    <div class="ends" aria-hidden="true"><span>&larr; Left</span><span>Right &rarr;</span></div>
  {/if}
</div>

<style>
  /* the wall: no box, the room band around it is the surface. Padding = FRAME_Y in use-desk-map. */
  .frame {
    position: relative;
    min-width: 0;
    padding: var(--space-5) var(--space-0) var(--space-6);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }
  .frame.editing {
    outline: 1px dashed var(--sky);
    outline-offset: calc(var(--space-2) * -1);
  }
  .desk { position: relative; max-width: 100%; }
  /* the desk edge each row of screens stands on */
  .edge {
    position: absolute;
    background: linear-gradient(to right, transparent, var(--bezel) 12%, var(--bezel) 88%, transparent);
  }
  /* light falling on the desk under the lit screen; slides with the light */
  .pool {
    position: absolute;
    height: var(--space-8);
    background: var(--pool), var(--pool); /* two layers: one read too faint on the dark room */
    opacity: 0;
    pointer-events: none;
    transition:
      left var(--motion-light),
      top var(--motion-light),
      width var(--motion-light),
      opacity var(--motion-light);
  }
  .pool.on { opacity: 1; }
  /* the name plate under each machine, below the desk edge */
  .plate {
    position: absolute;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: var(--fs-small);
    line-height: 1.2;
    color: var(--ink-dim);
    pointer-events: none;
  }
  .pname { font-weight: 600; color: var(--ink); transition: color var(--motion-light); }
  .plate.lit .pname { color: var(--sky); }
  .pnote.tone-warn { color: var(--warn); }
  .pnote.tone-bad { color: var(--bad); }
  .block {
    position: absolute;
    transition: left var(--t-snap) var(--ease-lift), transform var(--t-snap) var(--ease-lift);
  }
  .block.unplaced { opacity: 0.6; }
  .drag { cursor: grab; touch-action: none; user-select: none; border-radius: var(--radius-screen); }
  .drag:hover { outline: 1px dashed var(--ink-dim); outline-offset: 4px; }
  .drag:focus-visible { outline: 2px solid var(--focus); outline-offset: 4px; }
  .drag.dragging { cursor: grabbing; z-index: 2; transition: none; outline: 2px solid var(--sky); outline-offset: 4px; }
  .ends {
    align-self: stretch;
    display: flex;
    justify-content: space-between;
    padding: var(--space-0) var(--space-3);
    font-size: var(--fs-xs);
    color: var(--ink-dim);
  }
  @media (prefers-reduced-motion: reduce) {
    .block,
    .pool { transition: none; }
  }
</style>
