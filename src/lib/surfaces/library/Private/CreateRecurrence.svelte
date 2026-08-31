<script lang="ts">
  import type { Recurrence, Weekday } from '../../../domain';
  import { localeStore, t } from '../../../i18n';
  import { Field, Input, Select } from '../../../ui';
  import { weekdayLabel, WEEKDAY_VALUES } from './labels';
  import { buildRecurrence, isRecurrenceValid } from './recurrence-label';

  interface Props {
    recurrence?: Recurrence;
    valid?: boolean;
  }

  let { recurrence = $bindable<Recurrence>({ kind: 'daily' }), valid = $bindable(/* valid= */ false) }: Props =
    $props();

  const kindOptions = $derived.by(() => {
    if (localeStore.locale) {
      return [
        { value: 'daily' as const, label: t('domain.recurrence.daily') },
        { value: 'weekdays' as const, label: t('domain.recurrence.weekdays') },
        { value: 'weekly' as const, label: t('domain.recurrence.weeklyKind') },
        { value: 'monthlyDay' as const, label: t('domain.recurrence.monthlyKind') },
      ];
    }
    return [];
  });

  const weekdays = $derived.by(() => {
    if (localeStore.locale) {
      return WEEKDAY_VALUES.map((value) => ({ value, label: weekdayLabel(value) }));
    }
    return [];
  });

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
  <Field label={t('library.recurrence')}>
    <Select aria-label={t('library.recurrenceKind')} value={kind} onchange={/* set kind */ onKindChange}>
      {#each kindOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>

  {#if kind === 'weekly'}
    <Field label={t('library.weekday')}>
      <Select aria-label={t('library.weekday')} value={weekday} onchange={/* set weekday */ onWeekdayChange}>
        {#each weekdays as day (day.value)}
          <option value={day.value}>{day.label}</option>
        {/each}
      </Select>
    </Field>
  {/if}

  {#if kind === 'monthlyDay'}
    <Field label={t('library.dayOfMonth')}>
      <Input
        type="number"
        value={monthlyDay}
        aria-label={t('library.dayOfMonth')}
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
