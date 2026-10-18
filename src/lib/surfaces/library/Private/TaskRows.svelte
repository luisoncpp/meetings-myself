<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { taskRef, type TaskMenuController } from '../../../planning-actions';
  import { TaskChips, TaskLine } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    tasks: TaskView[];
    store: LibraryStore;
    menu: TaskMenuController;
  }

  let { tasks, store, menu }: Props = $props();

  function focusOf(id: string): 'current' | 'next' | 'none' {
    if (store.focusIds.has(id)) return 'current';
    return store.nextFocusIds.has(id) ? 'next' : 'none';
  }

  function toggleOf(task: TaskView): (() => void) | undefined {
    if (!task.oneOff) return undefined;
    return () => void store.actions.toggleDone(taskRef(task));
  }
</script>

{#each tasks as task (task.id)}
  <li>
    <TaskLine
      title={task.title}
      focus={focusOf(task.id)}
      selected={store.selected?.kind === 'task' && store.selected.id === task.id}
      ontoggle={toggleOf(task)}
      onselect={/* open details */ () => store.select({ kind: 'task', id: task.id })}
      onmenu={/* open task menu */ (point) => menu.open(taskRef(task), point)}
    >
      {#snippet meta()}
        <TaskChips importance={task.importance} urgency={task.urgency} overdue={task.overdue} />
      {/snippet}
    </TaskLine>
  </li>
{/each}
