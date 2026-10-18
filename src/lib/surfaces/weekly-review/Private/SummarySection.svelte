<script lang="ts">
  import type { WeeklySummary } from '../../../domain';
  import { t } from '../../../i18n';
  import HabitOutcomes from './HabitOutcomes.svelte';

  interface Props {
    summary: WeeklySummary;
  }

  let { summary }: Props = $props();
</script>

<section class="summary" aria-labelledby="summary-heading">
  <h2 id="summary-heading">
    <span>{t('weeklyReview.thisWeek')}</span>
    <span class="still-open">{t('weeklyReview.stillOpen', { count: summary.stillOpen })}</span>
  </h2>

  {#if summary.completed.length > 0}
    <h3>{t('weeklyReview.completed')}</h3>
    <ul>
      {#each summary.completed as title (title)}
        <li>{title}</li>
      {/each}
    </ul>
  {/if}

  {#if summary.overdue.length > 0}
    <h3>{t('weeklyReview.overdue')}</h3>
    <ul>
      {#each summary.overdue as title (title)}
        <li>{title}</li>
      {/each}
    </ul>
  {/if}

  {#if summary.habits.length > 0}
    <h3>{t('weeklyReview.habits')}</h3>
    <HabitOutcomes habits={summary.habits} />
  {/if}

  {#if summary.goalsAchieved.length > 0}
    <h3>{t('weeklyReview.goalsAchieved')}</h3>
    <ul>
      {#each summary.goalsAchieved as title (title)}
        <li>{title}</li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .summary {
    margin-bottom: var(--space-4);
  }

  h2 {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-3);
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
    text-wrap: balance;
  }

  .still-open {
    font-size: var(--text-label);
    font-weight: 500;
    color: var(--color-ink-muted);
  }

  h3 {
    margin: var(--space-3) 0 var(--space-2);
    font-size: var(--text-body);
    font-weight: 600;
  }

  ul {
    margin: 0;
    padding-left: var(--space-5);
  }
</style>
