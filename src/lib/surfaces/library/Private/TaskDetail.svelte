<script lang="ts">
  import type { LibraryView, TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { taskRef } from '../../../planning-actions';
  import { Button } from '../../../ui';
  import DetailLinks from './DetailLinks.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import RenameField from './RenameField.svelte';
  import TaskRowEdit from './TaskRowEdit.svelte';

  interface Props {
    task: TaskView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { task, view, store }: Props = $props();

  const end = $derived({ kind: 'task' as const, id: task.id });
  const ref = $derived(taskRef(task));
</script>

<RenameField
  title={task.title}
  label={t('library.fieldTitle')}
  onrename={/* rename */ (title) => void store.actions.rename(task.id, title)}
/>
<TaskRowEdit {task} {store} />
{#if !task.archived}
  <div class="focus">
    {#each store.actions.focusTargets as target (target.which)}
      <Button
        variant="secondary"
        pressed={target.taskIds.has(task.id)}
        onclick={/* toggle focus */ () => void store.actions.setFocus(target, task.id)}
      >
        {t(`library.focus.${target.which}`)}
      </Button>
    {/each}
  </div>
  <DetailLinks {end} title={task.title} {view} {store} />
{/if}
<div class="actions">
  {#if ref.done}
    <Button variant="quiet" onclick={/* reopen */ () => void store.actions.reopen(task.id)}>
      {t('taskMenu.reopen')}
    </Button>
  {:else}
    <Button variant="quiet" onclick={/* finish */ () => void store.actions.finish(ref)}>
      {t('taskMenu.doneAndArchive')}
    </Button>
  {/if}
  <Button
    variant="quiet"
    onclick={/* archive or restore */ () => void store.actions.setArchived(task.id, !task.archived)}
  >
    {task.archived ? t('common.restore') : t('common.archive')}
  </Button>
</div>

<style>
  .focus,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
