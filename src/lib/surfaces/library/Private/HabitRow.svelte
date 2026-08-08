<script lang="ts">
  import type { Cadence, HabitView, Weekday } from '../../../domain';
  import { Button, ListRow, StateFlag } from '../../../ui';
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

  function isDayChecked(day: Weekday): boolean {
    return currentWeekdays(habit.cadence).includes(day);
  }
</script>

<ListRow muted={habit.archived}>
  <div class="habit">
      <span class="title">{habit.title}</span>
      {#if habit.archived}
        <StateFlag kind="archived" />
      {:else}
        <label class="field">
          <span class="label">Habit Strength</span>
          <select
            aria-label="Habit Strength"
            value={habit.strength}
            onchange={/* set strength */ onStrengthChange}
          >
            {#each STRENGTH_OPTIONS as option (option.value)}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
        <fieldset class="cadence">
          <legend>Cadence</legend>
          {#each WEEKDAYS as day (day.value)}
            <label>
              <input
                type="checkbox"
                checked={isDayChecked(day.value)}
                onchange={/* toggle weekday */ (event) =>
                  onWeekdayToggle(day.value, (event.currentTarget as HTMLInputElement).checked)}
              />
              {day.label}
            </label>
          {/each}
        </fieldset>
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

  .field,
  .pinned {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .label {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .cadence {
    border: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  legend {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
    margin-bottom: var(--space-1);
  }
</style>
