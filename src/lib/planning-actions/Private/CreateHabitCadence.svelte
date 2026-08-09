<script lang="ts">
  import type { Weekday } from '../../domain';
  import { WEEKDAYS } from './weekdays';

  interface Props {
    selectedDays: Weekday[];
    ontoggle: (day: Weekday, checked: boolean) => void;
  }

  let { selectedDays, ontoggle }: Props = $props();
</script>

<fieldset class="cadence">
  <legend>Cadence</legend>
  {#each WEEKDAYS as day (day.value)}
    <label>
      <input
        type="checkbox"
        checked={selectedDays.includes(day.value)}
        onchange={/* toggle day */ (event) =>
          ontoggle(day.value, (event.currentTarget as HTMLInputElement).checked)}
      />
      {day.label}
    </label>
  {/each}
</fieldset>

<style>
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
