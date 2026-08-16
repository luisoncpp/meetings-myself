<script lang="ts">
  import type { GoalView, LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { LinkModal } from '../../../planning-actions';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import AssociationTags from './AssociationTags.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    goal: GoalView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { goal, view, store }: Props = $props();

  let linkModalOpen = $state(false);

  const end = $derived({ kind: 'goal' as const, id: goal.id });

  function toggleArchive(): void {
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
  <div class="content">
    <span>{goal.title}</span>
    {#if !goal.archived}
      <AssociationTags
        {end}
        {view}
        onunlink={(id) => void store.unlink(id)}
        onopenLink={() => (linkModalOpen = true)}
      />
    {/if}
  </div>
  {#snippet trailing()}
    {#if goal.archived}
      <StateFlag kind="archived" />
      <Button variant="quiet" onclick={/* restore goal */ toggleArchive}>{t('common.restore')}</Button>
    {:else}
      {#if goal.achieved}
        <StateFlag kind="completed" />
      {/if}
      {#if goal.targetDate}
        <span class="meta">{t('library.target', { date: goal.targetDate })}</span>
      {/if}
      <Button variant="quiet" onclick={/* toggle achieved */ toggleAchieved}>
        {goal.achieved ? t('library.markNotAchieved') : t('library.markAchieved')}
      </Button>
      <Button variant="quiet" onclick={/* archive goal */ toggleArchive}>{t('common.archive')}</Button>
    {/if}
  {/snippet}
</ListRow>

{#if linkModalOpen}
  <LinkModal
    fromEnd={end}
    fromTitle={goal.title}
    {view}
    onlink={(toEnd) => store.link(end, toEnd)}
    onclose={() => (linkModalOpen = false)}
  />
{/if}

<style>
  .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .meta {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>

