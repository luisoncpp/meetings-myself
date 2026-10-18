<script lang="ts">
  import type { GoalView, HabitView, ValueView } from '../../../domain';
  import { t } from '../../../i18n';
  import { TaskLine } from '../../../ui';
  import type { LibraryStore } from './LibraryStore.svelte';

  type Row = HabitView | GoalView | ValueView;

  interface Props {
    kind: 'habit' | 'goal' | 'value';
    rows: Row[];
    store: LibraryStore;
  }

  let { kind, rows, store }: Props = $props();

  /** One short muted note per row: what a glance needs, nothing more. */
  function note(row: Row): string {
    if ('pinned' in row) return row.pinned ? '' : t('library.notDaily');
    if ('achieved' in row && row.achieved) return t('domain.stateFlag.completed');
    if ('targetDate' in row && row.targetDate) return row.targetDate;
    return '';
  }
</script>

{#each rows as row (row.id)}
  <li>
    <TaskLine
      title={row.title}
      selected={store.selected?.kind === kind && store.selected.id === row.id}
      onselect={/* open details */ () => store.select({ kind, id: row.id })}
    >
      {#snippet meta()}
        <span>{note(row)}</span>
      {/snippet}
    </TaskLine>
  </li>
{/each}
