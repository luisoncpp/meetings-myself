<script lang="ts">
  interface Props {
    id?: string;
    type?: string;
    value?: string;
    disabled?: boolean;
    'aria-label'?: string;
    list?: string;
    autocomplete?: HTMLInputElement['autocomplete'];
    onkeydown?: (event: KeyboardEvent) => void;
    onchange?: (event: Event) => void;
    oninput?: (event: Event) => void;
  }

  let {
    id,
    type = 'text',
    value = $bindable(''),
    disabled = false,
    'aria-label': ariaLabel,
    list,
    autocomplete,
    onkeydown,
    onchange,
    oninput,
  }: Props = $props();
</script>

<input
  class="input"
  {id}
  {type}
  bind:value
  {disabled}
  aria-label={ariaLabel}
  {list}
  {autocomplete}
  onkeydown={onkeydown}
  onchange={onchange}
  oninput={oninput}
/>

<style>
  .input {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
  }

  .input:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
