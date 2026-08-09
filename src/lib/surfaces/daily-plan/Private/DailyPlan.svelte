<script lang="ts">
  import { formatPlanDate } from '../../../domain';
  import { Card, OrderableList, SurfaceLayout } from '../../../ui';
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

{#if store}
  {@const activeStore = store}
  {#if activeStore.loading && !activeStore.plan}
    <SurfaceLayout>
      <p class="loading">Loading plan…</p>
    </SurfaceLayout>
  {:else if activeStore.plan}
    {@const planView = activeStore.plan}
    <SurfaceLayout aria-labelledby="plan-date">
      <section class="daily-plan">
        <h1 id="plan-date">{formatPlanDate(planView.date)}</h1>

        <div class="sections">
          <Card>
            <h2>Today's Tasks</h2>
            <QuickAdd onsubmit={/* quick add */ (title) => void activeStore.quickAdd(title)} />
            {#if planView.tasks.length === 0}
              <p class="empty">
                No tasks for today. Add one above or pull from the task pool.
              </p>
            {:else}
              <OrderableList
                label="Today's tasks"
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
            {/if}
          </Card>

          <Card>
            <h2>Habits</h2>
            {#if planView.habits.length === 0}
              <p class="empty">No habits due today.</p>
            {:else}
              <HabitList
                habits={planView.habits}
                oncheckin={/* record check-in */ (habitId, outcome) =>
                  void activeStore.checkIn(habitId, outcome)}
              />
            {/if}
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

  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .loading,
  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
