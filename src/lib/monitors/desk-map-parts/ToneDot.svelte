<script lang="ts">
  interface Props {
    /** Token name: ok / warn / bad / info / off. */
    tone: string;
    label: string;
  }
  let { tone, label }: Props = $props();
</script>

<span class="dot" style:background={`var(--${tone})`} role="img" aria-label={label}></span>

<style>
  .dot {
    display: inline-block;
    width: var(--dot-md);
    height: var(--dot-md);
    border-radius: 50%;
    flex: none;
    transition: background var(--t-state) ease;
  }
</style>
