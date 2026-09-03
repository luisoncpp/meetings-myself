<script lang="ts">
  import type { CheckInOutcome, DailyPlanView, PlanTaskView } from '../../../domain';
  import { formatPlanDate } from '../../../domain';
  import { bcp47, localeStore, t } from '../../../i18n';
  import { Card } from '../../../ui';
  import HabitList from './HabitList.svelte';
  import PlanTaskRow from './PlanTaskRow.svelte';

  interface Props {
    plan: DailyPlanView;
    ontoggle: (task: PlanTaskView) => void;
    oncheckin: (habitId: string, outcome: CheckInOutcome) => void;
  }

  let { plan, ontoggle, oncheckin }: Props = $props();
</script>

<Card>
  <details>
    <summary>
      <span class="summary-copy">
        <h2>{t('dailyPlan.yesterday')}</h2>
        <span class="when">{formatPlanDate(plan.date, bcp47(localeStore.locale))}</span>
      </span>
    </summary>

    <h3>{t('dailyPlan.yesterdayTasks')}</h3>
    {#if plan.tasks.length === 0}
      <p class="empty">{t('dailyPlan.yesterdayTasksEmpty')}</p>
    {:else}
      <ul class="rows">
        {#each plan.tasks as task (task.id)}
          <li>
            <PlanTaskRow {task} ontoggle={/* toggle completion */ () => ontoggle(task)} />
          </li>
        {/each}
      </ul>
    {/if}

    <h3>{t('dailyPlan.yesterdayHabits')}</h3>
    {#if plan.habits.length === 0}
      <p class="empty">{t('dailyPlan.yesterdayHabitsEmpty')}</p>
    {:else}
      <HabitList habits={plan.habits} {oncheckin} />
    {/if}
  </details>
</Card>

<style>
  details {
    display: block;
  }

  summary {
    cursor: pointer;
    list-style: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    border-radius: var(--radius-card);
  }

  summary::-webkit-details-marker {
    display: none;
  }

  summary::after {
    content: '';
    width: 0.4rem;
    height: 0.4rem;
    flex-shrink: 0;
    border-right: 2px solid var(--color-ink-muted);
    border-bottom: 2px solid var(--color-ink-muted);
    transform: rotate(45deg);
    transition: transform var(--duration-fast) var(--ease-out);
  }

  details[open] summary::after {
    transform: rotate(225deg);
  }

  details[open] summary {
    margin-bottom: var(--space-4);
  }

  .summary-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  h2 {
    margin: 0;
    font-size: var(--text-headline);
    font-weight: 600;
  }

  h3 {
    margin: var(--space-4) 0 var(--space-3);
    font-size: var(--text-title);
    font-weight: 600;
  }

  .when,
  .empty {
    color: var(--color-ink-muted);
  }

  .when {
    font-size: var(--text-body);
  }

  .empty {
    margin: 0;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
</style>
