<script lang="ts">
  /** First paint before the config arrives: the room with every light off (three dark
   *  screens on the desk edge), so the layout does not jump when the desk map lands. */
  const screens = ['left', 'main', 'right'];
</script>

<section class="room" aria-busy="true">
  <p class="kicker" role="status">Loading config...</p>
  <div class="wall" aria-hidden="true">
    {#each screens as s (s)}<span class="screen"></span>{/each}
  </div>
  <span class="edge" aria-hidden="true"></span>
</section>

<style>
  .room {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    max-width: 36rem;
  }
  .kicker {
    margin: var(--space-0);
    color: var(--ink-dim);
    font-size: var(--fs-kicker);
    font-weight: 600;
    letter-spacing: var(--ls-kicker);
    text-transform: uppercase;
  }
  .wall {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: var(--space-2);
    align-items: end;
  }
  .screen {
    aspect-ratio: 16 / 10;
    background: var(--screen-off);
    border: var(--border-w) solid var(--bezel);
    border-radius: var(--radius-screen);
  }
  .edge {
    border-top: var(--border-w) solid var(--bezel);
  }
</style>
