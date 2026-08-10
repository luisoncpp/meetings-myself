<script lang="ts">
  import {
    PlanningActionBar,
    bindPlanningActionsHost,
    type CreatePayload,
    type EntityKind,
    type PlanningAction,
  } from '../../../planning-actions';
  import { t } from '../../../i18n';
  import { SurfaceLayout } from '../../../ui';
  import EntitySection from './EntitySection.svelte';
  import { LibraryStore } from './LibraryStore.svelte';
  import RecurringTaskSection from './RecurringTaskSection.svelte';

  let store = $state<LibraryStore>();
  let creating = $state<EntityKind | null>(null);
  let activeAction = $state<PlanningAction | null>(null);

  function toggleAction(action: PlanningAction): void {
    activeAction = activeAction === action ? null : action;
    if (activeAction !== null) creating = null;
  }

  $effect(() => {
    if (store) return;
    store = new LibraryStore();
    void store.load();
  });

  function closeAction(): void {
    activeAction = null;
  }

  function startCreate(kind: EntityKind): void {
    creating = kind;
    activeAction = null;
  }

  function cancelCreate(): void {
    creating = null;
  }

  async function handleCreate(payload: CreatePayload): Promise<void> {
    if (!store) return;
    if (payload.kind === 'value') await store.createValue(payload.title);
    if (payload.kind === 'goal') await store.createGoal(payload.title, payload.targetDate);
    if (payload.kind === 'habit') await store.createHabit(payload.title, payload.cadence);
    if (payload.kind === 'task') await store.createTask(payload.title, payload.oneOff);
    creating = null;
    activeAction = null;
  }

  async function handleToolbarGoalCreate(payload: CreatePayload): Promise<void> {
    if (payload.kind !== 'goal') return;
    await handleCreate(payload);
  }

</script>

{#if store}
  {@const activeStore = store}
  {#if activeStore.loading && !activeStore.view}
    <SurfaceLayout>
      <p class="loading">{t('library.loading')}</p>
    </SurfaceLayout>
  {:else if activeStore.view}
    {@const view = activeStore.view}
    <SurfaceLayout aria-labelledby="library-heading">
      <section class="library">
        <header class="top">
          <h1 id="library-heading">{t('library.title')}</h1>
          <label class="archived-toggle">
            <input
              type="checkbox"
              role="switch"
              checked={activeStore.includeArchived}
              onchange={/* toggle archived */ (event) =>
                void activeStore.setIncludeArchived((event.currentTarget as HTMLInputElement).checked)}
            />
            {t('library.showArchived')}
          </label>
        </header>

        <PlanningActionBar
          active={activeAction}
          {view}
          host={bindPlanningActionsHost(activeStore)}
          week={activeStore.week}
          ontoggle={toggleAction}
          onclose={closeAction}
          oncreateGoal={/* create goal */ (payload) => void handleToolbarGoalCreate(payload)}
        />

        <EntitySection
          kind="value"
          title={t('library.values')}
          emptyLabel={t('library.noValues')}
          store={activeStore}
          {creating}
          onstartCreate={startCreate}
          oncreate={/* create */ (payload) => void handleCreate(payload)}
          oncancelCreate={cancelCreate}
        />

        <EntitySection
          kind="goal"
          title={t('library.goals')}
          emptyLabel={t('library.noGoals')}
          store={activeStore}
          {creating}
          onstartCreate={startCreate}
          oncreate={/* create */ (payload) => void handleCreate(payload)}
          oncancelCreate={cancelCreate}
        />

        <EntitySection
          kind="habit"
          title={t('library.habits')}
          emptyLabel={t('library.noHabits')}
          store={activeStore}
          {creating}
          onstartCreate={startCreate}
          oncreate={/* create */ (payload) => void handleCreate(payload)}
          oncancelCreate={cancelCreate}
        />

        <EntitySection
          kind="task"
          title={t('library.tasks')}
          emptyLabel={t('library.noTasks')}
          store={activeStore}
          {creating}
          onstartCreate={startCreate}
          oncreate={/* create */ (payload) => void handleCreate(payload)}
          oncancelCreate={cancelCreate}
        />

        <RecurringTaskSection store={activeStore} />

        {#if activeStore.error}
          <p class="error" role="alert">{activeStore.error}</p>
        {/if}
      </section>
    </SurfaceLayout>
  {/if}
{/if}

<style>
  .top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  h1 {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 600;
  }

  .archived-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-body);
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
