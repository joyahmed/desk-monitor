<script lang="ts">
  import type { DeskNameHook } from '$lib/hooks/use-desk-name.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Cluster from '$lib/ui/Cluster.svelte';
  import Input from '$lib/ui/Input.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';

  interface Props {
    desk: DeskNameHook;
  }
  let { desk }: Props = $props();
</script>

<!-- The room kicker (the desk's name) with a Rename link; Rename opens an inline form. -->
<div class="bar">
  {#if desk.renaming}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="form" onkeydown={desk.onkeydown}>
      <div class="input">
        <Input
          value={desk.draft}
          oninput={(v) => (desk.draft = v)}
          label="Desk name"
          placeholder="Your desk"
          size="sm"
          focus
        />
      </div>
      <Button label="Save" variant="primary" size="sm" busy={desk.saving} onclick={() => void desk.save()} />
      <Button label="Cancel" size="sm" onclick={desk.cancel} />
    </div>
    {#if desk.error}<p class="err" role="alert">{desk.error}</p>{/if}
  {:else}
    <Cluster gap={3}>
      <SectionLabel text={desk.name} />
      <Button label="Rename" size="sm" onclick={desk.start} />
    </Cluster>
  {/if}
</div>

<style>
  .bar { display: flex; flex-direction: column; gap: var(--space-1); min-width: 0; }
  .form { display: flex; align-items: center; gap: var(--space-3); min-width: 0; }
  .input { flex: 1 1 140px; min-width: 0; max-width: 260px; }
  .err { margin: 0; color: var(--bad); font-size: var(--fs-small); overflow-wrap: anywhere; }
</style>
