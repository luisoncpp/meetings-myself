<script lang="ts">
  import type { LibraryView } from '../../../domain';
  import { Button, Field, Input, Select } from '../../../ui';
  import type { WeeklyReviewStore } from './WeeklyReviewStore.svelte';
  import FocusPanel from './FocusPanel.svelte';
  import LinkPanel from './LinkPanel.svelte';

  interface Props {
    store: WeeklyReviewStore;
    library: LibraryView;
    focusWeek: string;
  }

  let { store, library, focusWeek }: Props = $props();

  let showCreate = $state(false);
  let showAchieve = $state(false);
  let showLink = $state(false);
  let showFocus = $state(false);
  let goalTitle = $state('');
  let selectedGoalId = $state('');

  const openGoals = $derived(library.goals.filter((goal) => !goal.archived && !goal.achieved));

  async function createGoal(): Promise<void> {
    const title = goalTitle.trim();
    if (title === '') return;
    await store.createGoal(title);
    goalTitle = '';
    showCreate = false;
  }

  async function markAchieved(): Promise<void> {
    if (selectedGoalId === '') return;
    await store.achieveGoal(selectedGoalId);
    selectedGoalId = '';
    showAchieve = false;
  }
</script>

<div class="actions">
  <Button variant="primary" onclick={/* new goal */ () => (showCreate = !showCreate)}>
    New goal
  </Button>
  <Button variant="secondary" onclick={/* achieve */ () => (showAchieve = !showAchieve)}>
    Mark achieved
  </Button>
  <Button variant="secondary" onclick={/* link */ () => (showLink = !showLink)}>Link</Button>
  <Button variant="secondary" onclick={/* focus */ () => (showFocus = !showFocus)}>
    Next week focus
  </Button>
</div>

{#if showCreate}
  <form
    class="panel"
    onsubmit={/* create */ (event) => {
      event.preventDefault();
      void createGoal();
    }}
  >
    <Field label="Goal name">
      <Input bind:value={goalTitle} aria-label="Goal name" />
    </Field>
    <Button type="submit" variant="primary" disabled={goalTitle.trim() === ''}>Create goal</Button>
  </form>
{/if}

{#if showAchieve}
  <div class="panel">
    <Field label="Goal">
      <Select bind:value={selectedGoalId} aria-label="Goal to mark achieved">
        <option value="">Select…</option>
        {#each openGoals as goal (goal.id)}
          <option value={goal.id}>{goal.title}</option>
        {/each}
      </Select>
    </Field>
    <Button variant="primary" disabled={selectedGoalId === ''} onclick={/* achieve */ markAchieved}>
      Mark achieved
    </Button>
  </div>
{/if}

{#if showLink}
  <LinkPanel view={library} {store} onclose={/* close */ () => (showLink = false)} />
{/if}

{#if showFocus}
  <FocusPanel
    {store}
  library={library}
    {focusWeek}
    onclose={/* close */ () => (showFocus = false)}
  />
{/if}

<style>
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .panel {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
    padding: var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
  }

</style>
