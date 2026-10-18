<script lang="ts">
  import Icon from '$lib/ui/Icon.svelte';
  import { tabs, type TabKey } from './tabs';
  import type { HeaderControl } from './header-controls';

  interface Props {
    tab: TabKey;
    controls: HeaderControl[];
    onTab: (t: TabKey) => void;
  }
  let { tab, controls, onTab }: Props = $props();
</script>

<!-- The dark edge of the desk: logo, tabs (the active one carries a thin sky light line),
     controls at the bottom. Below 640 it is a bottom bar with the line on top. -->
<nav class="rail" aria-label="desk-monitor">
  <div class="group top">
    <img class="logo" src="/logo.svg" alt="" width="34" height="25" aria-hidden="true" />
    {#each tabs as t (t.key)}
      <button
        class="item tab"
        class:active={tab === t.key}
        aria-current={tab === t.key ? 'page' : undefined}
        title={t.label}
        aria-label={t.label}
        onclick={() => onTab(t.key)}
      >
        <Icon name={t.icon} />
        <span class="tiny">{t.rail}</span>
      </button>
    {/each}
  </div>
  <div class="group bottom">
    {#each controls as c (c.key)}
      <button
        class="item ctl"
        class:primary={c.primary}
        class:on={c.pressed}
        aria-pressed={c.pressed}
        aria-label={c.label}
        title={c.title}
        disabled={c.disabled}
        onclick={c.onclick}
      >
        <Icon name={c.icon} />
      </button>
    {/each}
  </div>
</nav>

<style>
  /* local: --rail-line-inset = how far the bottom-bar light line stops short of the item edges */
  .rail {
    --rail-line-inset: var(--space-4);
    position: fixed;
    inset: 0 auto 0 0;
    width: var(--rail-w);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: var(--space-3) var(--space-0) var(--space-2);
    background: var(--surface);
    border-right: var(--border);
    z-index: var(--z-rail);
  }
  .logo {
    display: block;
    margin-bottom: var(--space-4);
  }
  .group {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-1);
    width: 100%;
  }
  /* reset the global filled <button>: rail items are bare icons on the dark edge */
  .item {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    width: 100%;
    min-height: var(--size-xl);
    padding: var(--space-1) var(--space-0);
    background: transparent;
    border: 0;
    border-radius: 0;
    color: var(--ink-dim);
    transition: color var(--motion-hover);
  }
  .item:hover:not(:disabled) {
    color: var(--ink);
  }
  .item:focus-visible {
    outline-offset: -2px;
  }
  .tiny {
    font-family: var(--font-body);
    font-size: var(--fs-tiny);
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
  }
  /* the light line: sits on the rail's outer edge, glows like a lit screen edge */
  .tab::before {
    content: '';
    position: absolute;
    inset: var(--space-1) auto var(--space-1) 0;
    width: var(--bar-w);
    background: var(--sky);
    box-shadow: 0 0 var(--space-2) var(--sky);
    opacity: 0;
    transition: opacity var(--motion-hover);
  }
  .tab.active {
    color: var(--ink);
  }
  .tab.active::before {
    opacity: 1;
  }
  .tab.active .tiny {
    font-weight: 600;
  }
  .ctl.primary {
    color: var(--ink);
  }
  /* pressed toggle (Auto 15s, Start with Windows): the icon takes the light, a small LED under it */
  .ctl.on {
    color: var(--sky);
  }
  .ctl.on::after {
    content: '';
    position: absolute;
    bottom: var(--space-1);
    left: 50%;
    width: var(--led-size);
    height: var(--led-size);
    margin-left: calc(var(--led-size) / -2);
    border-radius: var(--radius-pill);
    background: var(--sky);
  }
  .ctl:disabled {
    opacity: 0.4;
  }

  /* bottom bar: every item stays reachable; the light line moves to the top edge */
  @media (max-width: 639.98px) {
    .rail {
      inset: auto 0 0 0;
      width: auto;
      height: var(--rail-bar-h);
      flex-direction: row;
      align-items: stretch;
      padding: var(--space-0) var(--space-1);
      border-right: 0;
      border-top: var(--border);
    }
    .group {
      flex-direction: row;
      align-items: stretch;
      gap: var(--space-0);
      width: auto;
    }
    .logo {
      display: none;
    }
    .item {
      width: var(--size-touch);
      min-height: var(--size-touch);
    }
    .tab {
      width: calc(var(--rail-bar-h) + var(--space-4));
    }
    .tab::before {
      inset: 0 var(--rail-line-inset) auto;
      width: auto;
      height: var(--bar-w);
    }
  }
</style>
