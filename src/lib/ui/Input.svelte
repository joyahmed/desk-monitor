<script lang="ts">
  import type { Size } from './layout';

  interface Props {
    value: string;
    /** Every keystroke, with the new value. */
    oninput?: (value: string) => void;
    /** On commit (blur / Enter), with the new value. */
    onchange?: (value: string) => void;
    /** sm 24px, md 32px (default), lg 40px. */
    size?: Size;
    type?: 'text' | 'number' | 'password' | 'search' | 'url';
    placeholder?: string;
    readonly?: boolean;
    disabled?: boolean;
    /** aria-label; omit when the Input sits inside a Field (the label wraps it). */
    label?: string;
    id?: string;
    /** Monospace text (IPs, hostnames, keys). Default false. */
    mono?: boolean;
    autocomplete?: 'off' | 'on';
    spellcheck?: boolean;
    /** Take keyboard focus when it appears (an input the user just asked for). */
    focus?: boolean;
  }
  let {
    value,
    oninput,
    onchange,
    size = 'md',
    type = 'text',
    placeholder,
    readonly = false,
    disabled = false,
    label,
    id,
    mono = false,
    autocomplete = 'off',
    spellcheck = false,
    focus = false
  }: Props = $props();

  const takeFocus = (el: HTMLInputElement) => {
    if (focus) el.focus();
  };
</script>

<input
  class="control {size}"
  class:mono
  {type}
  {value}
  {id}
  {placeholder}
  {readonly}
  {disabled}
  {autocomplete}
  {spellcheck}
  aria-label={label}
  {@attach takeFocus}
  oninput={(e) => oninput?.(e.currentTarget.value)}
  onchange={(e) => onchange?.(e.currentTarget.value)}
/>

<style>
  /* a line, not a box: underline in --bezel, --sky when focused */
  .control {
    width: 100%;
    min-width: 0;
    background: transparent;
    color: var(--ink);
    border: 0;
    border-bottom: var(--border-w) solid var(--bezel);
    border-radius: 0;
    transition: border-color var(--motion-hover);
  }
  .control::placeholder {
    color: var(--placeholder);
  }
  .control:hover:not(:disabled):not(:read-only) {
    border-bottom-color: var(--ink-dim);
  }
  .control:focus-visible {
    outline: none;
    border-bottom-color: var(--sky);
    box-shadow: 0 var(--border-w) 0 var(--sky);
  }
  .control:read-only {
    color: var(--ink-dim);
    border-bottom-style: dashed;
  }
  .mono {
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .sm {
    height: var(--size-sm);
    min-height: var(--size-sm);
    padding: var(--space-0) var(--space-1);
    font-size: var(--size-sm-fs);
  }
  .md {
    height: var(--size-md);
    min-height: var(--size-md);
    padding: var(--space-0) var(--space-1);
    font-size: var(--size-md-fs);
  }
  .lg {
    height: var(--size-lg);
    min-height: var(--size-lg);
    padding: var(--space-0) var(--space-1);
    font-size: var(--size-lg-fs);
  }
</style>
