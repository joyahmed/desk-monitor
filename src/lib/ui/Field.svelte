<script lang="ts">
  import type { Snippet } from 'svelte';

  /** Label above a control (Input / Select as children), optional hint or error below. */
  interface Props {
    label: string;
    /** Small dim text under the control. */
    hint?: string;
    /** Small --bad text under the control; replaces the hint. */
    error?: string;
    children: Snippet;
  }
  let { label, hint, error, children }: Props = $props();
</script>

<label class="field">
  <span class="name">{label}</span>
  {@render children()}
  {#if error}<small class="err" role="alert">{error}</small>{:else if hint}<small class="hint">{hint}</small>{/if}
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }
  .name {
    font-size: var(--fs-small);
    font-weight: 500;
    color: var(--ink-dim);
  }
  small {
    font-size: var(--fs-small);
    overflow-wrap: anywhere;
  }
  .hint {
    color: var(--ink-dim);
  }
  .err {
    color: var(--bad);
  }
</style>
