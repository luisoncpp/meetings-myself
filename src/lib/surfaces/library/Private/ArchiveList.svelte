<script lang="ts">
  import type { LibraryView, TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { taskRef, type TaskMenuController } from '../../../planning-actions';
  import { Button, TaskLine } from '../../../ui';
  import ArchivedOthers from './ArchivedOthers.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    view: LibraryView;
    store: LibraryStore;
    menu: TaskMenuController;
  }

  let { view, store, menu }: Props = $props();

  const tasks = $derived(view.tasks.filter((task) => task.archived));
  const done = $derived(
    tasks
      .filter((task) => task.completedOn !== null)
      .sort((a, b) => (b.completedOn ?? '').localeCompare(a.completedOn ?? '')),
  );
  const notDone = $derived(tasks.filter((task) => task.completedOn === null));
  const nothingElse = $derived(
    [view.habits, view.goals, view.values].every((items) => !items.some((item) => item.archived)) &&
      store.archivedRecurringTasks.length === 0,
  );
  const groups = $derived([
    { id: 'done', label: t('library.archive.done'), tasks: done },
    { id: 'not-done', label: t('library.archive.notDone'), tasks: notDone },
  ].filter((group) => group.tasks.length > 0));
</script>

{#snippet taskRow(task: TaskView)}
  <li>
    <TaskLine
      title={task.title}
      done={task.completedOn !== null}
      ontoggle={/* reopen or complete */ () => void store.actions.toggleDone(taskRef(task))}
      onselect={/* open details */ () => store.select({ kind: 'task', id: task.id })}
      onmenu={/* open task menu */ (point) => menu.open(taskRef(task), point)}
    >
      {#snippet meta()}<span>{task.completedOn ?? ''}</span>{/snippet}
      {#snippet trailing()}
        <Button variant="quiet" onclick={/* restore */ () => void store.actions.setArchived(task.id, false)}>
          {t('common.restore')}
        </Button>
      {/snippet}
    </TaskLine>
  </li>
{/snippet}

{#if tasks.length === 0 && nothingElse}
  <p class="empty">{t('library.archive.empty')}</p>
{/if}
{#each groups as group (group.id)}
  <h3>{group.label}</h3>
  <ul class="rows">{#each group.tasks as task (task.id)}{@render taskRow(task)}{/each}</ul>
{/each}
<ArchivedOthers {view} {store} />

<style>
  h3 {
    margin: var(--space-4) var(--space-2) var(--space-1);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .empty {
    margin: var(--space-2);
    color: var(--color-ink-muted);
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
