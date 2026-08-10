<script lang="ts">
  import type { WeeklySummary } from '../../../domain';
  import { t } from '../../../i18n';

  interface Props {
    summary: WeeklySummary;
  }

  let { summary }: Props = $props();
</script>

<section class="summary" aria-labelledby="summary-heading">
  <h2 id="summary-heading">{t('weeklyReview.thisWeek')}</h2>

  {#if summary.completed.length > 0}
    <h3>{t('weeklyReview.completed')}</h3>
    <ul>
      {#each summary.completed as title (title)}
        <li>{title}</li>
      {/each}
    </ul>
  {/if}

  <p class="count">{t('weeklyReview.stillOpen', { count: summary.stillOpen })}</p>

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
    <table aria-label={t('weeklyReview.habitsTable')}>
      <thead>
        <tr>
          <th scope="col">{t('weeklyReview.habitColumn')}</th>
          <th scope="col">{t('weeklyReview.doneColumn')}</th>
          <th scope="col">{t('weeklyReview.skippedColumn')}</th>
          <th scope="col">{t('weeklyReview.notCompletedColumn')}</th>
        </tr>
      </thead>
      <tbody>
        {#each summary.habits as habit (habit.title)}
          <tr>
            <td>{habit.title}</td>
            <td>{habit.done}</td>
            <td>{habit.skipped}</td>
            <td>{habit.notCompleted}</td>
          </tr>
        {/each}
      </tbody>
    </table>
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
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
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

  .count {
    margin: var(--space-2) 0;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--text-body);
  }

  th,
  td {
    text-align: left;
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--color-hairline);
  }

  th {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
    font-weight: 500;
  }
</style>
