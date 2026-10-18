<script lang="ts">
  import { t } from '../../../i18n';
  import { TaskMenu, TaskMenuController } from '../../../planning-actions';
  import { SurfaceLayout, TabStrip } from '../../../ui';
  import LibraryBody from './LibraryBody.svelte';
  import { LibraryStore, type LibraryTab } from './LibraryStore.svelte';

  let store = $state<LibraryStore>();
  const menu = new TaskMenuController();

  const TABS: LibraryTab[] = ['tasks', 'habits', 'goals', 'values', 'recurring', 'archive'];
  const tabs = $derived(TABS.map((id) => ({ id, label: t(`library.tabs.${id}`) })));

  $effect(/* createAndLoadStore */ () => {
    if (store) return;
    store = new LibraryStore();
    void store.load();
  });
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
      <header class="top">
        <h1 id="library-heading">{t('library.title')}</h1>
        <TabStrip
          {tabs}
          current={activeStore.tab}
          label={t('library.tabs.label')}
          onchange={/* switch tab */ (id) => activeStore.setTab(id)}
        />
      </header>

      <LibraryBody {view} store={activeStore} {menu} />

      {#if activeStore.error}
        <p class="error" role="alert">{activeStore.error}</p>
      {/if}
      <TaskMenu controller={menu} actions={activeStore.actions} />
    </SurfaceLayout>
  {/if}
{/if}

<style>
  .top {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  h1 {
    margin: 0;
    font-size: var(--text-display);
    font-weight: 600;
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
