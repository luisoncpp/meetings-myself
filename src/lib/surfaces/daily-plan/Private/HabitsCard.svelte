<script lang="ts">
  import type { CheckInOutcome, PlanHabitView } from '../../../domain';
  import { t } from '../../../i18n';
  import HabitList from './HabitList.svelte';

  interface Props {
    habits: PlanHabitView[];
    oncheckin: (habitId: string, outcome: CheckInOutcome) => void;
  }

  let { habits, oncheckin }: Props = $props();
</script>

<section class="strip" aria-labelledby="habits-heading">
  <h2 id="habits-heading">{t('dailyPlan.habits')}</h2>
  {#if habits.length === 0}
    <p class="empty">{t('dailyPlan.habitsEmpty')}</p>
  {:else}
    <HabitList {habits} {oncheckin} />
  {/if}
</section>

<style>
  .strip {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  h2 {
    margin: 0;
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }
</style>
