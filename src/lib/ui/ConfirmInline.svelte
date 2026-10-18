<script lang="ts">
  import Button from './Button.svelte';
  import type { Size } from './layout';

  interface Props {
    text: string;
    onyes: () => void;
    onno: () => void;
    busy?: boolean;
    /** Size of the Yes / No buttons. Default md. */
    size?: Size;
  }
  let { text, onyes, onno, busy = false, size = 'md' }: Props = $props();
</script>

<div class="confirm" role="group" aria-label={text}>
  <span class="text">{text}</span>
  <span class="btns">
    <Button label="Yes" variant="danger" onclick={onyes} {busy} {size} />
    <Button label="No" onclick={onno} disabled={busy} {size} />
  </span>
</div>

<style>
  .confirm {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-width: 0;
  }
  .text {
    flex: 1 1 0;
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .btns {
    display: flex;
    flex: none;
    gap: var(--space-4); /* side-by-side text links need room */
  }
</style>
