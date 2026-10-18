<script lang="ts">
  import type { ScreenView } from '$lib/hooks/use-desk-map.svelte';

  interface Props {
    screen: ScreenView;
    /** false in edit mode: the whole block is the drag handle, screens are just drawn. */
    interactive: boolean;
    onselect: (key: string) => void;
  }
  let { screen, interactive, onselect }: Props = $props();
</script>

<!-- The face is empty on purpose: the light, the login box and the sleep LED say the state;
     the name plate under the desk edge (DeskMap) carries the words. -->
{#snippet body()}
  <span class="light" aria-hidden="true"></span>
  {#if screen.look === 'login'}<span class="loginbox" aria-hidden="true"></span>{/if}
  {#if screen.look === 'asleep'}<span class="led" aria-hidden="true"></span>{/if}
{/snippet}

{#if interactive}
  <button
    type="button"
    class={screen.cls}
    style={screen.style}
    title={screen.title}
    aria-label={screen.title}
    aria-pressed={screen.selected}
    onclick={() => onselect(screen.key)}
  >
    {@render body()}
  </button>
{:else}
  <div class={screen.cls} style={screen.style} title={screen.title}>{@render body()}</div>
{/if}

<style>
  /* A screen on the wall: dark face, 1px bezel. Only the selected one is lit. */
  .screen {
    position: absolute;
    isolation: isolate;
    display: block;
    min-width: 0;
    overflow: hidden;
    padding: 0;
    margin: 0;
    background: var(--screen-off);
    color: var(--ink-dim);
    border: var(--border-w) solid var(--bezel);
    border-radius: var(--radius-screen);
    text-align: left;
    font: inherit;
    cursor: default;
    box-shadow: none;
    transition:
      box-shadow var(--motion-light),
      color var(--motion-light),
      background-color var(--motion-light),
      border-color var(--motion-hover);
  }
  button.screen { cursor: pointer; }
  button.screen:hover { border-color: var(--sky); }
  button.screen:focus-visible { outline: 2px solid var(--focus); outline-offset: 3px; }

  /* the light: a gradient layer that crossfades in, so switching screens moves the light */
  .light {
    position: absolute;
    inset: 0;
    z-index: -1;
    background: var(--grad-light);
    opacity: 0;
    transition: opacity var(--motion-light);
  }
  .selected { color: var(--on-lit); border-color: transparent; box-shadow: var(--lit-glow); }
  .selected .light { opacity: 1; }

  /* connected: dimly lit */
  .look-dim { background-color: var(--screen-dim); }
  /* hung (or the server stopped): the bezel turns --bad */
  .look-hung { border-color: var(--bad); }
  /* offline / unknown: a dashed bezel, nothing behind it */
  .look-off { border-style: dashed; border-color: var(--off); }
  /* disconnected = behind its login screen: a small login box in the middle */
  .loginbox {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 28%;
    height: 14%;
    min-height: var(--space-2);
    transform: translate(-50%, -50%);
    border: var(--border-w) solid var(--warn);
    border-radius: var(--radius-sm);
    pointer-events: none;
  }
  .selected .loginbox { border-color: var(--on-lit); }
  /* asleep: only the tiny white LED breathes */
  .led {
    position: absolute;
    left: 50%;
    bottom: var(--space-1);
    width: var(--led-size);
    height: var(--led-size);
    transform: translateX(-50%);
    border-radius: 50%;
    background: var(--info);
    animation: breathe var(--motion-breathe) infinite;
  }
  .selected .led { background: var(--on-lit); }
  @keyframes breathe {
    0%, 100% { opacity: 0.15; box-shadow: none; }
    50% { opacity: 1; box-shadow: 0 0 8px var(--info); }
  }

  @media (prefers-reduced-motion: reduce) {
    .screen,
    .light { transition: none; }
    .led { animation: none; opacity: 1; }
  }
</style>
