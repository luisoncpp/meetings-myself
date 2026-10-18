<script lang="ts">
  import type { WeeklyReviewView } from '../../../domain';
  import { t } from '../../../i18n';
  import PreviousReport from './PreviousReport.svelte';
  import ReportPath from './ReportPath.svelte';
  import SummarySection from './SummarySection.svelte';

  interface Props {
    view: WeeklyReviewView;
  }

  let { view }: Props = $props();
</script>

<!-- Reference material, collapsed to one line each so the focus board stays on screen. -->
<details class="disclosure">
  <summary>{t('weeklyReview.summaryDisclosure', { count: view.summary.completed.length })}</summary>
  <SummarySection summary={view.summary} />
</details>
{#if view.previousReport}
  <details class="disclosure">
    <summary>{t('weeklyReview.previousReport')}</summary>
    <PreviousReport body={view.previousReport} />
  </details>
{/if}
<ReportPath path={view.reportPath} />

<style>
  /* No flex on <summary>: WebView2 hides the collapsed disclosure (lessons-learned). */
  .disclosure {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-card);
    background: var(--color-lift);
  }

  .disclosure summary {
    padding: var(--space-1) 0;
    color: var(--color-ink-muted);
    cursor: pointer;
  }

  .disclosure[open] summary {
    margin-bottom: var(--space-3);
  }
</style>
