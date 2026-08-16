<script lang="ts">
  import type { LibraryView, ValueView } from '../../../domain';
  import { t } from '../../../i18n';
  import { LinkModal } from '../../../planning-actions';
  import { Button, ListRow, StateFlag } from '../../../ui';
  import AssociationTags from './AssociationTags.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    value: ValueView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { value, view, store }: Props = $props();

  let linkModalOpen = $state(false);

  const end = $derived({ kind: 'value' as const, id: value.id });

  function toggleArchive(): void {
    if (value.archived) {
      void store.restore(end);
      return;
    }
    void store.archive(end);
  }
</script>

<ListRow muted={value.archived}>
  <div class="content">
    <span>{value.title}</span>
    {#if !value.archived}
      <AssociationTags
        {end}
        {view}
        onunlink={(id) => void store.unlink(id)}
        onopenLink={() => (linkModalOpen = true)}
      />
    {/if}
  </div>
  {#if value.archived}
    <StateFlag kind="archived" />
  {/if}
  {#snippet trailing()}
    <Button variant="quiet" onclick={/* archive or restore */ toggleArchive}>
      {value.archived ? t('common.restore') : t('common.archive')}
    </Button>
  {/snippet}
</ListRow>

{#if linkModalOpen}
  <LinkModal
    fromEnd={end}
    fromTitle={value.title}
    {view}
    onlink={(toEnd) => store.link(end, toEnd)}
    onclose={() => (linkModalOpen = false)}
  />
{/if}

<style>
  .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }
</style>

