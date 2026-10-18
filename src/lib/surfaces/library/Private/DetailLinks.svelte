<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { LinkModal } from '../../../planning-actions';
  import AssociationTags from './AssociationTags.svelte';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    end: AssociationEnd;
    title: string;
    view: LibraryView;
    store: LibraryStore;
  }

  let { end, title, view, store }: Props = $props();

  let open = $state(false);
</script>

<div>
  <h3>{t('library.links')}</h3>
  <AssociationTags
    {end}
    {view}
    onunlink={/* unlink */ (id) => void store.unlink(id)}
    onopenLink={/* open link picker */ () => (open = true)}
  />
</div>

{#if open}
  <LinkModal
    fromEnd={end}
    fromTitle={title}
    {view}
    onlink={/* link */ (toEnd) => store.link(end, toEnd)}
    onclose={/* close picker */ () => (open = false)}
  />
{/if}

<style>
  h3 {
    margin: 0;
    font-size: var(--text-label);
    font-weight: 600;
    color: var(--color-ink-muted);
  }
</style>
