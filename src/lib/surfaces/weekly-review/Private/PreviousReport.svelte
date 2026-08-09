<script lang="ts">
  interface Props {
    body: string | null;
  }

  let { body }: Props = $props();
</script>

{#if body}
  <section class="previous" aria-labelledby="previous-report-heading">
    <h2 id="previous-report-heading">Previous report</h2>
    <div class="body">{body}</div>
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
    max-width: 65ch;
    white-space: pre-wrap;
    font-size: var(--text-body);
    color: var(--color-ink-muted);
  }
</style>
