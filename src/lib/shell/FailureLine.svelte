<script lang="ts">
  import Button from '$lib/ui/Button.svelte';

  /** A page-level failure: a line with the --bad bar on its left, the message, one link to act. */
  interface Props {
    /** Bold lead in --bad ("Could not load config."); null shows the message alone. */
    lead: string | null;
    msg: string;
    action: string;
    run: () => void;
  }
  let { lead, msg, action, run }: Props = $props();
</script>

<section class="failure" aria-label={lead ?? 'Error'}>
  <p class="text">
    {#if lead}<strong class="lead">{lead}</strong>{/if}
    <span class="msg" role="alert">{msg}</span>
  </p>
  <Button label={action} variant="primary" onclick={run} />
</section>

<style>
  .failure {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2) var(--space-5);
    padding: var(--space-1) var(--space-0) var(--space-1) var(--space-3);
    max-width: 44rem;
    border-left: var(--bar-w) solid var(--bad);
    animation: enter var(--motion-enter) both;
  }
  .text {
    flex: 1 1 16rem;
    min-width: 0;
    margin: var(--space-0);
    display: flex;
    flex-direction: column;
    gap: var(--space-hair);
  }
  .lead {
    color: var(--bad);
    font-weight: 600;
  }
  .msg {
    color: var(--ink);
    overflow-wrap: anywhere;
  }
  @keyframes enter {
    from {
      opacity: 0;
    }
  }
</style>
