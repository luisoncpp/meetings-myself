<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { Field, Input, Select } from '../../../ui';
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
  <Field label="Importance">
    <Select
      aria-label="Importance"
      value={task.importance}
      onchange={/* set importance */ onimportancechange}
    >
      {#each CLASSIFICATION_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>
  <Field label="Urgency">
    <Select
      aria-label="Urgency"
      value={task.urgency}
      onchange={/* set urgency */ onurgencychange}
    >
      {#each CLASSIFICATION_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>
  <Field label="Deadline">
    <Input
      type="date"
      aria-label="Deadline"
      value={task.deadline ?? ''}
      onchange={/* set deadline */ ondeadlinechange}
    />
  </Field>
</div>

<style>
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }
</style>
