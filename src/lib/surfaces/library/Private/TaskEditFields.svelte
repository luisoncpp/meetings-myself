<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { CLASSIFICATION_OPTIONS } from './labels';

  interface Props {
    task: TaskView;
    onimportancechange: (event: Event) => void;
    onurgencychange: (event: Event) => void;
    ondeadlinechange: (event: Event) => void;
  }

  let { task, onimportancechange, onurgencychange, ondeadlinechange }: Props = $props();
</script>

<div class="fields">
  <label class="field">
    <span class="label">Importance</span>
    <select
      aria-label="Importance"
      value={task.importance}
      onchange={/* set importance */ onimportancechange}
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
      onchange={/* set urgency */ onurgencychange}
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
      onchange={/* set deadline */ ondeadlinechange}
    />
  </label>
</div>

<style>
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
