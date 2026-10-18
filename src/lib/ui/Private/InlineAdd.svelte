<script lang="ts">
  interface Props {
    label: string;
    placeholder: string;
    onsubmit: (title: string) => void;
  }

  let { label, placeholder, onsubmit }: Props = $props();

  let draft = $state('');

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    const title = draft.trim();
    if (title === '') return;
    onsubmit(title);
    draft = '';
  }
</script>

<!-- Quiet dashed field at the end of a list: type, press Enter, keep going. -->
<input
  class="inline-add"
  type="text"
  aria-label={label}
  {placeholder}
  bind:value={draft}
  onkeydown={handleKeydown}
  autocomplete="off"
/>

<style>
  .inline-add {
    width: 100%;
    margin-top: var(--space-2);
    padding: var(--space-2);
    border: 1px dashed var(--color-hairline);
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink);
    font: inherit;
  }

  .inline-add::placeholder {
    color: var(--color-ink-muted);
  }
</style>
