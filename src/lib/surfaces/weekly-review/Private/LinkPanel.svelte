<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../../domain';
  import { Button, InsetPanel } from '../../../ui';
  import type { WeeklyReviewStore } from './WeeklyReviewStore.svelte';

  type EntityKind = AssociationEnd['kind'];

  interface Props {
    view: LibraryView;
    store: WeeklyReviewStore;
    onclose: () => void;
  }

  let { view, store, onclose }: Props = $props();

  let fromKind = $state<EntityKind>('goal');
  let fromId = $state('');
  let toKind = $state<EntityKind>('value');
  let toId = $state('');

  const targets: Record<EntityKind, EntityKind[]> = {
    value: ['goal'],
    goal: ['value', 'habit', 'task'],
    habit: ['goal', 'task'],
    task: ['goal', 'habit'],
  };

  const allowed = $derived(targets[fromKind]);

  $effect(() => {
    if (allowed.includes(toKind)) return;
    toKind = allowed[0] ?? 'goal';
    toId = '';
  });

  function entities(kind: EntityKind): { id: string; title: string }[] {
    if (kind === 'value') return view.values.map((v) => ({ id: v.id, title: v.title }));
    if (kind === 'goal') return view.goals.map((g) => ({ id: g.id, title: g.title }));
    if (kind === 'habit') return view.habits.map((h) => ({ id: h.id, title: h.title }));
    return view.tasks.map((t) => ({ id: t.id, title: t.title }));
  }

  function link(): void {
    if (fromId === '' || toId === '') return;
    void store.link({ kind: fromKind, id: fromId }, { kind: toKind, id: toId });
  }
</script>

<InsetPanel title="Link association" label="Association editor">
  <div class="pickers">
    <label>
      From
      <select bind:value={fromKind}>
        <option value="value">Value</option>
        <option value="goal">Goal</option>
        <option value="habit">Habit</option>
        <option value="task">Task</option>
      </select>
      <select bind:value={fromId}>
        <option value="">Select…</option>
        {#each entities(fromKind) as item (item.id)}
          <option value={item.id}>{item.title}</option>
        {/each}
      </select>
    </label>
    <label>
      To
      <select bind:value={toKind}>
        {#each allowed as kind (kind)}
          <option value={kind}>{kind}</option>
        {/each}
      </select>
      <select bind:value={toId}>
        <option value="">Select…</option>
        {#each entities(toKind) as item (item.id)}
          <option value={item.id}>{item.title}</option>
        {/each}
      </select>
    </label>
  </div>
  <Button variant="primary" disabled={fromId === '' || toId === ''} onclick={/* link */ link}>
    Link
  </Button>

  {#snippet footer()}
    <Button variant="quiet" onclick={/* close */ onclose}>Close</Button>
  {/snippet}
</InsetPanel>

<style>
  .pickers {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }
</style>
