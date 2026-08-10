<script lang="ts">
  import type { Recurrence } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button, Field, Input } from '../../../ui';
  import CreateRecurrence from './CreateRecurrence.svelte';

  interface Props {
    oncreate: (payload: { title: string; recurrence: Recurrence }) => void;
    oncancel: () => void;
  }

  let { oncreate, oncancel }: Props = $props();

  let title = $state('');
  let recurrence = $state<Recurrence>({ kind: 'daily' });
  let recurrenceValid = $state(/* valid= */ true);

  const canSubmit = $derived(title.trim() !== '' && recurrenceValid);

  function submit(): void {
    const trimmed = title.trim();
    if (trimmed === '' || !recurrenceValid) return;
    oncreate({ title: trimmed, recurrence });
  }
</script>

<form
  class="create"
  onsubmit={/* create recurring task */ (event) => {
    event.preventDefault();
    submit();
  }}
>
  <Field label={t('library.fieldTitle')}>
    <Input bind:value={title} aria-label={t('library.recurringTaskTitle')} />
  </Field>

  <CreateRecurrence bind:recurrence bind:valid={recurrenceValid} />

  <div class="actions">
    <Button type="submit" variant="primary" disabled={!canSubmit}>{t('common.create')}</Button>
    <Button variant="quiet" onclick={/* cancel */ oncancel}>{t('common.cancel')}</Button>
  </div>
</form>

<style>
  .create {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .actions {
    display: flex;
    gap: var(--space-2);
  }
</style>
