<script lang="ts">
  import { stateColour } from './state-colour';
  import type { Size } from './layout';

  interface Props {
    state: string;
    /** Dot diameter: sm 6px, md 8px (default), lg 10px. */
    size?: Size;
  }
  let { state, size = 'md' }: Props = $props();
  const colour = $derived(stateColour(state));
</script>

<span class="dot {size}" style:background={`var(--${colour})`} role="img" aria-label={state}></span>

<style>
  .dot {
    display: inline-block;
    border-radius: 50%;
    flex: none;
    vertical-align: middle;
    transition: background var(--t-state) ease;
  }
  .sm {
    width: var(--dot-sm);
    height: var(--dot-sm);
  }
  .md {
    width: var(--dot-md);
    height: var(--dot-md);
  }
  .lg {
    width: var(--dot-lg);
    height: var(--dot-lg);
  }
</style>
