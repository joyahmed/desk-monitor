<script lang="ts">
  import { iconPaths } from './icon-paths';

  interface Props {
    name: string;
    size?: number;
  }
  let { name, size = 20 }: Props = $props();
  const paths = $derived(iconPaths[name] ?? []);
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.5"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  {#each paths as d (d)}<path {d} />{/each}
</svg>

<style>
  svg {
    flex: none;
    display: block;
  }
</style>
