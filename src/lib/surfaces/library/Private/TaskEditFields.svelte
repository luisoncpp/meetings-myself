<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { localeStore, t } from '../../../i18n';
  import { Field, Input, Select } from '../../../ui';
  import { CLASSIFICATION_VALUES, classificationLabel } from './labels';

  interface Props {
    task: TaskView;
    onimportancechange: (event: Event) => void;
    onurgencychange: (event: Event) => void;
    ondeadlinechange: (event: Event) => void;
  }

  let { task, onimportancechange, onurgencychange, ondeadlinechange }: Props = $props();

  const classificationOptions = $derived.by(() => {
    if (localeStore.locale) {
      return CLASSIFICATION_VALUES.map((value) => ({ value, label: classificationLabel(value) }));
    }
    return [];
  });
</script>

<div class="fields">
  <Field label={t('library.importance')}>
    <Select
      aria-label={t('library.importance')}
      value={task.importance}
      onchange={/* set importance */ onimportancechange}
    >
      {#each classificationOptions as option (option.value)}
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
      {#each classificationOptions as option (option.value)}
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
