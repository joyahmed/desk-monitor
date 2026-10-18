<script lang="ts">
  import type { LayoutHook } from '$lib/hooks/use-desk-map.svelte';
  import Button from '$lib/ui/Button.svelte';
  import ConfirmInline from '$lib/ui/ConfirmInline.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';

  interface Props {
    ed: LayoutHook;
  }
  let { ed }: Props = $props();
</script>

<section class="bar" aria-label="Layout editor">
  <SectionLabel text="Arrange the desk" />
  {#if ed.confirming}
    <ConfirmInline
      text="Save layout? This restarts the Deskflow server."
      onyes={ed.save}
      onno={ed.cancelSave}
    />
  {:else}
    <div class="row">
      <p class="hint">
        Drag a machine, or focus it and use the arrow keys. Saving rewrites server.conf and restarts
        the Deskflow server; clients reconnect in a few seconds.
      </p>
      <Button
        label="Save layout"
        variant="primary"
        busy={ed.saving}
        disabled={!ed.loaded || ed.saving || !ed.dirty || ed.order.length === 0}
        onclick={ed.askSave}
      />
    </div>
  {/if}
  {#if ed.result}
    <p class="result" class:err={!ed.result.ok} role="status">{ed.result.text}</p>
  {/if}
</section>

<style>
  .bar {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-5) var(--gutter);
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3) var(--space-5);
  }
  p { margin: 0; overflow-wrap: anywhere; }
  .hint {
    flex: 1 1 calc(var(--space-8) * 4);
    max-width: 60ch;
    font-size: var(--fs-body);
    color: var(--ink);
  }
  .result {
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    border-left: var(--bar-w) solid var(--ok);
    white-space: pre-wrap;
    font-size: var(--fs-small);
  }
  .err { border-left-color: var(--bad); color: var(--bad); }
</style>
