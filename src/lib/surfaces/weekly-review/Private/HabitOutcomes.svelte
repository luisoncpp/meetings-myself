<script lang="ts">
  import type { HabitSummary } from '../../../domain';
  import { t } from '../../../i18n';
  import OutcomeMark from './OutcomeMark.svelte';

  interface Props {
    habits: HabitSummary[];
  }

  let { habits }: Props = $props();

  function phrase(kind: 'done' | 'skipped' | 'missed', count: number): string {
    const form = count === 1 ? 'One' : 'Many';
    return t(`weeklyReview.${kind}${form}`, { count });
  }
</script>

<ul class="habits" aria-label={t('weeklyReview.habitsTable')}>
  {#each habits as habit (habit.title)}
    <li>
      <span class="name">{habit.title}</span>
      <span class="outcomes">
        {#if habit.done > 0}
          <span class="outcome done">
            <OutcomeMark kind="done" />
            {phrase('done', habit.done)}
          </span>
        {/if}
        {#if habit.skipped > 0}
          <span class="outcome">
            <OutcomeMark kind="skipped" />
            {phrase('skipped', habit.skipped)}
          </span>
        {/if}
        {#if habit.notCompleted > 0}
          <span class="outcome">
            <OutcomeMark kind="missed" />
            {phrase('missed', habit.notCompleted)}
          </span>
        {/if}
      </span>
    </li>
  {/each}
</ul>

<style>
  .habits {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    gap: var(--space-2) var(--space-4);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--color-hairline);
  }

  .name {
    color: var(--color-ink);
  }

  .outcomes {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    font-size: var(--text-label);
    font-variant-numeric: tabular-nums;
  }

  .outcome {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    color: var(--color-ink-muted);
  }

  .done {
    color: var(--color-done);
  }
</style>
