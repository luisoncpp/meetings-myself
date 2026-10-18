<script lang="ts">
  import { t } from '../../../i18n';
  import HabitOutcomes from './HabitOutcomes.svelte';
  import { presentPreviousReport } from './report-body';

  interface Props {
    body: string | null;
  }

  let { body }: Props = $props();

  const parts = $derived(body ? presentPreviousReport(body) : null);
  const visible = $derived(
    parts !== null &&
      (parts.title !== null ||
        parts.recordHtml !== '' ||
        (parts.habits?.length ?? 0) > 0 ||
        parts.proseHtml !== ''),
  );
</script>

{#if parts && visible}
  <section
    class="previous"
    aria-labelledby={parts.title ? 'previous-report-heading' : undefined}
    aria-label={parts.title ? undefined : t('weeklyReview.previousReport')}
  >
    {#if parts.title}
      <h2 id="previous-report-heading">
        <span>{parts.title}</span>
        {#if parts.stillOpen !== null}
          <span class="still-open">{t('weeklyReview.stillOpen', { count: parts.stillOpen })}</span>
        {/if}
      </h2>
    {/if}

    {#if parts.recordHtml}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked output of the saved report -->
      <div class="record">{@html parts.recordHtml}</div>
    {/if}

    {#if parts.habits && parts.habits.length > 0}
      <h3>{t('weeklyReview.habits')}</h3>
      <HabitOutcomes habits={parts.habits} />
    {/if}

    {#if parts.proseHtml}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -- marked output of the saved reflection -->
      <div class="prose">{@html parts.proseHtml}</div>
    {/if}
  </section>
{/if}

<style>
  .previous {
    margin-bottom: var(--space-8);
  }

  h2 {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--space-3);
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
    color: var(--color-ink);
    text-wrap: balance;
  }

  .still-open {
    font-size: var(--text-label);
    font-weight: 500;
    color: var(--color-ink-muted);
  }

  h3 {
    margin: var(--space-3) 0 var(--space-2);
    font-size: var(--text-body);
    font-weight: 600;
  }

  .record {
    font-size: var(--text-body);
    color: var(--color-ink);
  }

  .record :global(p),
  .record :global(ul) {
    margin: var(--space-2) 0;
  }

  .record :global(ul) {
    padding-left: var(--space-5);
  }

  .record :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: var(--space-2) 0;
  }

  .record :global(th),
  .record :global(td) {
    text-align: left;
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--color-hairline);
  }

  .record :global(th) {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
    font-weight: 500;
  }

  .prose {
    margin-top: var(--space-6);
    color: var(--color-ink);
    font-size: var(--text-body);
    line-height: 1.6;
  }

  .prose :global(h2) {
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
    color: var(--color-ink);
    text-wrap: balance;
  }

  .prose :global(p) {
    margin: 0 0 var(--space-4);
    text-wrap: pretty;
  }
</style>
