<script lang="ts">
  import type { Association, AssociationEnd, LibraryView } from '../../../domain';
  import { Button, Field, InsetPanel, Select } from '../../../ui';
  import * as api from '../../../api';
  import {
    candidatesFor,
    entityTitle,
    linkTargetsFor,
    type EntityKind,
  } from './associations';
  import type { LibraryStore } from './LibraryStore.svelte';

  interface Props {
    view: LibraryView;
    store: LibraryStore;
    onclose: () => void;
  }

  let { view, store, onclose }: Props = $props();

  let fromKind = $state<EntityKind>('goal');
  let fromId = $state('');
  let toKind = $state<EntityKind>('value');
  let toId = $state('');
  let links = $state<Association[]>([]);

  const fromEnd = $derived(
    fromId === '' ? null : ({ kind: fromKind, id: fromId } satisfies AssociationEnd),
  );

  const allowedTargets = $derived(linkTargetsFor(fromKind));

  $effect(() => {
    if (!allowedTargets.includes(toKind)) {
      toKind = allowedTargets[0] ?? 'goal';
      toId = '';
    }
  });

  $effect(() => {
    if (!fromEnd) {
      links = [];
      return;
    }
    void api.associationsFor(fromEnd).then((result) => {
      links = result;
    });
  });

  function entitiesFor(kind: EntityKind): { id: string; title: string }[] {
    switch (kind) {
      case 'value':
        return view.values.map((item) => ({ id: item.id, title: item.title }));
      case 'goal':
        return view.goals.map((item) => ({ id: item.id, title: item.title }));
      case 'habit':
        return view.habits.map((item) => ({ id: item.id, title: item.title }));
      case 'task':
        return view.tasks.map((item) => ({ id: item.id, title: item.title }));
    }
  }

  function linkEntities(): void {
    if (!fromEnd || toId === '') return;
    void store.link(fromEnd, { kind: toKind, id: toId });
  }
</script>

<InsetPanel title="Link associations" label="Association editor">
  <div class="pickers">
    <Field label="From">
      <Select bind:value={fromKind}>
        <option value="value">Value</option>
        <option value="goal">Goal</option>
        <option value="habit">Habit</option>
        <option value="task">Task</option>
      </Select>
      <Select bind:value={fromId}>
        <option value="">Select…</option>
        {#each entitiesFor(fromKind) as item (item.id)}
          <option value={item.id}>{item.title}</option>
        {/each}
      </Select>
    </Field>

    <Field label="To">
      <Select bind:value={toKind}>
        {#each allowedTargets as kind (kind)}
          <option value={kind}>{kind}</option>
        {/each}
      </Select>
      <Select bind:value={toId}>
        <option value="">Select…</option>
        {#each candidatesFor(view, toKind) as end (end.id)}
          <option value={end.id}>{entityTitle(view, end)}</option>
        {/each}
      </Select>
    </Field>
  </div>

  <Button variant="primary" disabled={fromId === '' || toId === ''} onclick={/* link */ linkEntities}>
    Link
  </Button>

  {#if links.length > 0}
    <ul class="links">
      {#each links as link (link.id)}
        <li>
          {entityTitle(view, link.left)} ↔ {entityTitle(view, link.right)}
          <Button variant="quiet" onclick={/* unlink */ () => void store.unlink(link.id)}>
            Unlink
          </Button>
        </li>
      {/each}
    </ul>
  {/if}

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

  .links {
    list-style: none;
    margin: var(--space-3) 0;
    padding: 0;
  }

  li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
</style>
