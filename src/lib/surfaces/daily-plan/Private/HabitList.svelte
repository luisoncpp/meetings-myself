<script lang="ts">
  import type { CheckInOutcome, PlanHabitView } from '../../../domain';
  import { CheckInControl, StateFlag } from '../../../ui';

  interface Props {
    habits: PlanHabitView[];
    oncheckin: (habitId: string, outcome: CheckInOutcome) => void;
  }

  let { habits, oncheckin }: Props = $props();
</script>

<ul class="habits">
  {#each habits as habit (habit.id)}
    <li class="habit" class:checked={habit.outcome !== null}>
      <span class="title">{habit.title}</span>
      {#if habit.unpinned}
        <StateFlag kind="unpinned" />
      {/if}
      <CheckInControl
        compact
        value={habit.outcome}
        label={habit.title}
        onchange={/* record outcome */ (outcome) => oncheckin(habit.id, outcome)}
      />
    </li>
  {/each}
</ul>

<style>
  .habits {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .habit {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) var(--space-1) var(--space-1) var(--space-3);
    border-radius: var(--radius-pill);
    background: var(--color-lift);
  }

  .checked .title {
    color: var(--color-ink-muted);
  }
</style>
