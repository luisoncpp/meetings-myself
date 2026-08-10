<script lang="ts">
  import { t } from '../../../i18n';
  import { Button } from '../../../ui';
  import type { WeeklyReviewStore } from './WeeklyReviewStore.svelte';
  import { nextWeek, prevWeek } from './week-nav';

  interface Props {
    week: string;
    store: WeeklyReviewStore;
  }

  let { week, store }: Props = $props();
</script>

<nav class="week-nav" aria-label={t('weeklyReview.weekNav')}>
  <Button variant="quiet" onclick={/* previous week */ () => void store.openWeek(prevWeek(week))}>
    ← {prevWeek(week)}
  </Button>
  <Button variant="quiet" onclick={/* next week */ () => void store.openWeek(nextWeek(week))}>
    {nextWeek(week)} →
  </Button>
</nav>

<style>
  .week-nav {
    display: flex;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }
</style>
