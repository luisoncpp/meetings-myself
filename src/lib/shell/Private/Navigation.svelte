<script lang="ts">
  import { openWeeklyReviewWindow } from '../../api';
  import { t } from '../../i18n';
  import { LanguageSelect } from '../../ui';

  interface Props {
    current: 'daily-plan' | 'library';
    onnavigate: (view: 'daily-plan' | 'library') => void;
    onswitchFolder?: () => void;
  }

  let { current, onnavigate, onswitchFolder }: Props = $props();

  async function openReviewWindow(): Promise<void> {
    await openWeeklyReviewWindow();
  }
</script>

<nav class="nav" aria-label={t('nav.main')}>
  <div class="nav-inner">
    <button
      type="button"
      class:selected={current === 'daily-plan'}
      onclick={/* show daily plan */ () => onnavigate('daily-plan')}
    >
      {t('nav.dailyPlan')}
    </button>
    <button
      type="button"
      class:selected={current === 'library'}
      onclick={/* show library */ () => onnavigate('library')}
    >
      {t('nav.library')}
    </button>
    <button
      type="button"
      aria-label={t('nav.openWeeklyReview')}
      onclick={openReviewWindow}
    >
      {t('nav.weeklyReview')} <span class="external" aria-hidden="true">↗</span>
    </button>
    <div class="actions">
      {#if onswitchFolder}
        <button
          type="button"
          class="switch-folder"
          aria-label={t('nav.switchFolder')}
          onclick={onswitchFolder}
        >
          {t('nav.switchFolder')}
        </button>
      {/if}
      <LanguageSelect compact />
    </div>
  </div>
</nav>

<style>
  .nav {
    border-bottom: 1px solid var(--color-hairline);
  }

  .nav-inner {
    display: flex;
    gap: var(--space-2);
    width: 100%;
    max-width: min(var(--content-max-width), 100%);
    margin-inline: auto;
    padding: var(--space-4) var(--space-6);
    align-items: center;
  }

  button {
    padding: var(--space-2) var(--space-3);
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-body);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  button:hover {
    color: var(--color-ink);
  }

  button.selected {
    color: var(--color-gold);
    font-weight: 600;
  }

  .external {
    font-size: var(--text-label);
    opacity: 0.7;
  }

  .actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
</style>
