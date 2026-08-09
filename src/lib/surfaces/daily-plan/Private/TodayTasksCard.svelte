<script lang="ts">
  import type { PlanTaskView } from '../../../domain';
  import { Card, OrderableList } from '../../../ui';
  import PlanTaskRow from './PlanTaskRow.svelte';
  import QuickAdd from './QuickAdd.svelte';

  interface Props {
    tasks: PlanTaskView[];
    onquickadd: (title: string) => void;
    onreorder: (order: string[]) => void;
    ontoggle: (task: PlanTaskView) => void;
  }

  let { tasks, onquickadd, onreorder, ontoggle }: Props = $props();
</script>

<Card>
  <h2>Today's Tasks</h2>
  <QuickAdd onsubmit={/* quick add */ onquickadd} />
  {#if tasks.length === 0}
    <p class="empty">No tasks for today. Add one above or pull from the task pool.</p>
  {:else}
    <OrderableList
      label="Today's tasks"
      items={tasks}
      getId={(task) => task.id}
      onreorder={/* reorder tasks */ onreorder}
    >
      {#snippet children(task)}
        <PlanTaskRow {task} ontoggle={/* toggle completion */ () => ontoggle(task)} />
      {/snippet}
    </OrderableList>
  {/if}
</Card>

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }
</style>
