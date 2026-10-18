<script lang="ts">
  import type { Config } from '$lib/api';
  import { useHotkeys } from '$lib/hooks/use-hotkeys.svelte';
  import { HOTKEY_HINT, type Option } from './hotkey-data';
  import Block from '$lib/ui/Block.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Cluster from '$lib/ui/Cluster.svelte';
  import ConfirmInline from '$lib/ui/ConfirmInline.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';
  import Select from '$lib/ui/Select.svelte';
  import Stack from '$lib/ui/Stack.svelte';

  interface Props {
    config: Config;
    /** Called after a successful save, so the hotkey chips reload. */
    onsaved?: () => void;
  }
  let { config, onsaved }: Props = $props();

  const ed = useHotkeys(() => config, () => onsaved?.());
</script>

{#snippet pick(label: string, value: string, options: Option[], onchange: (v: string) => void)}
  <span class="pick"><Select {label} {value} {options} {onchange} /></span>
{/snippet}

<Block label="Hotkeys editor" bleed>
  {#snippet header()}
    <Cluster gap={5}>
      <SectionLabel text="Hotkeys" count={ed.loaded ? ed.rows.length : undefined} />
      <Button label="Add key" onclick={ed.add} disabled={!ed.loaded || ed.saving || !ed.canAdd} />
      {#if ed.loaded && ed.rows.length === 0}
        <Button label="Use suggested" onclick={ed.useSuggested} disabled={ed.saving} />
      {/if}
    </Cluster>
  {/snippet}
  {#snippet actions()}
    <Button
      label="Save hotkeys"
      variant="primary"
      busy={ed.saving}
      disabled={!ed.loaded || ed.saving || !ed.dirty}
      onclick={ed.askSave}
    />
  {/snippet}

  {#if !ed.loaded}
    <p class="note">loading hotkeys...</p>
  {:else if ed.loadError}
    <p class="note err" role="status">{ed.loadError}</p>
  {:else if ed.rows.length === 0}
    <p class="note">No hotkeys yet.</p>
  {/if}

  {#each ed.rows as row (row.id)}
    <!-- Remove stays top-right on every row; only the pickers wrap (ragged rows at 420 before). -->
    <Stack direction="row" align="end" justify="between" gap={4}>
      <span class="fields"><Cluster gap={4} align="end">
        {@render pick('Key', row.key, ed.keyOptions(row), (v) => ed.setKey(row, v))}
        <span class="arrow" aria-hidden="true">&rarr;</span>
        {@render pick('Screen', row.screen, ed.screenOptions, (v) => ed.setScreen(row, v))}
        {#if ed.isServer(row.screen)}
          {@render pick('Monitor', String(row.monitor ?? ''), ed.monitorOptions, (v) =>
            ed.setMonitor(row, v)
          )}
        {/if}
      </Cluster></span>
      <Button label="Remove" variant="danger" onclick={() => ed.remove(row.id)} disabled={ed.saving} />
    </Stack>
  {/each}

  {#if ed.problems.length > 0}
    <ul class="problems" role="status">
      {#each ed.problems as line, i (i)}
        <li>{line}</li>
      {/each}
    </ul>
  {/if}

  {#if ed.result}
    <p class="result" class:err={!ed.result.ok} role="status">{ed.result.text}</p>
  {/if}

  {#snippet footer()}
    {#if ed.confirming}
      <ConfirmInline
        text="Save hotkeys? This restarts the Deskflow server."
        onyes={ed.save}
        onno={ed.cancelSave}
      />
    {:else}
      <p class="hint">{HOTKEY_HINT}</p>
    {/if}
  {/snippet}
</Block>

<style>
  .fields { flex: 1 1 0; min-width: 0; }
  .pick { flex: 0 1 auto; min-width: 0; }
  .arrow { color: var(--ink-dim); padding-bottom: var(--space-1); }
  p { margin: 0; font-size: var(--fs-small); overflow-wrap: anywhere; }
  .note, .hint { color: var(--ink-dim); }
  /* results and warnings are a line of text with a 2px left bar in the state colour */
  .hint,
  .problems,
  .result {
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    border-left: var(--bar-w) solid var(--warn);
    animation: enter var(--motion-enter);
  }
  .problems {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
    list-style: none;
    font-size: var(--fs-small);
    color: var(--warn);
    overflow-wrap: anywhere;
  }
  .result { border-left-color: var(--ok); color: var(--ink); white-space: pre-wrap; max-width: 70ch; }
  .err { color: var(--bad); }
  .result.err { border-left-color: var(--bad); }
  .hint { animation: none; }
  @keyframes enter {
    from { opacity: 0; transform: translateY(var(--space-2)); }
    to { opacity: 1; transform: none; }
  }
  @media (prefers-reduced-motion: reduce) {
    .problems,
    .result { animation: none; }
  }
</style>
