<script lang="ts">
  import {
    PlanningActionBar,
    TaskMenu,
    TaskMenuController,
    bindPlanningActionsHost,
    type CreatePayload,
    type PlanningAction,
  } from '../../../planning-actions';
  import { t } from '../../../i18n';
  import { SurfaceLayout } from '../../../ui';
  import { WeeklyReviewStore } from './WeeklyReviewStore.svelte';
  import FocusBoard from './FocusBoard.svelte';
  import ReflectionEditor from './ReflectionEditor.svelte';
  import ReviewHistory from './ReviewHistory.svelte';
  import WeekNav from './WeekNav.svelte';

  let store = $state<WeeklyReviewStore>();
  let activeAction = $state<PlanningAction | null>(null);
  const menu = new TaskMenuController();

  async function handleToolbarCreate(
    reviewStore: WeeklyReviewStore,
    payload: CreatePayload,
  ): Promise<void> {
    await reviewStore.createEntity(payload);
    activeAction = null;
  }

  $effect(/* createLoadAndSaveOnLeave */ () => {
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
          <WeekNav week={view.week} store={activeStore} />
        </header>
        {#if activeStore.isHistorical}
          <p class="historical">{t('weeklyReview.historical')}</p>
        {/if}

        <FocusBoard {library} store={activeStore} {menu} />

        <PlanningActionBar
          active={activeAction}
          view={library}
          host={bindPlanningActionsHost(activeStore)}
          ontoggle={/* toggle */ (action) => (activeAction = activeAction === action ? null : action)}
          onclose={/* close */ () => (activeAction = null)}
          oncreate={/* create */ (payload) => void handleToolbarCreate(activeStore, payload)}
        />

        <ReflectionEditor
          value={activeStore.draftReflection}
          saveState={activeStore.saveState}
          oninput={/* edit */ (text) => activeStore.setReflection(text)}
          onblur={/* blur */ () => activeStore.onReflectionBlur()}
        />

        <ReviewHistory {view} />

        {#if activeStore.error}
          <p class="error" role="alert">{activeStore.error}</p>
        {/if}
      </section>
      <TaskMenu controller={menu} actions={activeStore.actions} />
    </SurfaceLayout>
  {/if}
{/if}

<style>
  .weekly-review {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .top {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-4);
  }

  h1 {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 600;
  }

  .historical {
    margin: 0;
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .loading {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .error {
    margin: 0;
    color: var(--color-overdue);
  }
</style>
