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
  class="select control-field"
  {id}
  bind:value
  {disabled}
  aria-label={ariaLabel}
  onchange={onchange}
>
  {@render children()}
</select>
