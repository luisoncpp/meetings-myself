<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../domain';
  import { t } from '../../i18n';
  import { Button, Field, InsetPanel, Select } from '../../ui';
  import {
    linkTargetsFor,
    linkedEntitiesFor,
    unlinkedCandidatesFor,
    type EntityKind,
  } from './associations';
  import type { PlanningActionsHost } from './planning-actions-host';

  interface Props {
    view: LibraryView;
    host: PlanningActionsHost;
    onclose: () => void;
  }

  let { view, host, onclose }: Props = $props();

  let fromKind = $state<EntityKind>('goal');
  let fromId = $state('');
  let toKind = $state<EntityKind>('value');
  let toId = $state('');
  let linking = $state(false);

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

  const links = $derived(fromEnd ? linkedEntitiesFor(view, fromEnd) : []);
  const availableCandidates = $derived(
    fromEnd ? unlinkedCandidatesFor(view, fromEnd, toKind) : [],
  );

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

  async function linkEntities(): Promise<void> {
    if (!fromEnd || toId === '' || linking) return;
    linking = true;
    try {
      await host.link(fromEnd, { kind: toKind, id: toId });
      toId = '';
    } finally {
      linking = false;
    }
  }

  async function unlinkEntity(associationId: string): Promise<void> {
    await host.unlink(associationId);
  }
</script>

<InsetPanel title={t('planningActions.linkAssociations')} label={t('planningActions.associationEditor')}>
  <div class="pickers">
    <Field label={t('planningActions.from')}>
      <Select bind:value={fromKind}>
        <option value="value">{t('domain.entityKind.value')}</option>
        <option value="goal">{t('domain.entityKind.goal')}</option>
        <option value="habit">{t('domain.entityKind.habit')}</option>
        <option value="task">{t('domain.entityKind.task')}</option>
      </Select>
      <Select bind:value={fromId}>
        <option value="">{t('common.select')}</option>
        {#each entitiesFor(fromKind) as item (item.id)}
          <option value={item.id}>{item.title}</option>
        {/each}
      </Select>
    </Field>

    <Field label={t('planningActions.to')}>
      <Select bind:value={toKind}>
        {#each allowedTargets as kind (kind)}
          <option value={kind}>{t(`domain.entityKind.${kind}`)}</option>
        {/each}
      </Select>
      <Select bind:value={toId}>
        <option value="">{t('common.select')}</option>
        {#each availableCandidates as end (end.id)}
          <option value={end.id}>{end.title}</option>
        {/each}
      </Select>
    </Field>
  </div>

  <Button
    variant="primary"
    disabled={fromId === '' || toId === '' || linking}
    onclick={/* link */ () => void linkEntities()}
  >
    {t('common.link')}
  </Button>

  {#if links.length > 0}
    <ul class="links">
      {#each links as link (link.associationId)}
        <li>
          <span>{link.title} ({t(`domain.entityKind.${link.kind}`)})</span>
          <Button variant="quiet" onclick={/* unlink */ () => void unlinkEntity(link.associationId)}>
            {t('common.unlink')}
          </Button>
        </li>
      {/each}
    </ul>
  {/if}

  {#snippet footer()}
    <Button variant="quiet" onclick={/* close */ onclose}>{t('common.close')}</Button>
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
