<script lang="ts">
  import type { Cadence, HabitView, Weekday } from '../../../domain';
  import { CreateHabitCadence } from '../../../planning-actions';
  import { Button, Field, ListRow, Select, StateFlag } from '../../../ui';
  import { STRENGTH_OPTIONS, WEEKDAYS } from './labels';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    habit: HabitView;
    store: LibraryStore;
  }

  let { habit, store }: Props = $props();

  function toggleArchive(): void {
    const end = { kind: 'habit' as const, id: habit.id };
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
      return WEEKDAYS.map((day) => day.value);
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
        <Field label="Habit Strength">
          <Select
            aria-label="Habit Strength"
            value={habit.strength}
            onchange={/* set strength */ onStrengthChange}
          >
            {#each STRENGTH_OPTIONS as option (option.value)}
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
          Pinned
        </label>
      {/if}
    </div>
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {habit.archived ? 'Restore' : 'Archive'}
    </Button>
  {/snippet}
</ListRow>

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
