<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    id?: string;
    value?: string;
    disabled?: boolean;
    'aria-label'?: string;
    onchange?: (event: Event) => void;
    children: Snippet;
  }

  let {
    id,
    value = $bindable(''),
    disabled = false,
    'aria-label': ariaLabel,
    onchange,
    children,
  }: Props = $props();
</script>

<select
  class="select"
  {id}
  bind:value
  {disabled}
  aria-label={ariaLabel}
  onchange={onchange}
>
  {@render children()}
</select>

<style>
  .select {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
  }

  .select:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
