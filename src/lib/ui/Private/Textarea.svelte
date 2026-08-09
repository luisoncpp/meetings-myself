<script lang="ts">
  interface Props {
    id?: string;
    value?: string;
    rows?: number;
    disabled?: boolean;
    'aria-label'?: string;
    oninput?: (event: Event) => void;
    onblur?: (event: Event) => void;
  }

  let {
    id,
    value = $bindable(''),
    rows,
    disabled = false,
    'aria-label': ariaLabel,
    oninput,
    onblur,
  }: Props = $props();
</script>

<textarea
  class="textarea"
  {id}
  bind:value
  {rows}
  {disabled}
  aria-label={ariaLabel}
  oninput={oninput}
  onblur={onblur}
></textarea>

<style>
  .textarea {
    width: 100%;
    box-sizing: border-box;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
    line-height: 1.5;
    resize: vertical;
  }

  .textarea:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
