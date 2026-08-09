<script lang="ts">
  import type { Recurrence } from '../../../domain';
  import { Button, Card } from '../../../ui';
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

<section class="section" aria-label="Recurring tasks">
  <Card>
    <div class="header">
      <h2>Recurring tasks</h2>
      <Button variant="quiet" onclick={/* start create */ startCreate}>New recurring task</Button>
    </div>

    {#if creating}
      <CreateRecurringTask
        oncreate={/* create */ (payload) => void handleCreate(payload)}
        oncancel={/* cancel */ cancelCreate}
      />
    {/if}

    {#if tasks.length === 0}
      <p class="empty">No recurring tasks yet.</p>
    {:else}
      <div class="list">
        {#each tasks as task (task.id)}
          <RecurringTaskRow {task} {store} />
        {/each}
      </div>
    {/if}
  </Card>
</section>

<style>
  .section {
    margin-bottom: var(--space-4);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  h2 {
    margin: 0;
    font-size: var(--text-headline);
    font-weight: 600;
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
