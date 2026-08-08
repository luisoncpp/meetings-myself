<script lang="ts">
  import type { Classification, TaskView } from '../../../domain';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import { CLASSIFICATION_OPTIONS } from './labels';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    task: TaskView;
    store: LibraryStore;
  }

  let { task, store }: Props = $props();

  function toggleArchive(): void {
    const end = { kind: 'task' as const, id: task.id };
    if (task.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }

  function onImportanceChange(event: Event): void {
    const select = event.currentTarget as HTMLSelectElement;
    void store.classifyTask(
      task.id,
      select.value as Classification,
      task.urgency,
    );
  }

  function onUrgencyChange(event: Event): void {
    const select = event.currentTarget as HTMLSelectElement;
    void store.classifyTask(
      task.id,
      task.importance,
      select.value as Classification,
    );
  }

  function onDeadlineChange(event: Event): void {
    const input = event.currentTarget as HTMLInputElement;
    const deadline = input.value === '' ? null : input.value;
    void store.setDeadline(task.id, deadline);
  }
</script>

<ListRow muted={task.archived || task.state === 'completed'}>
  {#snippet leading()}
    {#if !task.archived}
      <input
        type="checkbox"
        checked={task.state === 'completed'}
        aria-label={task.title}
        onchange={/* toggle completion */ () => void store.toggleTask(task)}
      />
    {/if}
  {/snippet}
  <div class="task">
    <span class="title">{task.title}</span>
    <div class="flags">
      {#if task.archived}
        <StateFlag kind="archived" />
      {/if}
      {#if task.overdue}
        <StateFlag kind="overdue" />
      {/if}
      {#if task.state === 'completed'}
        <StateFlag kind="completed" />
      {/if}
    </div>
    {#if !task.archived}
      <div class="fields">
        <label class="field">
          <span class="label">Importance</span>
          <select
            aria-label="Importance"
            value={task.importance}
            onchange={/* set importance */ onImportanceChange}
          >
            {#each CLASSIFICATION_OPTIONS as option (option.value)}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span class="label">Urgency</span>
          <select
            aria-label="Urgency"
            value={task.urgency}
            onchange={/* set urgency */ onUrgencyChange}
          >
            {#each CLASSIFICATION_OPTIONS as option (option.value)}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span class="label">Deadline</span>
          <input
            type="date"
            aria-label="Deadline"
            value={task.deadline ?? ''}
            onchange={/* set deadline */ onDeadlineChange}
          />
        </label>
      </div>
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {task.archived ? 'Restore' : 'Archive'}
    </Button>
  {/snippet}
</ListRow>

<style>
  .task {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .title {
    font-weight: 500;
  }

  .flags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .label {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
