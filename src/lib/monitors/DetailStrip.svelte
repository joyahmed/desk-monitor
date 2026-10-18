<script lang="ts">
  import type { DetailView } from '$lib/hooks/use-desk-map.svelte';
  import ToneDot from './desk-map-parts/ToneDot.svelte';
  import Headline from './desk-map-parts/Headline.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Cluster from '$lib/ui/Cluster.svelte';
  import ConfirmInline from '$lib/ui/ConfirmInline.svelte';
  import PasswordInline from '$lib/ui/PasswordInline.svelte';

  interface Props {
    detail: DetailView;
  }
  let { detail }: Props = $props();
</script>

<!-- The focused machine: its name as the headline, one sentence, the ring and the links. -->
<section class="detail" aria-label="{detail.name} details">
  <Headline text={detail.name} title={detail.techName || undefined} />

  <p class="meta">
    <span class="st tone-{detail.tone}">
      <ToneDot tone={detail.tone} label={detail.word} />
      <span aria-live="polite">{detail.word}</span>
    </span>
    {#each detail.meta as t, i (i)}<span>{t}</span>{/each}
  </p>

  {#if detail.reason}<p class="reason">{detail.reason}</p>{/if}

  {#if detail.result}
    <p class="result" class:err={!detail.result.ok} role="status">{detail.result.text}</p>
  {/if}

  {#if detail.confirm}
    <ConfirmInline text={detail.confirm.text} onyes={detail.confirm.yes} onno={detail.confirm.no} />
  {:else if detail.password}
    <PasswordInline
      label={detail.password.label}
      value={detail.password.value}
      oninput={detail.password.oninput}
      onyes={detail.password.yes}
      onno={detail.password.no}
      pulse={detail.pulse}
    />
  {:else}
    <div class="acts">
      {#if detail.ring}
        <Button
          label={detail.ring.label}
          variant="ring"
          pulse={detail.pulse}
          busy={detail.ring.busy}
          disabled={detail.busy}
          onclick={detail.ring.run}
        />
      {/if}
      <Cluster gap={5}>
        {#each detail.links as a (a.key)}
          <Button label={a.label} variant={a.variant} busy={a.busy} disabled={detail.busy} onclick={a.run} />
        {/each}
        {#if detail.edit}
          <Button label="Edit machine" onclick={detail.edit} disabled={detail.busy} />
        {:else}
          <Button label="Add monitor" variant="primary" onclick={detail.add} />
        {/if}
      </Cluster>
    </div>
  {/if}
</section>

<style>
  .detail {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
    padding: var(--space-6) var(--gutter) var(--space-5);
    animation: enter var(--motion-enter);
  }
  @keyframes enter {
    from { opacity: 0; transform: translateY(var(--space-2)); }
    to { opacity: 1; transform: none; }
  }
  p { margin: 0; overflow-wrap: anywhere; }
  .meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1) var(--space-4);
    font-size: var(--fs-small);
    color: var(--ink-dim);
    font-variant-numeric: tabular-nums;
  }
  .st { display: inline-flex; align-items: center; gap: var(--space-2); font-weight: 600; }
  .tone-ok { color: var(--ok); }
  .tone-warn { color: var(--warn); }
  .tone-bad { color: var(--bad); }
  .tone-info { color: var(--info); }
  .tone-off { color: var(--ink-dim); }
  .reason { max-width: 60ch; font-size: var(--fs-body); line-height: 1.5; color: var(--ink); }
  .result {
    max-width: 70ch;
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    border-left: var(--bar-w) solid var(--ok);
    white-space: pre-wrap;
    font-size: var(--fs-small);
    color: var(--ink);
    animation: enter var(--motion-enter);
  }
  .result.err { border-left-color: var(--bad); color: var(--bad); }
  .acts {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-4) var(--space-6);
    padding-top: var(--space-2);
  }
  @media (prefers-reduced-motion: reduce) {
    .detail,
    .result { animation: none; }
  }
</style>
