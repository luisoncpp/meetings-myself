<script lang="ts">
  import type { Cadence, HabitView, LibraryView, Weekday } from '../../../domain';
  import { CreateHabitCadence } from '../../../planning-actions';
  import { localeStore, t } from '../../../i18n';
  import { Button, Field, Select } from '../../../ui';
  import DetailLinks from './DetailLinks.svelte';
  import { STRENGTH_VALUES, strengthLabel, WEEKDAY_VALUES } from './labels';
  import type { LibraryStore } from './LibraryStore.svelte';
  import RenameField from './RenameField.svelte';

  interface Props {
    habit: HabitView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { habit, view, store }: Props = $props();

  const end = $derived({ kind: 'habit' as const, id: habit.id });
  const strengthOptions = $derived.by(() => {
    if (!localeStore.locale) return [];
    return STRENGTH_VALUES.map((value) => ({ value, label: strengthLabel(value) }));
  });
  const days = $derived(habit.cadence.kind === 'everyDay' ? [...WEEKDAY_VALUES] : habit.cadence.days);

  function onWeekdayToggle(day: Weekday, checked: boolean): void {
    const next = checked ? [...days, day] : days.filter((d) => d !== day);
    const cadence: Cadence = next.length === 7 ? { kind: 'everyDay' } : { kind: 'onWeekdays', days: next };
    void store.setHabitCadence(habit.id, cadence);
  }

  function onStrengthChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value as HabitView['strength'];
    void store.setHabitStrength(habit.id, value);
  }
</script>

{#if !habit.archived}
  <RenameField
    title={habit.title}
    label={t('library.habitTitle')}
    onrename={/* rename */ (title) => void store.renameHabit(habit.id, title)}
  />
  <label class="checkbox-label" title={habit.pinned ? t('library.pinnedHint') : t('library.unpinnedHint')}>
    <input
      type="checkbox"
      role="switch"
      checked={habit.pinned}
      onchange={/* toggle daily */ (event) =>
        void store.setHabitPinned(habit.id, (event.currentTarget as HTMLInputElement).checked)}
    />
    {t('library.showInDailyPlan')}
  </label>
  <Field label={t('library.habitStrength')}>
    <Select aria-label={t('library.habitStrength')} value={habit.strength} onchange={/* set strength */ onStrengthChange}>
      {#each strengthOptions as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </Select>
  </Field>
  <CreateHabitCadence selectedDays={days} ontoggle={/* toggle weekday */ onWeekdayToggle} />
  <DetailLinks {end} title={habit.title} {view} {store} />
{/if}
<div>
  <Button variant="quiet" onclick={/* archive or restore */ () => void store.setArchived(end, !habit.archived)}>
    {habit.archived ? t('common.restore') : t('common.archive')}
  </Button>
</div>
