<script lang="ts">
  import type { LibraryView } from '../../domain';
  import { t } from '../../i18n';
  import { Button } from '../../ui';
  import AchieveGoalPanel from './AchieveGoalPanel.svelte';
  import type { EntityKind } from './associations';
  import CreateEntity, { type CreatePayload } from './CreateEntity.svelte';
  import type { PlanningActionsHost } from './planning-actions-host';
  import WeeklyFocusPanel from './WeeklyFocusPanel.svelte';

  export type PlanningAction = 'new-task' | 'new-habit' | 'new-goal' | 'achieve' | 'focus';

  interface Props {
    active: PlanningAction | null;
    view: LibraryView;
    host: PlanningActionsHost;
    week: string;
    ontoggle: (action: PlanningAction) => void;
    onclose: () => void;
    oncreate: (payload: CreatePayload) => void;
  }

  let { active, view, host, week, ontoggle, onclose, oncreate }: Props = $props();

  const actions: { id: PlanningAction; label: string }[] = [
    { id: 'new-task', label: t('planningActions.newTask') },
    { id: 'new-habit', label: t('planningActions.newHabit') },
    { id: 'new-goal', label: t('planningActions.newGoal') },
    { id: 'achieve', label: t('planningActions.markAchieved') },
    { id: 'focus', label: t('planningActions.weeklyFocus') },
  ];

  const createKind = $derived<EntityKind | null>(
    active === 'new-task'
      ? 'task'
      : active === 'new-habit'
        ? 'habit'
        : active === 'new-goal'
          ? 'goal'
          : null,
  );
</script>

<section class="action-bar" aria-label={t('planningActions.bar')}>
  <div class="toolbar" role="toolbar" aria-label={t('planningActions.toolbar')}>
    {#each actions as action (action.id)}
      <Button
        variant={active === action.id ? 'primary' : 'secondary'}
        pressed={active === action.id}
        onclick={/* toggle action */ () => ontoggle(action.id)}
      >
        {action.label}
      </Button>
    {/each}
  </div>

  {#if active}
    <div class="action-panel" role="region" aria-label={actions.find((item) => item.id === active)?.label}>
      {#if createKind !== null}
        <CreateEntity
          kind={createKind}
          {oncreate}
          oncancel={/* close */ onclose}
        />
      {:else if active === 'achieve'}
        <AchieveGoalPanel goals={view.goals} {host} onclose={/* close */ onclose} />
      {:else if active === 'focus'}
        <WeeklyFocusPanel {view} {host} {week} onclose={/* close */ onclose} />
      {/if}
    </div>
  {/if}
</section>

<style>
  .action-bar {
    margin-bottom: var(--space-4);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
    background: var(--color-raised);
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    padding: var(--space-2);
  }

  .action-panel {
    padding: var(--space-3);
    border-top: 1px solid var(--color-hairline);
  }

  .action-panel :global(.panel),
  .action-panel :global(.inset-panel) {
    margin-bottom: 0;
    padding: 0;
    border: none;
    border-radius: 0;
    background: transparent;
  }

  .action-panel :global(.create) {
    margin-bottom: 0;
  }
</style>
