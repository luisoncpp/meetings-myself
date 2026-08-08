<script lang="ts">
  interface Props {
    onsubmit: (title: string) => void;
  }

  let { onsubmit }: Props = $props();

  let draft = $state('');

  function submit(): void {
    const title = draft.trim();
    if (title === '') return;
    onsubmit(title);
    draft = '';
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return;
    event.preventDefault();
    submit();
  }
</script>

<label class="field" for="quick-add">
  <span class="label">Add a task</span>
  <input
    id="quick-add"
    type="text"
    bind:value={draft}
    onkeydown={handleKeydown}
    autocomplete="off"
  />
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .label {
    font-size: var(--text-caption);
    color: var(--color-ink-muted);
  }

  input {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
  }

  input:focus-visible {
    outline: 2px solid var(--color-gold);
    outline-offset: 2px;
  }
</style>
