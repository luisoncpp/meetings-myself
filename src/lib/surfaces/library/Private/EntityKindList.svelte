<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import type { EntityKind } from '../../../planning-actions';
  import GoalRow from './GoalRow.svelte';
  import HabitRow from './HabitRow.svelte';
  import TaskRow from './TaskRow.svelte';
  import ValueRow from './ValueRow.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    kind: EntityKind;
    view: LibraryView;
    store: LibraryStore;
  }

  let { kind, view, store }: Props = $props();
</script>

{#if kind === 'value'}
  {#each view.values as value (value.id)}
    <ValueRow {value} {view} {store} />
  {/each}
{:else if kind === 'goal'}
  {#each view.goals as goal (goal.id)}
    <GoalRow {goal} {view} {store} />
  {/each}
{:else if kind === 'habit'}
  {#each view.habits as habit (habit.id)}
    <HabitRow {habit} {view} {store} />
  {/each}
{:else}
  {#each view.tasks as task (task.id)}
    <TaskRow {task} {view} {store} />
  {/each}
{/if}
