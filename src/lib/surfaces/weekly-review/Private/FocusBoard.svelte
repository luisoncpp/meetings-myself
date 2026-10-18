<script lang="ts">
  import type { LibraryView, TaskView } from '../../../domain';
  import { t } from '../../../i18n';
  import { taskRef, type FocusTarget, type TaskMenuController } from '../../../planning-actions';
  import { InlineAdd, TaskChips, TaskLine } from '../../../ui';
  import type { WeeklyReviewStore } from './WeeklyReviewStore.svelte';

  interface Props {
    library: LibraryView;
    store: WeeklyReviewStore;
    menu: TaskMenuController;
  }

  let { library, store, menu }: Props = $props();

  const target = $derived<FocusTarget>(store.actions.focusTargets[0]!);
  const active = $derived(library.tasks.filter((task) => !task.archived));
  const inFocus = $derived(active.filter((task) => target.taskIds.has(task.id)));
  const rest = $derived(active.filter((task) => !target.taskIds.has(task.id)));

</script>

{#snippet row(task: TaskView, focused: boolean)}
  <li>
    <TaskLine title={task.title} muted={!focused} onmenu={/* open task menu */ (point) => menu.open(taskRef(task), point)}>
      {#snippet meta()}
        <TaskChips importance={task.importance} urgency={task.urgency} overdue={task.overdue} />
      {/snippet}
      {#snippet trailing()}
        <button
          type="button"
          class="toggle"
          aria-label={t(focused ? 'weeklyReview.removeFromFocusFor' : 'weeklyReview.addToFocusFor', { title: task.title })}
          onclick={/* toggle focus */ () => void store.actions.setFocus(target, task.id)}
        >
          {focused ? '×' : '☆'}
        </button>
      {/snippet}
    </TaskLine>
  </li>
{/snippet}

<div class="board">
  <section class="column focus" aria-labelledby="focus-heading">
    <h2 id="focus-heading">{t('weeklyReview.focusFor', { week: target.week })}</h2>
    {#if inFocus.length === 0}
      <p class="empty">{t('weeklyReview.nothingChosen')}</p>
    {/if}
    <ul class="rows">{#each inFocus as task (task.id)}{@render row(task, /* focused= */ true)}{/each}</ul>
    <InlineAdd
      label={t('weeklyReview.newFocusTask')}
      placeholder={t('weeklyReview.newFocusTaskPlaceholder')}
      onsubmit={/* create into focus */ (title) => void store.createTaskInFocus(title)}
    />
  </section>
  <section class="column" aria-labelledby="rest-heading">
    <h2 id="rest-heading">{t('weeklyReview.everythingElse')}</h2>
    <ul class="rows">{#each rest as task (task.id)}{@render row(task, /* focused= */ false)}{/each}</ul>
  </section>
</div>

<style>
  .board {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--space-4);
    align-items: start;
  }

  .column {
    padding: var(--space-3);
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }

  .focus {
    background: var(--color-gold-wash);
  }

  h2 {
    margin: 0 0 var(--space-2) var(--space-2);
    font-size: var(--text-title);
    font-weight: 600;
  }

  .empty {
    margin: 0 var(--space-2);
    color: var(--color-ink-muted);
  }

  .rows {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .toggle {
    padding: var(--space-1) var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: pointer;
  }

  .toggle:hover {
    color: var(--color-gold);
  }
</style>
