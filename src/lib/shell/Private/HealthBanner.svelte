<script lang="ts">
  import type { StoreHealth } from '../../api';
  import { Button } from '../../ui';
  import HealthBannerMessage from './HealthBannerMessage.svelte';

  interface Props {
    health: StoreHealth;
    onretry?: () => void;
  }

  let { health, onretry }: Props = $props();

  const RETRYABLE = new Set(['folderMissing', 'lockedByAnotherDevice']);
  const canRetry = $derived(RETRYABLE.has(health.status) && !!onretry);
</script>

{#if health.status !== 'ready'}
  <div class="banner" role="alert">
    <span class="marker" aria-hidden="true"></span>
    <div class="content">
      <HealthBannerMessage {health} />
      {#if canRetry}
        <Button variant="secondary" onclick={onretry}>Try again</Button>
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
</style>
