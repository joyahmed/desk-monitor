<script lang="ts">
  import Button from './Button.svelte';
  import Input from './Input.svelte';

  interface Props {
    /** What is asked for, e.g. "Linux password". Also the field's aria-label. */
    label: string;
    value: string;
    oninput: (value: string) => void;
    /** Enter or the ring button. */
    onyes: () => void;
    onno: () => void;
    /** Field placeholder, e.g. "Password for alex". Default: none. */
    placeholder?: string;
    /** Ring button text. Default "Log in". */
    action?: string;
    /** Pulse the ring (only when this is the action that needs the user now). Default false. */
    pulse?: boolean;
    /** Ring shows "Log in..." and everything is disabled. Default false. */
    busy?: boolean;
  }
  let {
    label,
    value,
    oninput,
    onyes,
    onno,
    placeholder,
    action = 'Log in',
    pulse = false,
    busy = false
  }: Props = $props();

  // At narrow widths the buttons would open under the fixed bottom rail: bring the whole row up.
  const reveal = (el: HTMLFormElement) => {
    el.scrollIntoView({ block: 'center' });
  };

  const submit = (e: SubmitEvent) => {
    e.preventDefault();
    if (value && !busy) onyes();
  };
</script>

<form class="ask" onsubmit={submit} {@attach reveal}>
  <span class="field">
    <span class="text">{label}</span>
    <Input type="password" {value} {oninput} {label} {placeholder} size="lg" disabled={busy} focus />
  </span>
  <span class="btns">
    <Button label={action} variant="ring" {pulse} {busy} onclick={onyes} disabled={!value} />
    <Button label="Cancel" onclick={onno} disabled={busy} />
  </span>
</form>

<style>
  .ask {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-4) var(--space-5);
    min-width: 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    flex-grow: 1;
    /* wraps the ring below before the field gets narrower than ~192px */
    flex-basis: calc(var(--space-8) * 3);
    max-width: calc(var(--space-8) * 5);
    min-width: 0;
  }
  .text {
    font-size: var(--fs-kicker);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: var(--ls-kicker);
    color: var(--ink-dim);
  }
  .btns {
    display: flex;
    flex: none;
    align-items: center;
    gap: var(--space-5);
  }
</style>
