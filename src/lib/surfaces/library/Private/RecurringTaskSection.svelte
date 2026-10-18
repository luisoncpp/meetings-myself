<script lang="ts">
  import type { Recurrence } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button } from '../../../ui';
  import CreateRecurringTask from './CreateRecurringTask.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import RecurringTaskRow from './RecurringTaskRow.svelte';

  interface Props {
    store: LibraryStore;
  }

  let { store }: Props = $props();

  let creating = $state(false);

  const tasks = $derived(store.recurringTasks);

  function startCreate(): void {
    creating = true;
  }

  function cancelCreate(): void {
    creating = false;
  }

  async function handleCreate(payload: { title: string; recurrence: Recurrence }): Promise<void> {
    await store.createRecurringTask(payload.title, payload.recurrence);
    creating = false;
  }
</script>

<section aria-label={t('library.recurringTasks')}>
    <div class="header">
      <Button variant="quiet" onclick={/* start create */ startCreate}>+ {t('library.newRecurringTask')}</Button>
    </div>

    {#if creating}
      <CreateRecurringTask
        oncreate={/* create */ (payload) => void handleCreate(payload)}
        oncancel={/* cancel */ cancelCreate}
      />
    {/if}

    {#if tasks.length === 0}
      <p class="empty">{t('library.noRecurringTasks')}</p>
    {:else}
      <div class="list">
        {#each tasks as task (task.id)}
          <RecurringTaskRow {task} {store} />
        {/each}
      </div>
    {/if}
</section>

<style>
  .header {
    margin-bottom: var(--space-2);
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
</style>
