<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { Button } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    view: LibraryView;
    store: LibraryStore;
    week: string;
    onclose: () => void;
  }

  let { view, store, week, onclose }: Props = $props();

  const focusIds = $derived(new Set(store.focus?.tasks ?? []));

  function inFocus(taskId: string): boolean {
    return focusIds.has(taskId);
  }
</script>

<section class="focus-panel" aria-label="Weekly focus">
  <h3>Weekly focus ({week})</h3>

  <ul class="tasks">
    {#each view.tasks.filter((task) => !task.archived) as task (task.id)}
      <li>
        <span>{task.title}</span>
        {#if inFocus(task.id)}
          <Button variant="quiet" onclick={/* remove */ () => void store.removeFromFocus(task.id)}>
            Remove from focus
          </Button>
        {:else}
          <Button variant="quiet" onclick={/* add */ () => void store.addToFocus(task.id)}>
            Add to focus
          </Button>
        {/if}
      </li>
    {/each}
  </ul>

  <Button variant="quiet" onclick={/* close */ onclose}>Close</Button>
</section>

<style>
  .focus-panel {
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
  }

  h3 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-title);
  }

  .tasks {
    list-style: none;
    margin: 0 0 var(--space-3);
    padding: 0;
  }

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-1) 0;
  }
</style>
