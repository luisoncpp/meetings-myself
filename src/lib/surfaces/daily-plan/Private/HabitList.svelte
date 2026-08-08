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
    <li class="habit">
      <div class="heading">
        <span class="title">{habit.title}</span>
        {#if habit.unpinned}
          <StateFlag kind="unpinned" />
        {/if}
      </div>
      <CheckInControl
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
    flex-direction: column;
    gap: var(--space-4);
  }

  .heading {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .title {
    font-weight: 500;
  }
</style>
