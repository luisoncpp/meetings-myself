<script lang="ts">
  import { formatPlanDate } from '../../../domain';
  import { bcp47, localeStore, t } from '../../../i18n';
  import { planTaskRef, TaskMenu, TaskMenuController, taskRef } from '../../../planning-actions';
  import { SurfaceLayout } from '../../../ui';
  import { DailyPlanStore } from './DailyPlanStore.svelte';
  import HabitsCard from './HabitsCard.svelte';
  import TaskPool from './TaskPool.svelte';
  import TodayTasksCard from './TodayTasksCard.svelte';
  import YesterdayCard from './YesterdayCard.svelte';

  let store = $state<DailyPlanStore>();
  const menu = new TaskMenuController();

  $effect(/* createAndLoadStore */ () => {
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

        {#if activeStore.yesterday}
          {@const yesterday = activeStore.yesterday}
          <YesterdayCard
            plan={yesterday}
            ontoggle={/* toggle yesterday completion */ (task) =>
              void activeStore.toggleCompletion(task, yesterday.date)}
            oncheckin={/* record yesterday check-in */ (habitId, outcome) =>
              void activeStore.checkIn(habitId, outcome, yesterday.date)}
            onmenu={/* open task menu */ (task, point) => menu.open(planTaskRef(task), point)}
          />
        {/if}

        <HabitsCard
          habits={planView.habits}
          oncheckin={/* record check-in */ (habitId, outcome) =>
            void activeStore.checkIn(habitId, outcome, planView.date)}
        />

        <div class="columns">
          <TodayTasksCard
            tasks={planView.tasks}
            focusIds={activeStore.focusIds}
            onquickadd={/* quick add */ (title) => void activeStore.quickAdd(title)}
            onreorder={/* reorder tasks */ (order) => void activeStore.reorder(order)}
            ontoggle={/* toggle completion */ (task) =>
              void activeStore.toggleCompletion(task, planView.date)}
            onmenu={/* open task menu */ (task, point) => menu.open(planTaskRef(task), point)}
          />
          {#if activeStore.pool}
            <TaskPool
              pool={activeStore.pool}
              onselect={/* add from pool */ (taskId) => void activeStore.select(taskId)}
              onmenu={/* open task menu */ (task, point) => menu.open(taskRef(task), point)}
            />
          {/if}
        </div>

        {#if activeStore.error}
          <p class="error" role="alert">{activeStore.error}</p>
        {/if}
      </section>
      <TaskMenu controller={menu} actions={activeStore.actions} />
    </SurfaceLayout>
  {/if}
{/if}

<style>
  .daily-plan {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  h1 {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 600;
    line-height: 1.15;
  }

  .columns {
    display: grid;
    grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
    gap: var(--space-4);
    align-items: start;
  }

  .loading {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .error {
    margin: 0;
    color: var(--color-overdue);
  }
</style>
