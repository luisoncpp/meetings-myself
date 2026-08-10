<script lang="ts">
  import { t } from '../../../i18n';
  import { MarkdownEditor } from '../../../ui';
  import type { SaveState } from './WeeklyReviewStore.svelte';

  interface Props {
    value: string;
    saveState: SaveState;
    oninput: (text: string) => void;
    onblur: () => void;
  }

  let { value, saveState, oninput, onblur }: Props = $props();

  const statusLabel = $derived(
    saveState === 'saving'
      ? t('weeklyReview.saving')
      : saveState === 'unsaved'
        ? t('weeklyReview.unsaved')
        : t('weeklyReview.saved'),
  );
</script>

<section class="reflection">
  <div class="header">
    <h2>{t('weeklyReview.reflection')}</h2>
    <span class="status" role="status">{statusLabel}</span>
  </div>
  <MarkdownEditor aria-label={t('weeklyReview.reflection')} {value} oninput={/* edit */ oninput} onblur={/* blur */ onblur} />
</section>

<style>
  .reflection {
    margin-bottom: var(--space-4);
  }

  .header {
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
    margin-bottom: var(--space-2);
  }

  h2 {
    margin: 0;
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .status {
    font-size: var(--text-label);
    font-weight: 500;
    color: var(--color-ink-muted);
  }
</style>
