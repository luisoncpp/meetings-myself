<script lang="ts">
  import type { LibraryView, ValueView } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button } from '../../../ui';
  import DetailLinks from './DetailLinks.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    value: ValueView;
    view: LibraryView;
    store: LibraryStore;
  }

  let { value, view, store }: Props = $props();

  const end = $derived({ kind: 'value' as const, id: value.id });
</script>

{#if !value.archived}
  <DetailLinks {end} title={value.title} {view} {store} />
{/if}
<div>
  <Button variant="quiet" onclick={/* archive or restore */ () => void store.setArchived(end, !value.archived)}>
    {value.archived ? t('common.restore') : t('common.archive')}
  </Button>
</div>
