<script lang="ts">
  import type { CheckInOutcome, DailyPlanView, PlanTaskView } from '../../../domain';
  import { formatPlanDate } from '../../../domain';
  import { bcp47, localeStore, t } from '../../../i18n';
  import type { MenuPoint } from '../../../ui';
  import HabitList from './HabitList.svelte';
  import PlanTaskRow from './PlanTaskRow.svelte';

  interface Props {
    plan: DailyPlanView;
    ontoggle: (task: PlanTaskView) => void;
    oncheckin: (habitId: string, outcome: CheckInOutcome) => void;
    onmenu: (task: PlanTaskView, point: MenuPoint) => void;
  }

  let { plan, ontoggle, oncheckin, onmenu }: Props = $props();
  let open = $state(/*expanded=*/ false);

  const openTasks = $derived(plan.tasks.filter((task) => !task.completed).length);
  const unchecked = $derived(plan.habits.filter((habit) => habit.outcome === null).length);
</script>

<section class="yesterday">
  <button
    type="button"
    class="toggle"
    aria-expanded={open}
    aria-controls="yesterday-panel"
    onclick={/* toggleYesterday= */ () => {
      open = !open;
    }}
  >
    <span class="title">{t('dailyPlan.yesterday')}</span>
    <span class="when">{formatPlanDate(plan.date, bcp47(localeStore.locale))}</span>
    <span class="counts">{t('dailyPlan.yesterdayCounts', { open: openTasks, unchecked })}</span>
    <span class="chevron" aria-hidden="true"></span>
  </button>

  {#if open}
    <div id="yesterday-panel" class="panel">
      {#if plan.tasks.length === 0}
        <p class="empty">{t('dailyPlan.yesterdayTasksEmpty')}</p>
      {:else}
        <ul class="rows" aria-label={t('dailyPlan.yesterdayTasks')}>
          {#each plan.tasks as task (task.id)}
            <li>
              <PlanTaskRow
                {task}
                ontoggle={/* toggle completion */ () => ontoggle(task)}
                onmenu={/* open task menu */ (point) => onmenu(task, point)}
              />
            </li>
          {/each}
        </ul>
      {/if}
      {#if plan.habits.length === 0}
        <p class="empty">{t('dailyPlan.yesterdayHabitsEmpty')}</p>
      {:else}
        <HabitList habits={plan.habits} {oncheckin} />
      {/if}
    </div>
  {/if}
</section>

<style>
  .yesterday {
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }

  .toggle {
    width: 100%;
    display: flex;
    align-items: baseline;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    border: 0;
    border-radius: var(--radius-card);
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .title {
    font-weight: 600;
  }

  .when,
  .counts,
  .empty {
    color: var(--color-ink-muted);
  }

  .counts {
    margin-left: auto;
    font-size: var(--text-label);
  }

  .chevron {
    align-self: center;
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

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    padding: 0 var(--space-3) var(--space-3);
  }

  .empty {
    margin: 0 var(--space-2);
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }
</style>
