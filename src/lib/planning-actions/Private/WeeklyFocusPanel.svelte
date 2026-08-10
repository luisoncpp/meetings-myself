<script lang="ts">
  import type { LibraryView } from '../../domain';
  import { t } from '../../i18n';
  import { Button, InsetPanel } from '../../ui';
  import type { PlanningActionsHost } from './planning-actions-host';

  interface Props {
    view: LibraryView;
    host: PlanningActionsHost;
    week: string;
    onclose: () => void;
  }

  let { view, host, week, onclose }: Props = $props();

  const focusIds = $derived(new Set(host.focus?.tasks ?? []));

  function inFocus(taskId: string): boolean {
    return focusIds.has(taskId);
  }
</script>

<InsetPanel title={t('planningActions.weeklyFocusWithWeek', { week })} label={t('planningActions.weeklyFocus')}>
  <ul class="tasks">
    {#each view.tasks.filter((task) => !task.archived) as task (task.id)}
      <li>
        <span>{task.title}</span>
        {#if inFocus(task.id)}
          <Button variant="quiet" onclick={/* remove */ () => void host.removeFromFocus(task.id)}>
            {t('planningActions.removeFromFocus')}
          </Button>
        {:else}
          <Button variant="quiet" onclick={/* add */ () => void host.addToFocus(task.id)}>
            {t('planningActions.addToFocus')}
          </Button>
        {/if}
      </li>
    {/each}
  </ul>

  {#snippet footer()}
    <Button variant="quiet" onclick={/* close */ onclose}>{t('common.close')}</Button>
  {/snippet}
</InsetPanel>

<style>
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
