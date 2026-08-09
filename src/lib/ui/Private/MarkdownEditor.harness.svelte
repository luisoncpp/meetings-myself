<script lang="ts">
  interface Props {
    value?: string;
    disabled?: boolean;
    'aria-label'?: string;
    oninput?: (text: string) => void;
    onblur?: () => void;
  }

  let {
    value = '',
    disabled = false,
    'aria-label': ariaLabel = 'Editor',
    oninput,
    onblur,
  }: Props = $props();

  function handleInput(event: Event): void {
    const target = event.currentTarget;
    if (target === null || !('value' in target) || typeof target.value !== 'string') return;
    oninput?.(target.value);
  }
</script>

<textarea
  class="harness"
  aria-label={ariaLabel}
  {disabled}
  {value}
  oninput={/* edit */ handleInput}
  onblur={/* blur */ onblur}
></textarea>

<style>
  .harness {
    width: 100%;
    box-sizing: border-box;
    min-height: 12rem;
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
    line-height: 1.5;
  }
</style>
