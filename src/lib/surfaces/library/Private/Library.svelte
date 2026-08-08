<script lang="ts">
  import { Button } from '../../../ui';
  import AssociationEditor from './AssociationEditor.svelte';
  import EntitySection from './EntitySection.svelte';
  import type { CreatePayload } from './CreateEntity.svelte';
  import { LibraryStore } from './LibraryStore.svelte';
  import type { EntityKind } from './associations';
  import WeeklyFocusPanel from './WeeklyFocusPanel.svelte';

  let store = $state<LibraryStore>();
  let creating = $state<EntityKind | null>(null);
  let showLink = $state(false);
  let showFocus = $state(false);
  let selectedGoalId = $state<string | null>(null);

  $effect(() => {
    if (store) return;
    store = new LibraryStore();
    void store.load();
  });

  function startCreate(kind: EntityKind): void {
    creating = kind;
  }

  function cancelCreate(): void {
    creating = null;
  }

  async function handleCreate(payload: CreatePayload): Promise<void> {
    if (!store) return;
    if (payload.kind === 'value') await store.createValue(payload.title);
    if (payload.kind === 'goal') await store.createGoal(payload.title, payload.targetDate);
    if (payload.kind === 'habit') await store.createHabit(payload.title, payload.cadence);
    if (payload.kind === 'task') await store.createTask(payload.title);
    creating = null;
  }

  function markSelectedAchieved(): void {
    if (!store || !selectedGoalId) return;
    const goal = store.view?.goals.find((item) => item.id === selectedGoalId);
    if (!goal || goal.archived) return;
    if (goal.achieved) {
      void store.unachieveGoal(goal.id);
      return;
    }
    void store.achieveGoal(goal.id);
  }
</script>

{#if store?.view}
  {@const activeStore = store}
  {@const view = activeStore.view!}
  <section class="library" aria-labelledby="library-heading">
    <header class="top">
      <h1 id="library-heading">Library</h1>
      <label class="archived-toggle">
        <input
          type="checkbox"
          role="switch"
          checked={activeStore.includeArchived}
          onchange={/* toggle archived */ (event) =>
            void activeStore.setIncludeArchived((event.currentTarget as HTMLInputElement).checked)}
        />
        Show archived
      </label>
    </header>

    <div class="toolbar">
      <Button variant="primary" onclick={/* new goal */ () => startCreate('goal')}>New goal</Button>
      <Button variant="secondary" onclick={/* mark achieved */ markSelectedAchieved}>
        Mark achieved
      </Button>
      <Button variant="secondary" onclick={/* open link editor */ () => (showLink = !showLink)}>
        Link
      </Button>
      <Button variant="secondary" onclick={/* open weekly focus */ () => (showFocus = !showFocus)}>
        Weekly focus
      </Button>
    </div>

    {#if showLink}
      <AssociationEditor {view} store={activeStore} onclose={/* close */ () => (showLink = false)} />
    {/if}

    {#if showFocus}
      <WeeklyFocusPanel
        {view}
        store={activeStore}
        week={activeStore.week}
        onclose={/* close */ () => (showFocus = false)}
      />
    {/if}

    <EntitySection
      kind="value"
      title="Values"
      emptyLabel="No values yet."
      store={activeStore}
      {creating}
      onstartCreate={startCreate}
      oncreate={/* create */ (payload) => void handleCreate(payload)}
      oncancelCreate={cancelCreate}
    />

    <EntitySection
      kind="goal"
      title="Goals"
      emptyLabel="No goals yet."
      store={activeStore}
      {selectedGoalId}
      onselectGoal={/* select */ (goalId) => (selectedGoalId = goalId)}
      {creating}
      onstartCreate={startCreate}
      oncreate={/* create */ (payload) => void handleCreate(payload)}
      oncancelCreate={cancelCreate}
    />

    <EntitySection
      kind="habit"
      title="Habits"
      emptyLabel="No habits yet."
      store={activeStore}
      {creating}
      onstartCreate={startCreate}
      oncreate={/* create */ (payload) => void handleCreate(payload)}
      oncancelCreate={cancelCreate}
    />

    <EntitySection
      kind="task"
      title="Tasks"
      emptyLabel="No tasks yet."
      store={activeStore}
      {creating}
      onstartCreate={startCreate}
      oncreate={/* create */ (payload) => void handleCreate(payload)}
      oncancelCreate={cancelCreate}
    />

    {#if activeStore.error}
      <p class="error" role="alert">{activeStore.error}</p>
    {/if}
  </section>
{/if}

<style>
  .library {
    padding: var(--space-6);
    max-width: 48rem;
  }

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

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
