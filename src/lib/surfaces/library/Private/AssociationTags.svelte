<script lang="ts">
  import type { AssociationEnd, LibraryView } from '../../../domain';
  import { t } from '../../../i18n';
  import { linkedEntitiesFor } from '../../../planning-actions';
  import { Button } from '../../../ui';

  interface Props {
    end: AssociationEnd;
    view: LibraryView;
    onunlink: (associationId: string) => Promise<void> | void;
    onopenLink: () => void;
  }

  let { end, view, onunlink, onopenLink }: Props = $props();

  const linked = $derived(linkedEntitiesFor(view, end));
</script>

<div class="associations-row">
  <div class="tags">
    {#each linked as item (item.associationId)}
      <span
        class="tag"
        title={`${t(`domain.entityKind.${item.kind}`)}: ${item.title}`}
      >
        <span class="tag-title">{item.title}</span>
        <button
          type="button"
          class="unlink-btn"
          aria-label={t('planningActions.removeAssociation', { title: item.title })}
          onclick={() => void onunlink(item.associationId)}
        >
          ×
        </button>
      </span>
    {/each}
  </div>
  <Button variant="quiet" onclick={onopenLink}>
    {t('planningActions.linkTo')}
  </Button>
</div>

<style>
  .associations-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1);
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-1);
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 2px var(--space-2);
    background: var(--color-raised);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-pill);
    font-size: var(--text-label);
    color: var(--color-ink-muted);
    line-height: 1.4;
  }

  .tag:hover {
    color: var(--color-ink);
    border-color: var(--color-ink-muted);
  }

  .tag-title {
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .unlink-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    padding: 0;
    margin: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--color-ink-muted);
    font-size: 13px;
    line-height: 1;
    cursor: pointer;
  }

  .unlink-btn:hover {
    background: var(--color-hairline);
    color: var(--color-ink);
  }
</style>
