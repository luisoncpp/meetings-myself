<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    label: string;
    children: Snippet;
    footer?: Snippet;
  }

  let { title, label, children, footer }: Props = $props();
</script>

<section class="inset-panel" aria-label={label}>
  <h3>{title}</h3>
  {@render children()}
  {#if footer}
    {@render footer()}
  {/if}
</section>

<style>
  .inset-panel {
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
  }

  h3 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-title);
  }
</style>
