<script lang="ts">
  import type { Machine } from '$lib/api';
  import { useSessions } from '$lib/hooks/use-sessions.svelte';
  import SectionLabel from '$lib/ui/SectionLabel.svelte';
  import Button from '$lib/ui/Button.svelte';
  import ConfirmInline from '$lib/ui/ConfirmInline.svelte';

  import { sessionColumns } from './session-columns';

  interface Props {
    machine: Machine;
  }

  let { machine }: Props = $props();

  const s = useSessions(() => machine);
</script>

<!-- Sessions: a sub-band under the PC - kicker + Reload, then a ruled table, Log off as a danger link. -->
<section class="sessions" aria-label="Sessions on {machine.name}">
  <header class="head">
    <SectionLabel text="Sessions" count={s.sessions.length} />
    <Button label="Reload" size="sm" busy={s.loading} disabled={s.loading} onclick={s.load} />
  </header>

  {#if s.loading}
    <p class="msg">Loading sessions...</p>
  {:else if s.error}
    <p class="msg result err" role="alert">{s.error}</p>
  {:else if s.sessions.length === 0}
    <p class="msg">No sessions.</p>
  {/if}
  {#if s.note}<p class="msg result" role="status">{s.note}</p>{/if}

  {#if s.sessions.length > 0}
    <table>
      <thead>
        <tr>
          {#each sessionColumns as col (col.key)}
            <th class={col.key}>{#if col.key === 'act'}<span class="sr">{col.label}</span>{:else}{col.label}{/if}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each s.sessions as row (row.id + row.name)}
          <tr class:asking={s.confirmId === row.id}>
            {#each sessionColumns as col (col.key)}
              <td class={col.key}>
                {#if col.value}
                  {col.value(row)}
                {:else if s.confirmId === row.id}
                  <ConfirmInline
                    text="Log off {row.user || row.id}?"
                    busy={s.busyId === row.id}
                    size="sm"
                    onyes={() => s.logoff(row)}
                    onno={() => s.ask(null)}
                  />
                {:else}
                  <Button label="Log off" variant="danger" size="sm" onclick={() => s.ask(row.id)} />
                {/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .sessions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-top: var(--space-1);
    padding-top: var(--space-3);
    border-top: var(--border);
    min-width: 0;
    font-size: var(--fs-small);
    container-type: inline-size;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
  }
  .msg {
    margin: var(--space-0);
    color: var(--ink-dim);
    overflow-wrap: anywhere;
  }
  /* result line: 2px bar in the state colour */
  .result {
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    border-left: var(--bar-w) solid var(--ok);
    color: var(--ink);
  }
  .result.err {
    border-left-color: var(--bad);
    color: var(--bad);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
  }
  th,
  td {
    text-align: left;
    padding: var(--space-2) var(--space-3) var(--space-2) var(--space-0);
    border-bottom: var(--border);
    vertical-align: middle;
    overflow-wrap: anywhere;
  }
  th {
    padding-top: var(--space-0);
    font-size: var(--fs-kicker);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: var(--ls-kicker);
    color: var(--ink-dim);
  }
  td.user {
    font-weight: 500;
  }
  td.id,
  td.state,
  td.name {
    color: var(--ink-dim);
  }
  .act {
    padding-right: var(--space-0);
    text-align: right;
  }
  td.act :global(.confirm) {
    justify-content: flex-end;
  }
  tr.asking td {
    border-bottom-color: color-mix(in srgb, var(--bad) 45%, transparent);
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  /* narrow band: each session is two lines - "user #id state" then "name  Log off" */
  @container (max-width: 459.98px) {
    thead {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip-path: inset(50%);
    }
    tbody tr {
      display: grid;
      grid-template-columns: auto auto 1fr;
      column-gap: var(--space-3);
      row-gap: var(--space-1);
      align-items: center;
      padding: var(--space-2) var(--space-0);
      border-bottom: var(--border);
    }
    tbody tr.asking {
      border-bottom-color: color-mix(in srgb, var(--bad) 45%, transparent);
    }
    td {
      border: 0;
      padding: var(--space-0);
    }
    .id::before {
      content: '#';
    }
    .name {
      grid-column: 1 / 3;
    }
    .act {
      grid-column: 3;
      grid-row: 2;
      justify-self: end;
      min-width: 0;
    }
    /* the confirm needs the whole second line */
    tr.asking .name {
      display: none;
    }
    tr.asking .act {
      grid-column: 1 / -1;
      justify-self: stretch;
    }
  }
</style>
