<script lang="ts">
  import { WeeklyReviewStore } from './WeeklyReviewStore.svelte';
  import NextWeekFocus from './NextWeekFocus.svelte';
  import PreviousReport from './PreviousReport.svelte';
  import ReflectionEditor from './ReflectionEditor.svelte';
  import ReportPath from './ReportPath.svelte';
  import ReviewActions from './ReviewActions.svelte';
  import SummarySection from './SummarySection.svelte';
  import WeekNav from './WeekNav.svelte';

  let store = $state<WeeklyReviewStore>();

  $effect(() => {
    const created = new WeeklyReviewStore();
    store = created;
    void created.load();
    return () => created.destroy();
  });
</script>

{#if store}
  {@const activeStore = store}
  {#if activeStore.loading && !activeStore.view}
    <p class="loading">Loading review…</p>
  {:else if activeStore.view && activeStore.library}
  {@const view = activeStore.view!}
  {@const library = activeStore.library!}
  <section class="weekly-review" aria-labelledby="review-week">
    <header class="top">
      <h1 id="review-week">{view.week}</h1>
      {#if activeStore.isHistorical}
        <p class="historical">Viewing a past week — not the current one.</p>
      {/if}
    </header>

    <WeekNav week={view.week} store={activeStore} />

    <ReviewActions store={activeStore} {library} focusWeek={activeStore.focusWeek} />

    <PreviousReport body={view.previousReport} />
    <SummarySection summary={view.summary} />
    <ReflectionEditor
      value={activeStore.draftReflection}
      saveState={activeStore.saveState}
      oninput={/* edit */ (text) => activeStore.setReflection(text)}
      onblur={/* blur */ () => activeStore.onReflectionBlur()}
    />
    <NextWeekFocus tasks={view.nextWeekFocus} />
    <ReportPath path={view.reportPath} />

    {#if activeStore.error}
      <p class="error" role="alert">{activeStore.error}</p>
    {/if}
  </section>
  {/if}
{/if}

<style>
  .weekly-review {
    padding: var(--space-6);
    max-width: 48rem;
  }

  .top {
    margin-bottom: var(--space-2);
  }

  h1 {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 600;
  }

  .historical {
    margin: var(--space-2) 0 0;
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .loading {
    margin: 0;
    padding: var(--space-6);
    color: var(--color-ink-muted);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
