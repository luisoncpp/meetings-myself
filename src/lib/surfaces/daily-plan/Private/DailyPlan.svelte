<script lang="ts">
  import { formatPlanDate } from '../../../domain';
  import { Card, OrderableList } from '../../../ui';
  import { DailyPlanStore } from './DailyPlanStore.svelte';
  import HabitList from './HabitList.svelte';
  import PlanTaskRow from './PlanTaskRow.svelte';
  import QuickAdd from './QuickAdd.svelte';
  import TaskPool from './TaskPool.svelte';

  let store = $state<DailyPlanStore>();

  $effect(() => {
    if (store) return;
    store = new DailyPlanStore();
    void store.load();
  });
</script>

{#if store && store.plan}
  {@const activeStore = store}
  {@const planView = activeStore.plan!}
  <section class="daily-plan" aria-labelledby="plan-date">
    <h1 id="plan-date">{formatPlanDate(planView.date)}</h1>

    <div class="sections">
      <Card>
        <h2>Today's Tasks</h2>
        <QuickAdd onsubmit={/* quick add */ (title) => void activeStore.quickAdd(title)} />
        <OrderableList
          items={planView.tasks}
          getId={(task) => task.id}
          onreorder={/* reorder tasks */ (order) => void activeStore.reorder(order)}
        >
          {#snippet children(task)}
            <PlanTaskRow
              {task}
              ontoggle={/* toggle completion */ () => void activeStore.toggleCompletion(task)}
            />
          {/snippet}
        </OrderableList>
      </Card>

      <Card>
        <h2>Habits</h2>
        <HabitList
          habits={planView.habits}
          oncheckin={/* record check-in */ (habitId, outcome) =>
            void activeStore.checkIn(habitId, outcome)}
        />
      </Card>

      {#if activeStore.pool}
        <Card>
          <TaskPool
            pool={activeStore.pool}
            onselect={/* add from pool */ (taskId) => void activeStore.select(taskId)}
          />
        </Card>
      {/if}
    </div>

    {#if activeStore.error}
      <p class="error" role="alert">{activeStore.error}</p>
    {/if}
  </section>
{/if}

<style>
  .daily-plan {
    padding: var(--space-6);
    max-width: 42rem;
  }

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

  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-title);
    font-weight: 600;
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
