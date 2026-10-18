<script lang="ts">
  import type { TaskPoolView, TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { TaskChips, TaskLine, type MenuPoint } from '../../../ui';

  interface Props {
    pool: TaskPoolView;
    onselect: (taskId: string) => void;
    onmenu: (task: TaskView, point: MenuPoint) => void;
  }

  let { pool, onselect, onmenu }: Props = $props();
</script>

<section class="column" aria-labelledby="pool-heading">
  <h2 id="pool-heading">{t('dailyPlan.taskPool')}</h2>
  {#if pool.focus.length === 0 && pool.rest.length === 0}
    <p class="empty">{t('dailyPlan.taskPoolEmpty')}</p>
  {/if}
  <ul class="tasks" aria-label={t('dailyPlan.taskPoolRegion')}>
    {#each pool.focus as task (task.id)}
      <li>{@render poolRow(task, /* inFocus= */ true)}</li>
    {/each}
    {#each pool.rest as task (task.id)}
      <li>{@render poolRow(task, /* inFocus= */ false)}</li>
    {/each}
  </ul>
</section>

{#snippet poolRow(task: TaskView, inFocus: boolean)}
  <TaskLine
    title={task.title}
    focus={inFocus ? 'current' : 'none'}
    muted={!inFocus}
    onmenu={/* open task menu */ (point) => onmenu(task, point)}
  >
    {#snippet meta()}
      <TaskChips importance={task.importance} urgency={task.urgency} overdue={task.overdue} />
    {/snippet}
    {#snippet trailing()}
      <button
        type="button"
        class="add"
        aria-label={t('dailyPlan.addToTodayFor', { title: task.title })}
        title={t('dailyPlan.addToToday')}
        onclick={/* add to today */ () => onselect(task.id)}
      >
        ←
      </button>
    {/snippet}
  </TaskLine>
{/snippet}

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

  .tasks {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .add {
    flex-shrink: 0;
    padding: var(--space-1) var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: pointer;
  }

  .add:hover {
    background: var(--color-lift);
    color: var(--color-gold);
  }
</style>
