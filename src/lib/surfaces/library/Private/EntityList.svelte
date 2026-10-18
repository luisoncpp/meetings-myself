<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { CreateEntity, type EntityKind, type TaskMenuController } from '../../../planning-actions';
  import { Button } from '../../../ui';
  import EntityRows from './EntityRows.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import TaskRows from './TaskRows.svelte';

  interface Props {
    kind: EntityKind;
    view: LibraryView;
    store: LibraryStore;
    menu: TaskMenuController;
  }

  let { kind, view, store, menu }: Props = $props();

  let creating = $state(false);

  const tasks = $derived(view.tasks.filter((row) => !row.archived));
  const others = $derived(
    { task: [], habit: view.habits, goal: view.goals, value: view.values }[kind].filter(
      (row) => !row.archived,
    ),
  );
  const empty = $derived(kind === 'task' ? tasks.length === 0 : others.length === 0);
</script>

<div class="toolbar">
  <Button variant="quiet" pressed={creating} onclick={/* toggle create form */ () => (creating = !creating)}>
    + {t(`library.new.${kind}`)}
  </Button>
</div>
{#if creating}
  <CreateEntity
    {kind}
    oncreate={/* create */ (payload) => {
      creating = false;
      void store.createEntity(payload);
    }}
    oncancel={/* cancel */ () => (creating = false)}
  />
{/if}

{#if empty}
  <p class="empty">{t(`library.empty.${kind}`)}</p>
{/if}
<ul class="rows">
  {#if kind === 'task'}
    <TaskRows {tasks} {store} {menu} />
  {:else}
    <EntityRows {kind} rows={others} {store} />
  {/if}
</ul>

<style>
  .toolbar {
    margin-bottom: var(--space-2);
  }

  .empty {
    margin: var(--space-2);
    color: var(--color-ink-muted);
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
