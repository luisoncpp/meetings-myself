<script lang="ts">
  import type { ValueView } from '../../../domain';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    value: ValueView;
    store: LibraryStore;
  }

  let { value, store }: Props = $props();

  function toggleArchive(): void {
    const end = { kind: 'value' as const, id: value.id };
    if (value.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }
</script>

<ListRow muted={value.archived}>
  <span>{value.title}</span>
  {#if value.archived}
    <StateFlag kind="archived" />
  {/if}
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {value.archived ? 'Restore' : 'Archive'}
    </Button>
  {/snippet}
</ListRow>
