<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../domain';
  import { t } from '../../i18n';
  import { Button } from '../../ui';
  import {
    orderedLinkTargetsFor,
    unlinkedCandidatesFor,
    type EntityKind,
  } from './associations';

  interface Props {
    fromEnd: AssociationEnd;
    fromTitle: string;
    view: LibraryView;
    onlink: (toEnd: AssociationEnd) => Promise<void>;
    onclose: () => void;
  }

  let { fromEnd, fromTitle, view, onlink, onclose }: Props = $props();

  const allowedTabs = $derived(orderedLinkTargetsFor(fromEnd.kind));
  let activeTab = $state<EntityKind>('goal');
  let linking = $state(false);

  $effect(() => {
    if (allowedTabs.length > 0 && !allowedTabs.includes(activeTab)) {
      activeTab = allowedTabs[0] ?? 'goal';
    }
  });

  const candidates = $derived(unlinkedCandidatesFor(view, fromEnd, activeTab));

  async function handleLink(targetId: string): Promise<void> {
    if (linking) return;
    linking = true;
    try {
      await onlink({ kind: activeTab, id: targetId });
      onclose();
    } finally {
      linking = false;
    }
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="backdrop" onclick={onclose} role="presentation">
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    aria-label={t('planningActions.linkEntity', { title: fromTitle })}
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="header">
      <h3>{t('planningActions.linkEntity', { title: fromTitle })}</h3>
      <Button variant="quiet" onclick={onclose}>{t('common.close')}</Button>
    </div>

    {#if allowedTabs.length > 1}
      <div class="tabs" role="tablist">
        {#each allowedTabs as kind (kind)}
          <button
            type="button"
            role="tab"
            class="tab"
            class:active={activeTab === kind}
            aria-selected={activeTab === kind}
            onclick={() => (activeTab = kind)}
          >
            {t(`domain.entityKind.${kind}`)}
          </button>
        {/each}
      </div>
    {/if}

    <div class="tab-content" role="tabpanel">
      {#if candidates.length === 0}
        <p class="empty">
          {t('planningActions.noCandidatesAvailable', {
            kind: t(`domain.entityKind.${activeTab}`),
          })}
        </p>
      {:else}
        <ul class="candidate-list">
          {#each candidates as item (item.id)}
            <li class="candidate-item">
              <span class="candidate-title">{item.title}</span>
              <Button
                variant="primary"
                disabled={linking}
                onclick={() => void handleLink(item.id)}
              >
                {t('common.link')}
              </Button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgb(0 0 0 / 0.5);
    padding: var(--space-4);
  }
  .modal {
    width: 100%;
    max-width: 480px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    background: var(--color-lift);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-card);
    overflow: hidden;
  }
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--color-hairline);
    background: var(--color-raised);
  }
  h3 { margin: 0; font-size: var(--text-body); font-weight: 600; color: var(--color-ink); }
  .tabs {
    display: flex;
    gap: var(--space-1);
    padding: var(--space-2) var(--space-4) 0;
    border-bottom: 1px solid var(--color-hairline);
    background: var(--color-raised);
  }
  .tab {
    padding: var(--space-2) var(--space-3);
    border: none;
    border-bottom: 2px solid transparent;
    background: transparent;
    font-size: var(--text-body);
    color: var(--color-ink-muted);
    cursor: pointer;
    text-transform: capitalize;
  }
  .tab.active { color: var(--color-gold); border-bottom-color: var(--color-gold); font-weight: 600; }
  .tab:hover:not(.active) { color: var(--color-ink); }
  .tab-content { padding: var(--space-4); overflow-y: auto; flex: 1; }
  .empty {
    margin: 0;
    color: var(--color-ink-muted);
    font-size: var(--text-body);
    text-align: center;
    padding: var(--space-4) 0;
  }
  .candidate-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }
  .candidate-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
    background: var(--color-raised);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
  }
  .candidate-title {
    font-size: var(--text-body);
    color: var(--color-ink);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
