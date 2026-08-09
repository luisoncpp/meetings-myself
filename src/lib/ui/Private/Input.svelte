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
  class="input control-field"
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
