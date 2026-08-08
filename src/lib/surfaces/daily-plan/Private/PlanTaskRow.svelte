<script lang="ts">
  import type { Classification, PlanTaskView } from '../../../domain';
  import { StateFlag } from '../../../ui';

  interface Props {
    task: PlanTaskView;
    ontoggle: () => void;
  }

  let { task, ontoggle }: Props = $props();

  const CLASS_LABELS: Record<Exclude<Classification, 'unclassified'>, string> = {
    low: 'Low',
    high: 'High',
  };
</script>

<div class="row" class:muted={task.archived || task.state === 'completed'}>
  <input
    type="checkbox"
    checked={task.state === 'completed'}
    aria-label={task.title}
    onchange={/* toggle completion */ ontoggle}
  />
  <div class="main">
    <span class="title">{task.title}</span>
    <div class="flags">
      {#if task.archived}
        <StateFlag kind="archived" />
      {/if}
      {#if task.overdue}
        <StateFlag kind="overdue" />
      {/if}
      {#if task.state === 'completed'}
        <StateFlag kind="completed" />
      {/if}
    </div>
  </div>
  <div class="chips">
    {#if task.importance !== 'unclassified'}
      <span class="chip">{CLASS_LABELS[task.importance]} importance</span>
    {/if}
    {#if task.urgency !== 'unclassified'}
      <span class="chip">{CLASS_LABELS[task.urgency]} urgency</span>
    {/if}
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .muted {
    color: var(--color-ink-muted);
  }

  .main {
    flex: 1;
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
