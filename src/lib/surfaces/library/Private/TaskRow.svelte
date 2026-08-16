<script lang="ts">
  import type { LibraryView, TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { LinkModal } from '../../../planning-actions';
  import { Button, ListRow } from '../../../ui';
  import AssociationTags from './AssociationTags.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import TaskRowEdit from './TaskRowEdit.svelte';
  import TaskRowFlags from './TaskRowFlags.svelte';

  interface Props {
    task: TaskView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { task, view, store }: Props = $props();

  let linkModalOpen = $state(false);

  const end = $derived({ kind: 'task' as const, id: task.id });
  const showCompletion = $derived(task.oneOff);
  const muted = $derived(task.archived || (showCompletion && task.state === 'completed'));

  function toggleArchive(): void {
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
      <AssociationTags
        {end}
        {view}
        onunlink={(id) => void store.unlink(id)}
        onopenLink={() => (linkModalOpen = true)}
      />
      <TaskRowEdit {task} {store} onCompletionToggle={onCompletionToggle} />
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={toggleArchive}>
      {task.archived ? t('common.restore') : t('common.archive')}
    </Button>
  {/snippet}
</ListRow>

{#if linkModalOpen}
  <LinkModal
    fromEnd={end}
    fromTitle={task.title}
    {view}
    onlink={(toEnd) => store.link(end, toEnd)}
    onclose={() => (linkModalOpen = false)}
  />
{/if}

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

