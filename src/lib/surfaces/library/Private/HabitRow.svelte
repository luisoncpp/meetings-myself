<script lang="ts">
  import type { Cadence, HabitView, LibraryView, Weekday } from '../../../domain';
  import { CreateHabitCadence, LinkModal } from '../../../planning-actions';
  import { localeStore, t } from '../../../i18n';
  import { Button, Field, ListRow, Select, StateFlag } from '../../../ui';
  import AssociationTags from './AssociationTags.svelte';
  import { STRENGTH_VALUES, strengthLabel, WEEKDAY_VALUES } from './labels';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    habit: HabitView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { habit, view, store }: Props = $props();

  let linkModalOpen = $state(false);

  const strengthOptions = $derived.by(() => {
    if (localeStore.locale) {
      return STRENGTH_VALUES.map((value) => ({ value, label: strengthLabel(value) }));
    }
    return [];
  });

  const end = $derived({ kind: 'habit' as const, id: habit.id });

  function toggleArchive(): void {
    if (habit.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }

  function onStrengthChange(event: Event): void {
    const select = event.currentTarget as HTMLSelectElement;
    void store.setHabitStrength(habit.id, select.value as HabitView['strength']);
  }

  function onPinnedChange(event: Event): void {
    const input = event.currentTarget as HTMLInputElement;
    void store.setHabitPinned(habit.id, input.checked);
  }

  function onWeekdayToggle(day: Weekday, checked: boolean): void {
    const days = currentWeekdays(habit.cadence);
    const next = checked ? [...days, day] : days.filter((d) => d !== day);
    const cadence: Cadence =
      next.length === 7 ? { kind: 'everyDay' } : { kind: 'onWeekdays', days: next };
    void store.setHabitCadence(habit.id, cadence);
  }

  function currentWeekdays(cadence: Cadence): Weekday[] {
    if (cadence.kind === 'everyDay') {
      return [...WEEKDAY_VALUES];
    }
    return cadence.days;
  }
</script>

<ListRow muted={habit.archived}>
  <div class="habit">
    <span class="title">{habit.title}</span>
    {#if habit.archived}
      <StateFlag kind="archived" />
    {:else}
      <AssociationTags
        {end}
        {view}
        onunlink={(id) => void store.unlink(id)}
        onopenLink={() => (linkModalOpen = true)}
      />
      <Field label={t('library.habitStrength')}>
        <Select
          aria-label={t('library.habitStrength')}
          value={habit.strength}
          onchange={/* set strength */ onStrengthChange}
        >
          {#each strengthOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </Select>
      </Field>
      <CreateHabitCadence
        selectedDays={currentWeekdays(habit.cadence)}
        ontoggle={/* toggle weekday */ onWeekdayToggle}
      />
      <label class="pinned">
        <input
          type="checkbox"
          checked={habit.pinned}
          onchange={/* toggle pinned */ onPinnedChange}
        />
        {t('library.pinned')}
      </label>
    {/if}
  </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {habit.archived ? t('common.restore') : t('common.archive')}
    </Button>
  {/snippet}
</ListRow>

{#if linkModalOpen}
  <LinkModal
    fromEnd={end}
    fromTitle={habit.title}
    {view}
    onlink={(toEnd) => store.link(end, toEnd)}
    onclose={() => (linkModalOpen = false)}
  />
{/if}

<style>
  .habit {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .title {
    font-weight: 500;
  }

  .pinned {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
</style>

