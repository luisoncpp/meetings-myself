<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    interactive?: boolean;
    children: Snippet;
  }

  let { interactive = false, children }: Props = $props();
</script>

<div class="card" class:interactive>
  {@render children()}
</div>

<style>
  .card {
    background: var(--color-lift);
    border-radius: var(--radius-card);
    padding: var(--space-4);
  }

  .interactive {
    transition: box-shadow var(--duration-fast) var(--ease-out);
  }

  .interactive:hover {
    box-shadow: var(--shadow-hover);
  }
</style>
