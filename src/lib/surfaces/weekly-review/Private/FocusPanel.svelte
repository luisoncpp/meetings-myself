<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { Button } from '../../../ui';
  import type { WeeklyReviewStore } from './WeeklyReviewStore.svelte';

  interface Props {
    store: WeeklyReviewStore;
    library: LibraryView;
    focusWeek: string;
    onclose: () => void;
  }

  let { store, library, focusWeek, onclose }: Props = $props();

  const focusIds = $derived(new Set(store.focus?.tasks ?? []));
</script>

<section class="focus-panel" aria-label="Next week focus editor">
  <h3>Adjust focus ({focusWeek})</h3>
  <ul>
    {#each library.tasks.filter((task) => !task.archived) as task (task.id)}
      <li>
        <span>{task.title}</span>
        {#if focusIds.has(task.id)}
          <Button variant="quiet" onclick={/* remove */ () => void store.removeFromFocus(task.id)}>
            Remove
          </Button>
        {:else}
          <Button variant="quiet" onclick={/* add */ () => void store.addToFocus(task.id)}>
            Add
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

  ul {
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
