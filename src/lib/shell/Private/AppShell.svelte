<script lang="ts">
  import { storeHealth, type StoreHealth } from '../../api';
  import { Setup } from '../../surfaces/setup';
  import { DailyPlan } from '../../surfaces/daily-plan';
  import type { Surface } from './surface';
  import HealthBanner from './HealthBanner.svelte';
  import Navigation from './Navigation.svelte';
  import SurfacePlaceholder from './SurfacePlaceholder.svelte';

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
    <!-- Waiting for the first health check — no surface content yet. -->
  {:else if health.status === 'setupIncomplete'}
    <Setup {health} onready={refreshHealth} />
  {:else if health.status !== 'ready'}
    <div class="health-gate">
      <HealthBanner health={health} onretry={refreshHealth} />
    </div>
  {:else if surface === 'weekly-review'}
    <SurfacePlaceholder kind="weekly-review" />
  {:else}
    <Navigation current={mainView} onnavigate={/* switch main view */ (view) => (mainView = view)} />
    {#if mainView === 'daily-plan'}
      <DailyPlan />
    {:else}
      <SurfacePlaceholder kind="library" />
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

  .health-gate {
    padding: var(--space-6);
  }
</style>
