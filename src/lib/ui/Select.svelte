<script lang="ts">
  import type { Size } from './layout';
  import type { SelectOption } from './select-options';

  interface Props {
    value: string;
    /** { value, label }[]; plain strings: toOptions(['a', 'b']) from ./select-options. */
    options: SelectOption[];
    onchange: (value: string) => void;
    /** sm 24px, md 32px (default), lg 40px. */
    size?: Size;
    disabled?: boolean;
    /** aria-label; omit when the Select sits inside a Field. */
    label?: string;
    id?: string;
  }
  let { value, options, onchange, size = 'md', disabled = false, label, id }: Props = $props();
</script>

<select
  class="control {size}"
  {value}
  {id}
  {disabled}
  aria-label={label}
  onchange={(e) => onchange(e.currentTarget.value)}
>
  {#each options as o (o.value)}
    <option value={o.value} selected={o.value === value}>{o.label}</option>
  {/each}
</select>

<style>
  /* a line, not a box: same underline as Input */
  .control {
    width: 100%;
    min-width: 0;
    background: transparent;
    color: var(--ink);
    border: 0;
    border-bottom: var(--border-w) solid var(--bezel);
    border-radius: 0;
    cursor: pointer;
    transition: border-color var(--motion-hover);
  }
  .control:hover:not(:disabled) {
    border-bottom-color: var(--ink-dim);
  }
  .control:focus-visible {
    outline: none;
    border-bottom-color: var(--sky);
    box-shadow: 0 var(--border-w) 0 var(--sky);
  }
  .control option {
    background: var(--surface);
    color: var(--ink);
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
    padding: var(--space-0) var(--space-1);
    font-size: var(--size-lg-fs);
  }
</style>
