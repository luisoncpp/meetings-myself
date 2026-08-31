<script lang="ts">
  import type { StoreHealth } from '../../api';
  import { Button } from '../../ui';
  import { t } from '../../i18n';
  import HealthBannerMessage from './HealthBannerMessage.svelte';

  interface Props {
    health: StoreHealth;
    onretry?: () => void;
    onchooseFolder?: () => void;
  }

  let { health, onretry, onchooseFolder }: Props = $props();

  const RETRYABLE = new Set(['folderMissing', 'lockedByAnotherDevice']);
  const canRetry = $derived(RETRYABLE.has(health.status) && !!onretry);
  const canChooseFolder = $derived(health.status === 'folderMissing' && !!onchooseFolder);
</script>

{#if health.status !== 'ready'}
  <div class="banner" role="alert">
    <span class="marker" aria-hidden="true"></span>
    <div class="content">
      <HealthBannerMessage {health} />
      {#if canRetry || canChooseFolder}
        <div class="actions">
          {#if canRetry}
            <Button variant="secondary" onclick={onretry}>{t('common.tryAgain')}</Button>
          {/if}
          {#if canChooseFolder}
            <Button variant="primary" onclick={onchooseFolder}>
              {t('health.chooseDifferentFolder')}
            </Button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-4);
    background: var(--color-lift);
    border-radius: var(--radius-card);
  }

  .marker {
    flex-shrink: 0;
    width: var(--space-1);
    border-radius: var(--radius-pill);
    background: var(--color-overdue);
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
