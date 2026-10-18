<script lang="ts">
  import type { Config, Machine, Role } from '$lib/api';
  import { useMachineForm } from '$lib/hooks/form-hooks/use-machine-form.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Block from '$lib/ui/Block.svelte';
  import ConfirmInline from '$lib/ui/ConfirmInline.svelte';
  import Input from '$lib/ui/Input.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';
  import Select from '$lib/ui/Select.svelte';
  import { toOptions } from '$lib/ui/select-options';
  import { OPTION_LABEL } from './machine-fields';

  interface Props {
    config: Config;
    machine: Machine | null;
    role: Role;
    onsaved: (c: Config) => void;
    oncancel: () => void;
  }

  let { config, machine, role, onsaved, oncancel }: Props = $props();

  const form = useMachineForm(() => config, () => machine, () => role, (c) => onsaved(c));
</script>

<!-- Save is a primary LINK, not the ring: the ring is the one machine action (Log in / Wake /
     Connect) and stays on screen under this panel; a second ring would split that meaning. -->
<form onsubmit={form.submit}>
  <Block variant="raised" label={form.editing ? 'Edit machine' : 'Add machine'}>
    {#snippet header()}
      <div class="title">
        <SectionLabel text={form.editing ? 'Edit machine' : 'Add machine'} />
        {#if form.editing && form.machineName}<p class="name">{form.machineName}</p>{/if}
      </div>
    {/snippet}

    <div class="fields">
      {#each form.visibleFields as f (f.key)}
        {@const hint = form.hintFor(f)}
        <label class="field">
          <span class="kicker">{f.label}</span>
          {#if f.type === 'select'}
            <Select
              value={form.valueOf(f.key)}
              options={toOptions(form.optionsFor(f), OPTION_LABEL)}
              onchange={(v) => form.setValue(f.key, v)}
              disabled={form.isReadonly(f.key)}
            />
          {:else}
            <Input
              value={form.valueOf(f.key)}
              oninput={(v) => form.setValue(f.key, v)}
              readonly={form.isReadonly(f.key)}
            />
          {/if}
          {#if hint}<small class="hint">{hint}</small>{/if}
        </label>
      {/each}
    </div>

    {#if form.error}<p class="err" role="alert">{form.error}</p>{/if}

    {#snippet footer()}
      <Button label="Save" variant="primary" onclick={form.save} busy={form.saving} disabled={form.saving} />
      <Button label="Cancel" variant="ghost" onclick={oncancel} disabled={form.saving} />
      {#if form.canDelete}
        <span class="spacer"></span>
        {#if form.confirmingDelete}
          <ConfirmInline
            text={`Delete ${form.machineName}?`}
            onyes={form.remove}
            onno={form.cancelDelete}
            busy={form.saving}
          />
        {:else}
          <Button label="Delete" variant="danger" onclick={form.askDelete} disabled={form.saving} />
        {/if}
      {/if}
    {/snippet}
  </Block>
</form>

<style>
  .title {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .name {
    margin: var(--space-0);
    font-family: var(--font-display);
    font-size: var(--fs-h1);
    font-weight: 800;
    line-height: 1;
    text-transform: uppercase;
    overflow-wrap: anywhere;
  }
  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 200px), 1fr));
    gap: var(--space-5) var(--space-5);
  }
  /* label = kicker above an underline control (same voice as the section kickers) */
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .kicker {
    font-size: var(--fs-kicker);
    font-weight: 600;
    letter-spacing: var(--ls-kicker);
    text-transform: uppercase;
    color: var(--ink-dim);
  }
  .hint {
    font-size: var(--fs-xs);
    color: var(--ink-dim);
    overflow-wrap: anywhere;
  }
  .err {
    margin: var(--space-0);
    padding-left: var(--space-3);
    border-left: var(--bar-w) solid var(--bad);
    color: var(--bad);
    overflow-wrap: anywhere;
  }
  .spacer {
    flex: 1;
  }
</style>
