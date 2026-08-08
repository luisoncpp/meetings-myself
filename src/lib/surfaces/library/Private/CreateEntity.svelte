<script lang="ts">
  import type { Cadence, Weekday } from '../../../domain';
  import { Button } from '../../../ui';
  import { WEEKDAYS } from './labels';
  import type { EntityKind } from './associations';

  interface Props {
    kind: EntityKind;
    oncreate: (payload: CreatePayload) => void;
    oncancel: () => void;
  }

  export type CreatePayload =
    | { kind: 'value'; title: string }
    | { kind: 'goal'; title: string; targetDate: string | null }
    | { kind: 'habit'; title: string; cadence: Cadence }
    | { kind: 'task'; title: string };

  let { kind, oncreate, oncancel }: Props = $props();

  let title = $state('');
  let targetDate = $state('');
  let selectedDays = $state<Weekday[]>([]);

  const habitReady = $derived(
    kind !== 'habit' || title.trim() !== '' && selectedDays.length > 0,
  );

  function toggleDay(day: Weekday, checked: boolean): void {
    selectedDays = checked
      ? [...selectedDays, day]
      : selectedDays.filter((value) => value !== day);
  }

  function cadenceFromDays(days: Weekday[]): Cadence {
    if (days.length === 7) return { kind: 'everyDay' };
    return { kind: 'onWeekdays', days };
  }

  function submit(): void {
    const trimmed = title.trim();
    if (trimmed === '') return;

    if (kind === 'value') {
      oncreate({ kind: 'value', title: trimmed });
      return;
    }
    if (kind === 'goal') {
      oncreate({
        kind: 'goal',
        title: trimmed,
        targetDate: targetDate === '' ? null : targetDate,
      });
      return;
    }
    if (kind === 'habit') {
      oncreate({ kind: 'habit', title: trimmed, cadence: cadenceFromDays(selectedDays) });
      return;
    }
    oncreate({ kind: 'task', title: trimmed });
  }
</script>

<form class="create" onsubmit={/* create entity */ (event) => { event.preventDefault(); submit(); }}>
  <label class="field">
    <span class="label">
      {#if kind === 'habit'}Habit name{:else if kind === 'goal'}Goal name{:else if kind === 'task'}Task name{:else}Value name{/if}
    </span>
    <input
      bind:value={title}
      aria-label={kind === 'habit' ? 'Habit name' : undefined}
    />
  </label>

  {#if kind === 'goal'}
    <label class="field">
      <span class="label">Target date</span>
      <input type="date" bind:value={targetDate} aria-label="Target date" />
    </label>
  {/if}

  {#if kind === 'habit'}
    <fieldset class="cadence">
      <legend>Cadence</legend>
      {#each WEEKDAYS as day (day.value)}
        <label>
          <input
            type="checkbox"
            checked={selectedDays.includes(day.value)}
            onchange={/* toggle day */ (event) =>
              toggleDay(day.value, (event.currentTarget as HTMLInputElement).checked)}
          />
          {day.label}
        </label>
      {/each}
    </fieldset>
  {/if}

  <div class="actions">
    <Button type="submit" variant="primary" disabled={!habitReady && kind === 'habit' || title.trim() === ''}>
      {#if kind === 'habit'}Create habit{:else if kind === 'goal'}Create goal{:else if kind === 'task'}Create task{:else}Create value{/if}
    </Button>
    <Button variant="quiet" onclick={/* cancel */ oncancel}>Cancel</Button>
  </div>
</form>

<style>
  .create {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .field {
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

  .actions {
    display: flex;
    gap: var(--space-2);
  }
</style>
