<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../../i18n';

  interface Props {
    title: string;
    onclose: () => void;
    children: Snippet;
  }

  let { title, onclose, children }: Props = $props();
</script>

<aside class="panel" aria-label={title}>
  <header>
    <h2>{title}</h2>
    <button type="button" class="close" aria-label={t('common.close')} onclick={/* close panel */ onclose}>
      ×
    </button>
  </header>
  <div class="body">{@render children()}</div>
</aside>

<style>
  .panel {
    position: sticky;
    top: var(--space-4);
    padding: var(--space-4);
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }

  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  h2 {
    margin: 0;
    font-size: var(--text-title);
    font-weight: 600;
    overflow-wrap: anywhere;
  }

  .close {
    padding: 0 var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-headline);
    line-height: 1;
    cursor: pointer;
  }

  .close:hover {
    color: var(--color-ink);
  }

  .body {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }
</style>
