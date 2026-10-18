<script lang="ts">
  import { osFamily, osPaths } from './os-glyph-data';

  interface Props {
    os: string;
    size?: number;
  }
  let { os, size = 20 }: Props = $props();
  const paths = $derived(osPaths[osFamily(os)]);
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
