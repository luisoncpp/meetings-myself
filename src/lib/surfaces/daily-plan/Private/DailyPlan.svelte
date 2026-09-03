<script lang="ts">
  import { formatPlanDate } from '../../../domain';
  import { bcp47, localeStore, t } from '../../../i18n';
  import { Card, SurfaceLayout } from '../../../ui';
  import { DailyPlanStore } from './DailyPlanStore.svelte';
  import HabitsCard from './HabitsCard.svelte';
  import TaskPool from './TaskPool.svelte';
  import TodayTasksCard from './TodayTasksCard.svelte';
  import YesterdayCard from './YesterdayCard.svelte';

  let store = $state<DailyPlanStore>();

  $effect(() => {
    if (store) return;
    store = new DailyPlanStore();
    void store.load();
  });
</script>

{#if store}
  {@const activeStore = store}
  {#if activeStore.loading && !activeStore.plan}
    <SurfaceLayout>
      <p class="loading">{t('dailyPlan.loading')}</p>
    </SurfaceLayout>
  {:else if activeStore.plan}
    {@const planView = activeStore.plan}
    <SurfaceLayout aria-labelledby="plan-date">
      <section class="daily-plan">
        <h1 id="plan-date">{formatPlanDate(planView.date, bcp47(localeStore.locale))}</h1>

        <div class="sections">
          <div class="task-columns">
            <TodayTasksCard
              tasks={planView.tasks}
              onquickadd={/* quick add */ (title) => void activeStore.quickAdd(title)}
              onreorder={/* reorder tasks */ (order) => void activeStore.reorder(order)}
              ontoggle={/* toggle completion */ (task) =>
                void activeStore.toggleCompletion(task, planView.date)}
            />
            {#if activeStore.pool}
              <Card>
                <TaskPool
                  pool={activeStore.pool}
                  onselect={/* add from pool */ (taskId) => void activeStore.select(taskId)}
                />
              </Card>
            {/if}
          </div>
          <HabitsCard
            habits={planView.habits}
            oncheckin={/* record check-in */ (habitId, outcome) =>
              void activeStore.checkIn(habitId, outcome, planView.date)}
          />
          {#if activeStore.yesterday}
            {@const yesterday = activeStore.yesterday}
            <YesterdayCard
              plan={yesterday}
              ontoggle={/* toggle yesterday completion */ (task) =>
                void activeStore.toggleCompletion(task, yesterday.date)}
              oncheckin={/* record yesterday check-in */ (habitId, outcome) =>
                void activeStore.checkIn(habitId, outcome, yesterday.date)}
            />
          {/if}
        </div>

        {#if activeStore.error}
          <p class="error" role="alert">{activeStore.error}</p>
        {/if}
      </section>
    </SurfaceLayout>
  {/if}
{/if}

<style>
  h1 {
    margin: 0 0 var(--space-6);
    font-size: var(--text-display);
    font-weight: 600;
    line-height: 1.15;
  }

  .sections {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .task-columns {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
    align-items: start;
  }

  .loading {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
