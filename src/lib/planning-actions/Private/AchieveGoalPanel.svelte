<script lang="ts">
  import type { GoalView } from '../../domain';
  import { Button, Field, Select } from '../../ui';
  import type { PlanningActionsHost } from './planning-actions-host';

  interface Props {
    goals: GoalView[];
    host: PlanningActionsHost;
    onclose: () => void;
  }

  let { goals, host, onclose }: Props = $props();

  let selectedGoalId = $state('');

  const openGoals = $derived(goals.filter((goal) => !goal.archived && !goal.achieved));

  async function markAchieved(): Promise<void> {
    if (selectedGoalId === '') return;
    await host.achieveGoal(selectedGoalId);
    selectedGoalId = '';
    onclose();
  }
</script>

<div class="panel">
  <Field label="Goal">
    <Select bind:value={selectedGoalId} aria-label="Goal to mark achieved">
      <option value="">Select…</option>
      {#each openGoals as goal (goal.id)}
        <option value={goal.id}>{goal.title}</option>
      {/each}
    </Select>
  </Field>
  <Button variant="primary" disabled={selectedGoalId === ''} onclick={/* achieve */ markAchieved}>
    Mark achieved
  </Button>
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
  }
</style>
