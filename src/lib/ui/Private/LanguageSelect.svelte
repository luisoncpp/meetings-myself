<script lang="ts">
  import type { UiLanguage } from '../../api';
  import { localeStore, t } from '../../i18n';

  interface Props {
    compact?: boolean;
  }

  let { compact = false }: Props = $props();

  const options: { value: UiLanguage; label: string }[] = [
    { value: 'en', label: t('language.en') },
    { value: 'es', label: t('language.es') },
  ];

  function onChange(event: Event): void {
    const value = (event.currentTarget as HTMLSelectElement).value as UiLanguage;
    void localeStore.setLocale(value);
  }
</script>

<label class="language" class:compact>
  {#if !compact}
    <span class="label">{t('language.label')}</span>
  {/if}
  <select
    aria-label={t('language.label')}
    value={localeStore.locale}
    onchange={/* set language */ onChange}
  >
    {#each options as option (option.value)}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
</label>

<style>
  .language {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .label {
    white-space: nowrap;
  }

  select {
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-base);
    color: var(--color-ink);
    font: inherit;
    padding: var(--space-1) var(--space-2);
  }

  .compact select {
    padding: var(--space-1);
  }
</style>
