<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button, ListRow } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';
  import TaskRowEdit from './TaskRowEdit.svelte';
  import TaskRowFlags from './TaskRowFlags.svelte';

  interface Props {
    task: TaskView;
    store: LibraryStore;
  }

  let { task, store }: Props = $props();

  const showCompletion = $derived(task.oneOff);
  const muted = $derived(task.archived || (showCompletion && task.state === 'completed'));

  function toggleArchive(): void {
    const end = { kind: 'task' as const, id: task.id };
    if (task.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }

  function onCompletionToggle(): void {
    void store.toggleTask(task);
  }
</script>

<ListRow {muted}>
  <div class="task">
    <span class="title">{task.title}</span>
    <TaskRowFlags {task} />
    {#if !task.archived}
      <TaskRowEdit {task} {store} onCompletionToggle={onCompletionToggle} />
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={toggleArchive}>
      {task.archived ? t('common.restore') : t('common.archive')}
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
</style>
