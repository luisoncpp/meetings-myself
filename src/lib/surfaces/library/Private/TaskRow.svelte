<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import TaskEditFields from './TaskEditFields.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    task: TaskView;
    store: LibraryStore;
  }

  let { task, store }: Props = $props();

  function toggleArchive(): void {
    const end = { kind: 'task' as const, id: task.id };
    if (task.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }
</script>

<ListRow muted={task.archived || task.state === 'completed'}>
  {#snippet leading()}
    {#if !task.archived}
      <input
        type="checkbox"
        checked={task.state === 'completed'}
        aria-label={task.title}
        onchange={/* toggle completion */ () => void store.toggleTask(task)}
      />
    {/if}
  {/snippet}
  <div class="task">
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
    {#if !task.archived}
      <TaskEditFields
        {task}
        onimportancechange={/* set importance */ (event) =>
          void store.classifyTask(
            task.id,
            (event.currentTarget as HTMLSelectElement).value as TaskView['importance'],
            task.urgency,
          )}
        onurgencychange={/* set urgency */ (event) =>
          void store.classifyTask(
            task.id,
            task.importance,
            (event.currentTarget as HTMLSelectElement).value as TaskView['urgency'],
          )}
        ondeadlinechange={/* set deadline */ (event) => {
          const value = (event.currentTarget as HTMLInputElement).value;
          void store.setDeadline(task.id, value === '' ? null : value);
        }}
      />
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {task.archived ? 'Restore' : 'Archive'}
    </Button>
  {/snippet}
</ListRow>

<style>
  .task {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .title {
    font-weight: 500;
  }

  .flags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }
</style>
