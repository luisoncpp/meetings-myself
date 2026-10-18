<script lang="ts" module>
  export type MainView = 'today' | 'library' | 'week';
</script>

<script lang="ts">
  import { t } from '../../i18n';
  import { LanguageSelect } from '../../ui';

  interface Props {
    current: MainView;
    onnavigate: (view: MainView) => void;
    onswitchFolder?: () => void;
  }

  let { current, onnavigate, onswitchFolder }: Props = $props();

  const views: { id: MainView; label: string }[] = $derived([
    { id: 'today', label: t('nav.today') },
    { id: 'library', label: t('nav.library') },
    { id: 'week', label: t('nav.week') },
  ]);
</script>

<nav class="rail" aria-label={t('nav.main')}>
  <ul class="views">
    {#each views as view (view.id)}
      <li>
        <button
          type="button"
          class:selected={current === view.id}
          aria-current={current === view.id ? 'page' : undefined}
          onclick={/* show this section */ () => onnavigate(view.id)}
        >
          {view.label}
        </button>
      </li>
    {/each}
  </ul>
  <div class="footer">
    {#if onswitchFolder}
      <button type="button" class="quiet" onclick={onswitchFolder}>
        {t('nav.switchFolder')}
      </button>
    {/if}
    <LanguageSelect compact />
  </div>
</nav>

<style>
  .rail {
    position: sticky;
    top: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: var(--space-4) var(--space-2);
    background: var(--color-lift);
  }

  .views {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  button:hover {
    color: var(--color-ink);
  }

  .selected {
    background: var(--color-raised);
    color: var(--color-gold);
    font-weight: 600;
  }

  .footer {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .quiet {
    font-size: var(--text-label);
  }
</style>
