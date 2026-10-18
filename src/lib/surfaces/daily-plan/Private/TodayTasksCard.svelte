<script lang="ts">
  import type { PlanTaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { InlineAdd, OrderableList, type MenuPoint } from '../../../ui';
  import PlanTaskRow from './PlanTaskRow.svelte';

  interface Props {
    tasks: PlanTaskView[];
    focusIds: ReadonlySet<string>;
    onquickadd: (title: string) => void;
    onreorder: (order: string[]) => void;
    ontoggle: (task: PlanTaskView) => void;
    onmenu: (task: PlanTaskView, point: MenuPoint) => void;
  }

  let { tasks, focusIds, onquickadd, onreorder, ontoggle, onmenu }: Props = $props();
</script>

<section class="column" aria-labelledby="today-heading">
  <h2 id="today-heading">{t('dailyPlan.todayTasks')}</h2>
  {#if tasks.length === 0}
    <p class="empty">{t('dailyPlan.todayTasksEmpty')}</p>
  {:else}
    <OrderableList
      label={t('dailyPlan.todayTasksList')}
      items={tasks}
      getId={(task) => task.id}
      onreorder={/* reorder tasks */ onreorder}
    >
      {#snippet children(task)}
        <PlanTaskRow
          {task}
          inFocus={focusIds.has(task.id)}
          ontoggle={/* toggle completion */ () => ontoggle(task)}
          onmenu={/* open task menu */ (point) => onmenu(task, point)}
        />
      {/snippet}
    </OrderableList>
  {/if}
  <InlineAdd
    label={t('dailyPlan.addTask')}
    placeholder={t('dailyPlan.addTaskPlaceholder')}
    onsubmit={/* quick add */ onquickadd}
  />
</section>

<style>
  .column {
    padding: var(--space-3);
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }

  h2 {
    margin: 0 0 var(--space-2) var(--space-2);
    font-size: var(--text-title);
    font-weight: 600;
  }

  .empty {
    margin: 0 var(--space-2);
    color: var(--color-ink-muted);
  }
</style>
