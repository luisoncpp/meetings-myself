<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { t } from '../../../i18n';
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
  <Field label={t('library.importance')}>
    <Select
      aria-label={t('library.importance')}
      value={task.importance}
      onchange={/* set importance */ onimportancechange}
    >
      {#each CLASSIFICATION_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>
  <Field label={t('library.urgency')}>
    <Select
      aria-label={t('library.urgency')}
      value={task.urgency}
      onchange={/* set urgency */ onurgencychange}
    >
      {#each CLASSIFICATION_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>
  <Field label={t('library.deadline')}>
    <Input
      type="date"
      aria-label={t('library.deadline')}
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
