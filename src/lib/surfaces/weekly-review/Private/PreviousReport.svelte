<script lang="ts">
  import { renderReportBody } from './report-body';

  interface Props {
    body: string | null;
  }

  let { body }: Props = $props();

  const html = $derived(body ? renderReportBody(body) : null);
</script>

{#if html}
  <section class="previous" aria-labelledby="previous-report-heading">
    <h2 id="previous-report-heading">Previous report</h2>
    <div class="body">{@html html}</div>
  </section>
{/if}

<style>
  .previous {
    margin-bottom: var(--space-4);
  }

  h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .body {
    font-size: var(--text-body);
    color: var(--color-ink-muted);
  }

  .body :global(h2) {
    margin: 0 0 var(--space-2);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
  }

  .body :global(h3) {
    margin: var(--space-3) 0 var(--space-2);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink);
  }

  .body :global(p) {
    margin: var(--space-2) 0;
  }

  .body :global(ul) {
    margin: var(--space-2) 0;
    padding-left: var(--space-5);
  }

  .body :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: var(--space-2) 0;
  }

  .body :global(th),
  .body :global(td) {
    text-align: left;
    padding: var(--space-1) var(--space-2);
    border-bottom: 1px solid var(--color-hairline);
  }

  .body :global(th) {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
    font-weight: 500;
  }
</style>
