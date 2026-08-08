<script lang="ts">
  import { Button, Card } from '../../../ui';
  import type { EntityKind } from './associations';
  import CreateEntity, { type CreatePayload } from './CreateEntity.svelte';
  import EntityKindList from './EntityKindList.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    kind: EntityKind;
    title: string;
    emptyLabel: string;
    store: LibraryStore;
    selectedGoalId?: string | null;
    onselectGoal?: (goalId: string) => void;
    creating: EntityKind | null;
    onstartCreate: (kind: EntityKind) => void;
    oncreate: (payload: CreatePayload) => void;
    oncancelCreate: () => void;
  }

  let {
    kind,
    title,
    emptyLabel,
    store,
    selectedGoalId = null,
    onselectGoal,
    creating,
    onstartCreate,
    oncreate,
    oncancelCreate,
  }: Props = $props();

  const view = $derived(store.view);
  const count = $derived.by(() => {
    if (!view) return 0;
    if (kind === 'value') return view.values.length;
    if (kind === 'goal') return view.goals.length;
    if (kind === 'habit') return view.habits.length;
    return view.tasks.length;
  });

  const newLabel = $derived(
    kind === 'value'
      ? 'Add value'
      : kind === 'goal'
        ? 'Add goal'
        : kind === 'habit'
          ? 'New habit'
          : 'Add task',
  );
</script>

<section class="section" aria-label={title}>
  <Card>
    <div class="header">
      <h2>{title}</h2>
      <Button variant="quiet" onclick={/* start create */ () => onstartCreate(kind)}>
        {newLabel}
      </Button>
    </div>

    {#if creating === kind}
      <CreateEntity {kind} {oncreate} oncancel={/* cancel */ oncancelCreate} />
    {/if}

    {#if count === 0}
      <p class="empty">{emptyLabel}</p>
    {:else if view}
      <div class="list">
        <EntityKindList {kind} {view} {store} {selectedGoalId} {onselectGoal} />
      </div>
    {/if}
  </Card>
</section>

<style>
  .section {
    margin-bottom: var(--space-4);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }

  h2 {
    margin: 0;
    font-size: var(--text-title);
    font-weight: 600;
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
</style>
