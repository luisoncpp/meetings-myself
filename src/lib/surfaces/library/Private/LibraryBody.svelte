<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import type { TaskMenuController } from '../../../planning-actions';
  import ArchiveList from './ArchiveList.svelte';
  import EntityList from './EntityList.svelte';
  import LibraryDetail from './LibraryDetail.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';
  import RecurringTaskSection from './RecurringTaskSection.svelte';

  interface Props {
    view: LibraryView;
    store: LibraryStore;
    menu: TaskMenuController;
  }

  let { view, store, menu }: Props = $props();

  const KIND_OF_TAB = { tasks: 'task', habits: 'habit', goals: 'goal', values: 'value' } as const;
</script>

<!-- List on the left, details of the selected row on the right: no scrolling between them. -->
<div class="body" class:with-detail={store.selected !== null}>
  <section class="list" aria-label={t(`library.tabs.${store.tab}`)}>
    {#if store.tab === 'recurring'}
      <RecurringTaskSection {store} />
    {:else if store.tab === 'archive'}
      <ArchiveList {view} {store} {menu} />
    {:else}
      <EntityList kind={KIND_OF_TAB[store.tab]} {view} {store} {menu} />
    {/if}
  </section>
  {#if store.selected}
    <LibraryDetail selected={store.selected} {view} {store} />
  {/if}
</div>

<style>
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-4);
    align-items: start;
  }

  .with-detail {
    grid-template-columns: minmax(0, 1fr) minmax(16rem, 22rem);
  }

  .list {
    padding: var(--space-3);
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }
</style>
