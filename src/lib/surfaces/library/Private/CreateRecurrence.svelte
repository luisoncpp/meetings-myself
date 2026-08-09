<script lang="ts">
  import type { Recurrence, Weekday } from '../../../domain';
  import { Field, Input, Select } from '../../../ui';
  import { WEEKDAYS } from './labels';
  import { buildRecurrence, isRecurrenceValid } from './recurrence-label';

  interface Props {
    recurrence?: Recurrence;
    valid?: boolean;
  }

  let { recurrence = $bindable<Recurrence>({ kind: 'daily' }), valid = $bindable(/* valid= */ false) }: Props =
    $props();

  const KIND_OPTIONS: { value: Recurrence['kind']; label: string }[] = [
    { value: 'daily', label: 'Daily' },
    { value: 'weekdays', label: 'Weekdays' },
    { value: 'weekly', label: 'Weekly' },
    { value: 'monthlyDay', label: 'Monthly (day of month)' },
  ];

  let kind = $state<Recurrence['kind']>('daily');
  let weekday = $state<Weekday>('mon');
  let monthlyDay = $state('1');

  $effect(() => {
    recurrence = buildRecurrence(kind, weekday, monthlyDay);
    valid = isRecurrenceValid(kind, weekday, monthlyDay);
  });

  function onKindChange(event: Event): void {
    kind = (event.currentTarget as HTMLSelectElement).value as Recurrence['kind'];
  }

  function onWeekdayChange(event: Event): void {
    weekday = (event.currentTarget as HTMLSelectElement).value as Weekday;
  }

  function onMonthlyDayInput(event: Event): void {
    monthlyDay = (event.currentTarget as HTMLInputElement).value;
  }
</script>

<div class="recurrence">
  <Field label="Recurrence">
    <Select aria-label="Recurrence kind" value={kind} onchange={/* set kind */ onKindChange}>
      {#each KIND_OPTIONS as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>

  {#if kind === 'weekly'}
    <Field label="Weekday">
      <Select aria-label="Weekday" value={weekday} onchange={/* set weekday */ onWeekdayChange}>
        {#each WEEKDAYS as day (day.value)}
          <option value={day.value}>{day.label}</option>
        {/each}
      </Select>
    </Field>
  {/if}

  {#if kind === 'monthlyDay'}
    <Field label="Day of month">
      <Input
        type="number"
        value={monthlyDay}
        aria-label="Day of month"
        oninput={/* set day */ onMonthlyDayInput}
      />
    </Field>
  {/if}
</div>

<style>
  .recurrence {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }
</style>
