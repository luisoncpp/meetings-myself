<script lang="ts">
  import type { Association, AssociationEnd, LibraryView } from '../../domain';
  import { t } from '../../i18n';
  import { Button, Field, InsetPanel, Select } from '../../ui';
  import * as api from '../../api';
  import {
    candidatesFor,
    entityTitle,
    linkTargetsFor,
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
    void host.link(fromEnd, { kind: toKind, id: toId });
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
        {#each candidatesFor(view, toKind) as end (end.id)}
          <option value={end.id}>{entityTitle(view, end)}</option>
        {/each}
      </Select>
    </Field>
  </div>

  <Button variant="primary" disabled={fromId === '' || toId === ''} onclick={/* link */ linkEntities}>
    {t('common.link')}
  </Button>

  {#if links.length > 0}
    <ul class="links">
      {#each links as link (link.id)}
        <li>
          {entityTitle(view, link.left)} ↔ {entityTitle(view, link.right)}
          <Button variant="quiet" onclick={/* unlink */ () => void host.unlink(link.id)}>
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
