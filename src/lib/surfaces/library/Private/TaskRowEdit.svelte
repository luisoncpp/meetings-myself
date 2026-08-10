<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { TaskCompletionToggle } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';
  import TaskEditFields from './TaskEditFields.svelte';

  interface Props {
    task: TaskView;
    store: LibraryStore;
    onCompletionToggle: () => void;
  }

  let { task, store, onCompletionToggle }: Props = $props();

  function onImportanceChange(event: Event): void {
    void store.classifyTask(
      task.id,
      (event.currentTarget as HTMLSelectElement).value as TaskView['importance'],
      task.urgency,
    );
  }

  function onUrgencyChange(event: Event): void {
    void store.classifyTask(
      task.id,
      task.importance,
      (event.currentTarget as HTMLSelectElement).value as TaskView['urgency'],
    );
  }

  function onDeadlineChange(event: Event): void {
    const value = (event.currentTarget as HTMLInputElement).value;
    void store.setDeadline(task.id, value === '' ? null : value);
  }

  function onOneOffChange(event: Event): void {
    const input = event.currentTarget as HTMLInputElement;
    void store.setTaskOneOff(task.id, input.checked);
  }
</script>

<TaskEditFields
  {task}
  onimportancechange={onImportanceChange}
  onurgencychange={onUrgencyChange}
  ondeadlinechange={onDeadlineChange}
/>
<div class="options-row">
  <label class="checkbox-label">
    <input type="checkbox" checked={task.oneOff} onchange={onOneOffChange} />
    {t('library.oneOff')}
  </label>
  {#if task.oneOff}
    <TaskCompletionToggle
      completed={task.state === 'completed'}
      taskTitle={task.title}
      ontoggle={onCompletionToggle}
    />
  {/if}
</div>

<style>
  .options-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
  }
</style>
