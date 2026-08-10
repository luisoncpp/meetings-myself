<script lang="ts">
  import type { PlanTaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { ListRow, StateFlag, TaskCompletionToggle } from '../../../ui';

  interface Props {
    task: PlanTaskView;
    ontoggle: () => void;
  }

  let { task, ontoggle }: Props = $props();
</script>

<ListRow muted={task.archived || task.state === 'completed'}>
  {#snippet leading()}
    <TaskCompletionToggle
      completed={task.state === 'completed'}
      taskTitle={task.title}
      ontoggle={/* toggle completion */ ontoggle}
    />
  {/snippet}
  <div class="main">
    <span class="title">{task.title}</span>
    <div class="flags">
      {#if task.archived}
        <StateFlag kind="archived" />
      {/if}
      {#if task.overdue}
        <StateFlag kind="overdue" />
      {/if}
    </div>
  </div>
  {#snippet trailing()}
    <div class="chips">
      {#if task.importance === 'low'}
        <span class="chip">{t('dailyPlan.lowImportance')}</span>
      {:else if task.importance === 'high'}
        <span class="chip">{t('dailyPlan.highImportance')}</span>
      {/if}
      {#if task.urgency === 'low'}
        <span class="chip">{t('dailyPlan.lowUrgency')}</span>
      {:else if task.urgency === 'high'}
        <span class="chip">{t('dailyPlan.highUrgency')}</span>
      {/if}
    </div>
  {/snippet}
</ListRow>

<style>
  .main {
    min-width: 0;
  }

  .title {
    display: block;
  }

  .flags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-top: var(--space-1);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    justify-content: flex-end;
  }

  .chip {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
