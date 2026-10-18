<script lang="ts">
  interface Props {
    /** The machine name (or "Your desk"): huge, condensed, lit by the headline gradient. */
    text: string;
    title?: string;
  }
  let { text, title }: Props = $props();
</script>

<h2 class="headline" {title}>{text}</h2>

<style>
  .headline {
    margin: 0;
    max-width: 100%;
    font-family: var(--font-display);
    font-weight: 900;
    font-size: var(--fs-headline-sm);
    line-height: var(--lh-headline);
    letter-spacing: 0.01em;
    text-transform: uppercase;
    overflow-wrap: anywhere;
    background: var(--grad-headline);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  @media (min-width: 640px) {
    .headline { font-size: var(--fs-headline); }
  }
</style>
