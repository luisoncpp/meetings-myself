<script lang="ts">
  import { Textarea } from '../../../ui';
  import type { SaveState } from './WeeklyReviewStore.svelte';

  interface Props {
    value: string;
    saveState: SaveState;
    oninput: (text: string) => void;
    onblur: () => void;
  }

  let { value, saveState, oninput, onblur }: Props = $props();

  const statusLabel = $derived(
    saveState === 'saving' ? 'Saving…' : saveState === 'unsaved' ? 'Unsaved' : 'Saved',
  );

  function handleInput(event: Event): void {
    const target = event.currentTarget;
    if (target === null || !('value' in target) || typeof target.value !== 'string') return;
    oninput(target.value);
  }
</script>

<section class="reflection">
  <div class="header">
    <h2>Reflection</h2>
    <span class="status" role="status">{statusLabel}</span>
  </div>
  <Textarea
    aria-label="Reflection"
    rows={8}
    {value}
    oninput={/* edit */ handleInput}
    onblur={/* blur save */ onblur}
  />
</section>

<style>
  .reflection {
    margin-bottom: var(--space-4);
  }

  .header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  h2 {
    margin: 0;
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .status {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .reflection :global(textarea) {
    max-width: 65ch;
  }
</style>
