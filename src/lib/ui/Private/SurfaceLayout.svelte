<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    class?: string;
    'aria-labelledby'?: string;
    children: Snippet;
  }

  let { class: className, 'aria-labelledby': ariaLabelledby, children }: Props = $props();
</script>

<div class="surface-layout {className ?? ''}" aria-labelledby={ariaLabelledby}>
  {@render children()}
</div>

<style>
  .surface-layout {
    width: 100%;
    max-width: min(var(--content-max-width), 100%);
    margin-inline: auto;
    padding: var(--space-6);
  }
</style>
