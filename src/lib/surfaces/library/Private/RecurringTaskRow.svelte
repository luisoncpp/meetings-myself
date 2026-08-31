<script lang="ts">
  import type { RecurringTask } from '../../../domain';
  import { localeStore, t } from '../../../i18n';
  import { Button, Input, ListRow, StateFlag } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';
  import { recurrenceLabel } from './recurrence-label';

  interface Props {
    task: RecurringTask;
    store: LibraryStore;
  }

  let { task, store }: Props = $props();

  const archived = $derived(task.lifecycle === 'archived');
  const recurrenceText = $derived.by(() => {
    if (localeStore.locale) {
      return recurrenceLabel(task.recurrence);
    }
    return '';
  });
  let title = $state('');

  $effect(() => {
    title = task.title;
  });

  function toggleArchive(): void {
    if (archived) {
      void store.restoreRecurringTask(task.id);
      return;
    }
    void store.archiveRecurringTask(task.id);
  }

  function onTitleChange(): void {
    const trimmed = title.trim();
    if (trimmed === '' || trimmed === task.title) return;
    void store.renameRecurringTask(task.id, trimmed);
  }
</script>

<ListRow muted={archived}>
  <div class="recurring-task">
    {#if archived}
      <span class="title">{task.title}</span>
    {:else}
      <Input
        bind:value={title}
        aria-label={t('library.recurringTaskTitle')}
        onchange={/* rename */ onTitleChange}
      />
    {/if}
    <span class="recurrence">{recurrenceText}</span>
    {#if archived}
      <StateFlag kind="archived" />
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {archived ? t('common.restore') : t('common.archive')}
    </Button>
  {/snippet}
</ListRow>

<style>
  .recurring-task {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .title {
    font-weight: 500;
  }

  .recurrence {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
