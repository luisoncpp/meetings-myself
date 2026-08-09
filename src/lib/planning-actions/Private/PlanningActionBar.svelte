<script lang="ts">
  import type { LibraryView } from '../../domain';
  import { Button } from '../../ui';
  import AchieveGoalPanel from './AchieveGoalPanel.svelte';
  import AssociationEditor from './AssociationEditor.svelte';
  import CreateEntity, { type CreatePayload } from './CreateEntity.svelte';
  import type { PlanningActionsHost } from './planning-actions-host';
  import WeeklyFocusPanel from './WeeklyFocusPanel.svelte';

  export type PlanningAction = 'new-goal' | 'achieve' | 'link' | 'focus';

  interface Props {
    active: PlanningAction | null;
    view: LibraryView;
    host: PlanningActionsHost;
    week: string;
    ontoggle: (action: PlanningAction) => void;
    onclose: () => void;
    oncreateGoal: (payload: CreatePayload) => void;
  }

  let { active, view, host, week, ontoggle, onclose, oncreateGoal }: Props = $props();

  const actions: { id: PlanningAction; label: string }[] = [
    { id: 'new-goal', label: 'New goal' },
    { id: 'achieve', label: 'Mark achieved' },
    { id: 'link', label: 'Link' },
    { id: 'focus', label: 'Weekly focus' },
  ];
</script>

<section class="action-bar" aria-label="Planning actions">
  <div class="toolbar" role="toolbar" aria-label="Quick actions">
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
      {#if active === 'new-goal'}
        <CreateEntity
          kind="goal"
          oncreate={oncreateGoal}
          oncancel={/* close */ onclose}
        />
      {:else if active === 'achieve'}
        <AchieveGoalPanel goals={view.goals} {host} onclose={/* close */ onclose} />
      {:else if active === 'link'}
        <AssociationEditor {view} {host} onclose={/* close */ onclose} />
      {:else}
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
