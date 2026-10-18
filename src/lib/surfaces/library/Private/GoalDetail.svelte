<script lang="ts">
  import type { GoalView, LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button } from '../../../ui';
  import DetailLinks from './DetailLinks.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    goal: GoalView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { goal, view, store }: Props = $props();

  const end = $derived({ kind: 'goal' as const, id: goal.id });
</script>

{#if goal.targetDate}
  <p class="meta">{t('library.target', { date: goal.targetDate })}</p>
{/if}
{#if !goal.archived}
  <DetailLinks {end} title={goal.title} {view} {store} />
{/if}
<div class="actions">
  {#if !goal.archived}
    <Button variant="quiet" onclick={/* toggle achieved */ () => void store.setGoalAchieved(goal.id, !goal.achieved)}>
      {goal.achieved ? t('library.markNotAchieved') : t('library.markAchieved')}
    </Button>
  {/if}
  <Button variant="quiet" onclick={/* archive or restore */ () => void store.setArchived(end, !goal.archived)}>
    {goal.archived ? t('common.restore') : t('common.archive')}
  </Button>
</div>

<style>
  .meta {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }
</style>
