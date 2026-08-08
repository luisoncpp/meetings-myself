<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import type { EntityKind } from './associations';
  import GoalRow from './GoalRow.svelte';
  import HabitRow from './HabitRow.svelte';
  import TaskRow from './TaskRow.svelte';
  import ValueRow from './ValueRow.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    kind: EntityKind;
    view: LibraryView;
    store: LibraryStore;
    selectedGoalId?: string | null;
    onselectGoal?: (goalId: string) => void;
  }

  let { kind, view, store, selectedGoalId = null, onselectGoal }: Props = $props();
</script>

{#if kind === 'value'}
  {#each view.values as value (value.id)}
    <ValueRow {value} {store} />
  {/each}
{:else if kind === 'goal'}
  {#each view.goals as goal (goal.id)}
    <GoalRow
      {goal}
      {store}
      selected={selectedGoalId === goal.id}
      onselect={/* select goal */ () => onselectGoal?.(goal.id)}
    />
  {/each}
{:else if kind === 'habit'}
  {#each view.habits as habit (habit.id)}
    <HabitRow {habit} {store} />
  {/each}
{:else}
  {#each view.tasks as task (task.id)}
    <TaskRow {task} {store} />
  {/each}
{/if}
