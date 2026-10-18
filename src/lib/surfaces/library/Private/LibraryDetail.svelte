<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../../domain';
  import { DetailPanel } from '../../../ui';
  import GoalDetail from './GoalDetail.svelte';
  import HabitDetail from './HabitDetail.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import TaskDetail from './TaskDetail.svelte';
  import ValueDetail from './ValueDetail.svelte';

  interface Props {
    selected: AssociationEnd;
    view: LibraryView;
    store: LibraryStore;
  }

  let { selected, view, store }: Props = $props();

  const task = $derived(selected.kind === 'task' ? view.tasks.find((x) => x.id === selected.id) : undefined);
  const habit = $derived(selected.kind === 'habit' ? view.habits.find((x) => x.id === selected.id) : undefined);
  const goal = $derived(selected.kind === 'goal' ? view.goals.find((x) => x.id === selected.id) : undefined);
  const value = $derived(selected.kind === 'value' ? view.values.find((x) => x.id === selected.id) : undefined);
  const title = $derived(task?.title ?? habit?.title ?? goal?.title ?? value?.title);
</script>

{#if title !== undefined}
  <DetailPanel {title} onclose={/* deselect */ () => store.select(null)}>
    {#if task}
      <TaskDetail {task} {view} {store} />
    {:else if habit}
      <HabitDetail {habit} {view} {store} />
    {:else if goal}
      <GoalDetail {goal} {view} {store} />
    {:else if value}
      <ValueDetail {value} {view} {store} />
    {/if}
  </DetailPanel>
{/if}
