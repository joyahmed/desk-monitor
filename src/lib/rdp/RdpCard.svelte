<script lang="ts">
  import type { Machine } from '$lib/api';
  import { useRdpStatus } from '$lib/hooks/use-rdp-status.svelte';
  import { useRdpActions } from '$lib/hooks/use-rdp-actions.svelte';
  import StatusDot from '$lib/ui/StatusDot.svelte';
  import Button from '$lib/ui/Button.svelte';
  import OsGlyph from '$lib/ui/OsGlyph.svelte';
  import Block from '$lib/ui/Block.svelte';

  interface Props {
    machine: Machine;
    tick: number;
    onedit: (m: Machine | null, role: 'rdp') => void;
    onstate?: (id: string, state: string) => void;
  }

  let { machine, tick, onedit, onstate }: Props = $props();

  const st = useRdpStatus(() => machine, () => tick, (id, s) => onstate?.(id, s));
  const act = useRdpActions(() => machine, (m, r) => onedit(m, r));
</script>

<!-- One PC = one band: a small screen (lit when online), the name, one meta line, links. -->
<Block variant="panel" state={st.lit ? st.state : undefined} label={machine.name}>
  {#snippet header()}
    <span class="screen" class:lit={st.lit} aria-hidden="true">
      <span class="os"><OsGlyph os={machine.os} size={14} /></span>
    </span>
    <div class="who">
      <h3 class:lit={st.lit}>{machine.name}</h3>
      {#if machine.label}<span class="sub">{machine.label}</span>{/if}
    </div>
  {/snippet}
  {#snippet actions()}
    <Button label="Edit" size="sm" onclick={act.edit} disabled={act.busy !== ''} />
  {/snippet}

  <div class="meta" aria-live="polite">
    <span class="state" class:ok={st.lit}><StatusDot state={st.state} size="sm" />{st.word}</span>
    <span class:none={!st.status?.ip}>{st.status?.ip ?? 'No IP'}</span>
    {#if st.checkedAt}<span>checked {st.checkedAt}</span>{/if}
  </div>
  {#if st.status?.reason}<p class="reason">{st.status.reason}</p>{/if}

  <div class="links">
    {#each act.shown as a (a.key)}
      <Button
        label={a.label}
        variant={a.variant}
        title={a.title}
        busy={act.isBusy(a.key)}
        disabled={act.isDisabled(a.key)}
        onclick={() => act.run(a.key)}
      />
    {/each}
  </div>

  {#if act.message}
    <p class="result" class:err={act.messageErr} role="status">{act.message}</p>
  {/if}

  {#if act.sessionsOpen}
    {#await import('./SessionsPanel.svelte') then mod}
      <mod.default {machine} />
    {/await}
  {/if}
</Block>

<style>
  /* the little screen: dark face + bezel; online fades the light in (--motion-light) */
  .screen {
    --glyph-w: var(--size-xl);
    --glyph-h: calc(var(--size-xl) * 0.625);
    position: relative;
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--glyph-w);
    height: var(--glyph-h);
    border: var(--border-w) solid var(--bezel);
    border-radius: var(--radius-screen);
    background: var(--screen-off);
    color: var(--ink-dim);
    transition:
      box-shadow var(--motion-light),
      border-color var(--motion-light);
  }
  .screen::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--grad-light);
    opacity: 0;
    transition: opacity var(--motion-light);
  }
  .screen.lit {
    border-color: transparent;
    box-shadow: var(--lit-glow);
    color: var(--on-lit);
  }
  .screen.lit::before {
    opacity: 1;
  }
  .os {
    position: relative;
    display: inline-flex;
    opacity: 0.8;
  }
  .who {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-hair);
  }
  h3 {
    margin: var(--space-0);
    font-family: var(--font-display);
    font-size: var(--fs-h1);
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.01em;
    line-height: 1;
    overflow-wrap: anywhere;
  }
  h3.lit {
    background: var(--grad-headline);
    background-clip: text;
    -webkit-background-clip: text;
    color: transparent;
  }
  .sub {
    font-size: var(--fs-xs);
    color: var(--ink-dim);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-4);
    font-size: var(--fs-small);
    font-variant-numeric: tabular-nums;
    color: var(--ink-dim);
  }
  .state {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 500;
    color: var(--ink);
  }
  .state.ok {
    color: var(--ok);
  }
  .none {
    font-style: italic;
  }
  .reason {
    margin: var(--space-0);
    font-size: var(--fs-small);
    color: var(--ink-dim);
    overflow-wrap: anywhere;
  }
  .links {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-5);
  }
  /* result: one line with a 2px bar in the state colour */
  .result {
    margin: var(--space-0);
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    border-left: var(--bar-w) solid var(--ok);
    font-size: var(--fs-small);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    animation: rise var(--motion-enter);
  }
  .result.err {
    border-left-color: var(--bad);
    color: var(--bad);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
  }
</style>
