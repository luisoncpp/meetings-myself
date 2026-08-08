<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { Button, InsetPanel } from '../../../ui';
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

<InsetPanel title="Adjust focus ({focusWeek})" label="Next week focus editor">
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

  {#snippet footer()}
    <Button variant="quiet" onclick={/* close */ onclose}>Close</Button>
  {/snippet}
</InsetPanel>

<style>
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
