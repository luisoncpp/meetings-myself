<script lang="ts">
  import type { CheckInOutcome, PlanHabitView } from '../../../domain';
  import { t } from '../../../i18n';
  import { Card } from '../../../ui';
  import HabitList from './HabitList.svelte';

  interface Props {
    habits: PlanHabitView[];
    oncheckin: (habitId: string, outcome: CheckInOutcome) => void;
  }

  let { habits, oncheckin }: Props = $props();
</script>

<Card>
  <h2>{t('dailyPlan.habits')}</h2>
  {#if habits.length === 0}
    <p class="empty">{t('dailyPlan.habitsEmpty')}</p>
  {:else}
    <HabitList {habits} {oncheckin} />
  {/if}
</Card>

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }
</style>
