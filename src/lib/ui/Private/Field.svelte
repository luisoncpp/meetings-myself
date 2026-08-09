<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    label: string;
    forId?: string;
    children: Snippet;
  }

  let { label, forId, children }: Props = $props();
</script>

<label class="field" for={forId}>
  <span class="label">{label}</span>
  {@render children()}
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .label {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
