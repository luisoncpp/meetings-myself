<script lang="ts">
  import {
    chooseSyncFolder,
    pickSyncFolder,
    reconnectStore,
    storeHealth,
    type StoreHealth,
  } from '../../api';
  import { t } from '../../i18n';
  import { SurfaceLayout } from '../../ui';
  import { Setup } from '../../surfaces/setup';
  import { DailyPlan } from '../../surfaces/daily-plan';
  import { Library } from '../../surfaces/library';
  import { WeeklyReview } from '../../surfaces/weekly-review';
  import HealthBanner from './HealthBanner.svelte';
  import Navigation, { type MainView } from './Navigation.svelte';

  let health: StoreHealth | null = $state(null);
  // Sections remount on switch, so each one reloads and shows the others' edits.
  let mainView: MainView = $state('today');
  let folderRevision = $state(0);

  async function refreshHealth(): Promise<void> {
    health = await storeHealth();
  }

  async function retryHealth(): Promise<void> {
    health = await reconnectStore();
  }

  async function chooseReplacementFolder(): Promise<void> {
    const folder = await pickSyncFolder();
    if (folder === null) return;
    health = await chooseSyncFolder(folder);
    folderRevision += 1;
  }

  $effect(() => {
    void refreshHealth();
  });
</script>

<div class="shell">
  {#if health === null}
    <SurfaceLayout>
      <p class="loading">{t('shell.loading')}</p>
    </SurfaceLayout>
  {:else if health.status === 'setupIncomplete'}
    <Setup {health} onready={refreshHealth} />
  {:else if health.status !== 'ready'}
    <SurfaceLayout>
      <div class="health-gate">
        <HealthBanner
          health={health}
          onretry={retryHealth}
          onchooseFolder={chooseReplacementFolder}
        />
      </div>
    </SurfaceLayout>
  {:else}
    <div class="frame">
      <Navigation
        current={mainView}
        onnavigate={/* switch main view */ (view) => (mainView = view)}
        onswitchFolder={chooseReplacementFolder}
      />
      <main class="content">
        {#key folderRevision}
          {#if mainView === 'today'}
            <DailyPlan />
          {:else if mainView === 'library'}
            <Library />
          {:else}
            <WeeklyReview />
          {/if}
        {/key}
      </main>
    </div>
  {/if}
</div>

<style>
  .shell {
    min-height: 100vh;
    background: var(--color-base);
    color: var(--color-ink);
    font-family: var(--font-sans);
    font-size: var(--text-body);
  }

  .frame {
    display: grid;
    grid-template-columns: 9rem minmax(0, 1fr);
    min-height: 100vh;
  }

  .content {
    min-width: 0;
  }

  .loading {
    margin: 0;
    color: var(--color-ink-muted);
  }
</style>
