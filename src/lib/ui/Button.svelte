<script lang="ts">
  import type { Size } from './layout';

  interface Props {
    label: string;
    onclick: () => void;
    /** ghost (default) / primary / danger: underlined text links (primary bolder, danger in --bad).
     *  ring: the ONE round action per machine (Log in / Wake), 84px, size is ignored. */
    variant?: 'primary' | 'ghost' | 'danger' | 'ring';
    /** sm 24px, md 32px (default), lg 40px hit height. Not used by ring. */
    size?: Size;
    busy?: boolean;
    disabled?: boolean;
    title?: string;
    /** ring only: a slow pulse ring (--motion-pulse) - only when this action needs the user now. */
    pulse?: boolean;
    /** submit lets a <form> own Enter. Default button. */
    type?: 'button' | 'submit';
  }
  let {
    label,
    onclick,
    variant = 'ghost',
    size = 'md',
    busy = false,
    disabled = false,
    title,
    pulse = false,
    type = 'button'
  }: Props = $props();
</script>

<button
  class="{variant} {size}"
  class:pulse={variant === 'ring' && pulse && !disabled && !busy}
  {type}
  {title}
  {onclick}
  disabled={disabled || busy}
  aria-busy={busy}
>
  {busy ? `${label}...` : label}
</button>

<style>
  /* ---- text links: ghost / primary / danger ---- */
  button {
    flex: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    background: transparent;
    border: 0;
    border-bottom: var(--border-w) solid var(--link-line);
    border-radius: 0;
    color: var(--ink);
    transition:
      border-color var(--motion-hover),
      color var(--motion-hover),
      box-shadow var(--motion-hover);
  }
  .sm {
    height: var(--size-sm);
    min-height: var(--size-sm);
    padding: var(--space-0);
    font-size: var(--size-sm-fs);
  }
  .md {
    height: var(--size-md);
    min-height: var(--size-md);
    padding: var(--space-0);
    font-size: var(--size-md-fs);
  }
  .lg {
    height: var(--size-lg);
    min-height: var(--size-lg);
    padding: var(--space-0);
    font-size: var(--size-lg-fs);
  }
  button:hover:not(:disabled) {
    border-bottom-color: var(--sky);
  }
  .primary {
    font-weight: 600;
    border-bottom-color: var(--sky);
  }
  .primary:hover:not(:disabled) {
    color: var(--sky);
  }
  .danger {
    color: var(--bad);
    border-bottom-color: color-mix(in srgb, var(--bad) 40%, transparent);
  }
  .danger:hover:not(:disabled) {
    border-bottom-color: var(--bad);
  }

  /* ---- ring: the one round action ---- */
  .ring {
    position: relative;
    width: var(--ring-size);
    height: var(--ring-size);
    min-height: var(--ring-size);
    padding: var(--space-0);
    border: var(--ring-w) solid var(--sky);
    border-radius: 50%;
    background: radial-gradient(circle at 50% 35%, color-mix(in srgb, var(--sky) 14%, transparent), transparent 70%);
    box-shadow: 0 0 0 var(--ring-halo) color-mix(in srgb, var(--sky) 7%, transparent);
    font-size: var(--fs-small);
    font-weight: 600;
    white-space: normal;
    text-align: center;
  }
  .ring:hover:not(:disabled) {
    border-color: var(--sky);
    box-shadow:
      0 0 0 var(--ring-halo) color-mix(in srgb, var(--sky) 12%, transparent),
      0 0 28px color-mix(in srgb, var(--glow) 35%, transparent);
  }
  .ring.pulse::after {
    content: '';
    position: absolute;
    inset: calc(var(--ring-w) * -1);
    border: var(--ring-w) solid var(--sky);
    border-radius: 50%;
    pointer-events: none;
    animation: ring-pulse var(--motion-pulse) infinite;
  }
  @keyframes ring-pulse {
    from {
      transform: scale(1);
      opacity: 0.7;
    }
    to {
      transform: scale(1.4);
      opacity: 0;
    }
  }
</style>
