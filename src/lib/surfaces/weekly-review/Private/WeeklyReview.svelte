<script lang="ts">
  import {
    PlanningActionBar,
    bindPlanningActionsHost,
    type CreatePayload,
    type PlanningAction,
  } from '../../../planning-actions';
  import { t } from '../../../i18n';
  import { SurfaceLayout } from '../../../ui';
  import { WeeklyReviewStore } from './WeeklyReviewStore.svelte';
  import NextWeekFocus from './NextWeekFocus.svelte';
  import PreviousReport from './PreviousReport.svelte';
  import ReflectionEditor from './ReflectionEditor.svelte';
  import ReportPath from './ReportPath.svelte';
  import SummarySection from './SummarySection.svelte';
  import WeekNav from './WeekNav.svelte';

  let store = $state<WeeklyReviewStore>();
  let activeAction = $state<PlanningAction | null>(null);

  function toggleAction(action: PlanningAction): void {
    activeAction = activeAction === action ? null : action;
  }

  function closeAction(): void {
    activeAction = null;
  }

  async function handleToolbarCreate(
    reviewStore: WeeklyReviewStore,
    payload: CreatePayload,
  ): Promise<void> {
    await reviewStore.createEntity(payload);
    activeAction = null;
  }

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
    <SurfaceLayout>
      <p class="loading">{t('weeklyReview.loading')}</p>
    </SurfaceLayout>
  {:else if activeStore.view && activeStore.library}
    {@const view = activeStore.view}
    {@const library = activeStore.library}
    <SurfaceLayout aria-labelledby="review-week">
      <section class="weekly-review">
        <header class="top">
          <h1 id="review-week">{view.week}</h1>
          {#if activeStore.isHistorical}
            <p class="historical">{t('weeklyReview.historical')}</p>
          {/if}
        </header>

        <WeekNav week={view.week} store={activeStore} />

        <PlanningActionBar
          active={activeAction}
          view={library}
          host={bindPlanningActionsHost(activeStore)}
          week={activeStore.focusWeek}
          ontoggle={toggleAction}
          onclose={closeAction}
          oncreate={/* create */ (payload) => void handleToolbarCreate(activeStore, payload)}
        />

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
    </SurfaceLayout>
  {/if}
{/if}

<style>
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
    color: var(--color-ink-muted);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
