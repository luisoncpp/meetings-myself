<script lang="ts">
  import type { Cadence, Weekday } from '../../../domain';
  import { Button } from '../../../ui';
  import { createButtonLabel, entityNameLabel } from './create-entity-copy';
  import CreateHabitCadence from './CreateHabitCadence.svelte';
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

  const nameLabel = $derived(entityNameLabel(kind));
  const submitLabel = $derived(createButtonLabel(kind));
  const habitReady = $derived(
    kind !== 'habit' || (title.trim() !== '' && selectedDays.length > 0),
  );
  const canSubmit = $derived(kind === 'habit' ? habitReady : title.trim() !== '');

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
    <span class="label">{nameLabel}</span>
    <input bind:value={title} aria-label={kind === 'habit' ? 'Habit name' : undefined} />
  </label>

  {#if kind === 'goal'}
    <label class="field">
      <span class="label">Target date</span>
      <input type="date" bind:value={targetDate} aria-label="Target date" />
    </label>
  {/if}

  {#if kind === 'habit'}
    <CreateHabitCadence selectedDays={selectedDays} ontoggle={toggleDay} />
  {/if}

  <div class="actions">
    <Button type="submit" variant="primary" disabled={!canSubmit}>{submitLabel}</Button>
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

  .actions {
    display: flex;
    gap: var(--space-2);
  }
</style>
