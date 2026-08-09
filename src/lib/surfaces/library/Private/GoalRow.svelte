<script lang="ts">
  import type { GoalView } from '../../../domain';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    goal: GoalView;
    store: LibraryStore;
  }

  let { goal, store }: Props = $props();

  function toggleArchive(): void {
    const end = { kind: 'goal' as const, id: goal.id };
    if (goal.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }

  function toggleAchieved(): void {
    if (goal.achieved) {
      void store.unachieveGoal(goal.id);
      return;
    }
    void store.achieveGoal(goal.id);
  }
</script>

<ListRow muted={goal.archived}>
  <span>{goal.title}</span>
  {#snippet trailing()}
    {#if goal.archived}
      <StateFlag kind="archived" />
      <Button variant="quiet" onclick={/* restore goal */ toggleArchive}>Restore</Button>
    {:else}
      {#if goal.achieved}
        <StateFlag kind="completed" />
      {/if}
      {#if goal.targetDate}
        <span class="meta">Target {goal.targetDate}</span>
      {/if}
      <Button variant="quiet" onclick={/* toggle achieved */ toggleAchieved}>
        {goal.achieved ? 'Mark not achieved' : 'Mark achieved'}
      </Button>
      <Button variant="quiet" onclick={/* archive goal */ toggleArchive}>Archive</Button>
    {/if}
  {/snippet}
</ListRow>

<style>
  .meta {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
