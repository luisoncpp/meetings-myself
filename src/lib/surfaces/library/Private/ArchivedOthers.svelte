<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { Button, TaskLine } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    view: LibraryView;
    store: LibraryStore;
  }

  let { view, store }: Props = $props();

  interface Row {
    id: string;
    title: string;
    kindLabel: string;
    restore: () => void;
  }

  function entityRows(kind: AssociationEnd['kind'], items: { id: string; title: string; archived: boolean }[]): Row[] {
    return items
      .filter((item) => item.archived)
      .map((item) => ({
        id: `${kind}-${item.id}`,
        title: item.title,
        kindLabel: t(`domain.entityKind.${kind}`),
        restore: () => void store.setArchived({ kind, id: item.id } as AssociationEnd, false),
      }));
  }

  const rows = $derived<Row[]>([
    ...entityRows('habit', view.habits),
    ...entityRows('goal', view.goals),
    ...entityRows('value', view.values),
    ...store.archivedRecurringTasks.map((rule) => ({
      id: `rule-${rule.id}`,
      title: rule.title,
      kindLabel: t('library.tabs.recurring'),
      restore: () => void store.setRecurringArchived(rule.id, false),
    })),
  ]);
</script>

{#if rows.length > 0}
  <h3>{t('library.archive.other')}</h3>
  <ul class="rows">
    {#each rows as row (row.id)}
      <li>
        <TaskLine title={row.title} muted>
          {#snippet meta()}<span>{row.kindLabel}</span>{/snippet}
          {#snippet trailing()}
            <Button variant="quiet" onclick={/* restore */ row.restore}>{t('common.restore')}</Button>
          {/snippet}
        </TaskLine>
      </li>
    {/each}
  </ul>
{/if}

<style>
  h3 {
    margin: var(--space-4) var(--space-2) var(--space-1);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }
</style>
