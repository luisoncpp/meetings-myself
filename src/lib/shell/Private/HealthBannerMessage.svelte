<script lang="ts">
  import type { StoreHealth } from '../../api';
  import { t } from '../../i18n';

  interface Props {
    health: StoreHealth;
  }

  let { health }: Props = $props();
</script>

{#if health.status === 'setupIncomplete'}
  <p>{t('health.finishSetup')}</p>
{:else if health.status === 'folderMissing'}
  <p>
    {t('health.folderMissing', { path: health.path })}
  </p>
{:else if health.status === 'lockedByAnotherDevice'}
  <p>
    {t('health.lockedByAnotherDevice', {
      deviceName: health.deviceName,
      since: health.since,
    })}
  </p>
{:else if health.status === 'syncConflict'}
  <p>{t('health.syncConflictIntro')}</p>
  <ul>
    {#each health.artifacts as artifact (artifact)}
      <li>{artifact}</li>
    {/each}
  </ul>
  <p>{t('health.syncConflictOutro')}</p>
{:else if health.status === 'unreadable'}
  <p>{t('health.unreadable', { detail: health.detail })}</p>
{/if}

<style>
  p,
  ul {
    margin: 0;
    font-size: var(--text-body);
    line-height: 1.5;
  }

  ul {
    padding-left: var(--space-4);
  }
</style>
