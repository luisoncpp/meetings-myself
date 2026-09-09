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
  let open = $state(/*expanded=*/ false);
</script>

<Card>
  <h2>
    <button
      type="button"
      class="toggle"
      aria-expanded={open}
      aria-controls="yesterday-panel"
      onclick={/* toggleYesterday= */ () => {
        open = !open;
      }}
    >
      <span class="summary-copy">
        <span class="title">{t('dailyPlan.yesterday')}</span>
        <span class="when">{formatPlanDate(plan.date, bcp47(localeStore.locale))}</span>
      </span>
      <span class="chevron" aria-hidden="true"></span>
    </button>
  </h2>

  {#if open}
    <div id="yesterday-panel">
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
    </div>
  {/if}
</Card>

<style>
  h2 {
    margin: 0;
  }

  .toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-card);
  }

  .summary-copy {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    min-width: 0;
  }

  .title {
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .chevron {
    width: 0.4rem;
    height: 0.4rem;
    flex-shrink: 0;
    border-right: 2px solid var(--color-ink-muted);
    border-bottom: 2px solid var(--color-ink-muted);
    transform: rotate(45deg);
    transition: transform var(--duration-fast) var(--ease-out);
  }

  .toggle[aria-expanded='true'] .chevron {
    transform: rotate(225deg);
  }

  .toggle[aria-expanded='true'] {
    margin-bottom: var(--space-4);
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
