<script lang="ts">
  import '../app.css';
  import '$lib/tokens.css';
  import { onMount } from 'svelte';
  import { app } from '$lib/state/app.svelte';
  import { theme } from '$lib/state/theme.svelte';
  import { headerControls } from '$lib/shell/header-controls';
  import IconRail from '$lib/shell/IconRail.svelte';
  import FailureLine from '$lib/shell/FailureLine.svelte';
  import LoadingRoom from '$lib/shell/LoadingRoom.svelte';
  import MonitorsTab from '$lib/monitors/MonitorsTab.svelte';
  import RdpTab from '$lib/rdp/RdpTab.svelte';

  onMount(() => {
    theme.init();
    void app.init();
  });

  const controls = $derived(
    headerControls({
      ready: !!app.config,
      auto: app.auto,
      autostart: app.autostart,
      autostartBusy: app.autostartBusy,
      theme: theme.theme,
      refresh: app.refresh,
      toggleAuto: app.toggleAuto,
      toggleAutostart: app.toggleAutostart,
      quit: app.quit,
      toggleTheme: theme.toggle
    })
  );
</script>

<IconRail tab={app.tab} {controls} onTab={app.setTab} />

<!-- main is full-bleed (the tabs pad their own bands by --gutter); the page's own states
     (action error, load error, loading, the edit panel) sit in .pad -->
<div class="stage">
  <main>
    {#if app.actionError}
      <div class="pad">
        <FailureLine lead={null} msg={app.actionError} action="Dismiss" run={app.dismissError} />
      </div>
    {/if}
    {#if app.loadError}
      <div class="pad">
        <FailureLine lead="Could not load config." msg={app.loadError} action="Retry" run={app.load} />
      </div>
    {:else if !app.config}
      <div class="pad"><LoadingRoom /></div>
    {:else}
      {#if app.editing}
        {#key `${app.editing.machine?.id ?? 'new'}:${app.editing.role}`}
          {#await import('$lib/machines/MachinePanel.svelte') then { default: MachinePanel }}
            <div class="pad">
              <MachinePanel
                config={app.config}
                machine={app.editing.machine}
                role={app.editing.role}
                onsaved={app.onsaved}
                oncancel={app.oncancel}
              />
            </div>
          {/await}
        {/key}
      {/if}
      {#if app.tab === 'monitors'}
        <MonitorsTab config={app.config} tick={app.tick} onedit={app.onedit} onsaved={app.onsaved} />
      {:else}
        <RdpTab config={app.config} tick={app.tick} onedit={app.onedit} />
      {/if}
    {/if}
  </main>
</div>

<style>
  .stage {
    margin-left: var(--rail-w);
    min-width: 0;
  }
  main {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .pad {
    padding: var(--gutter);
    min-width: 0;
  }
  /* rail is a bottom bar below 640 (IconRail.svelte) */
  @media (max-width: 639.98px) {
    .stage {
      margin-left: var(--space-0);
      padding-bottom: var(--rail-bar-h);
    }
  }
</style>
