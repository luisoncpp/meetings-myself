<script lang="ts">
  import { storeHealth, type StoreHealth } from '../../api';
  import { t } from '../../i18n';
  import { SurfaceLayout } from '../../ui';
  import { Setup } from '../../surfaces/setup';
  import { DailyPlan } from '../../surfaces/daily-plan';
  import { Library } from '../../surfaces/library';
  import { WeeklyReview } from '../../surfaces/weekly-review';
  import type { Surface } from './surface';
  import HealthBanner from './HealthBanner.svelte';
  import Navigation from './Navigation.svelte';

  interface Props {
    surface: Surface;
  }

  let { surface }: Props = $props();

  let health: StoreHealth | null = $state(null);
  let mainView: 'daily-plan' | 'library' = $state('daily-plan');

  async function refreshHealth(): Promise<void> {
    health = await storeHealth();
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
        <HealthBanner health={health} onretry={refreshHealth} />
      </div>
    </SurfaceLayout>
  {:else if surface === 'weekly-review'}
    <WeeklyReview />
  {:else}
    <Navigation current={mainView} onnavigate={/* switch main view */ (view) => (mainView = view)} />
    {#if mainView === 'daily-plan'}
      <DailyPlan />
    {:else}
      <Library />
    {/if}
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

  .loading {
    margin: 0;
    color: var(--color-ink-muted);
  }
</style>
