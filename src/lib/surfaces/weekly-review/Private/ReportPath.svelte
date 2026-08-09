<script lang="ts">
  interface Props {
    path: string;
  }

  let { path }: Props = $props();

  const filename = $derived(path.replace(/^.*[\\/]/, ''));
</script>

<p class="report-path">
  Report file:
  <span class="filename">{filename}</span>
</p>

<style>
  .report-path {
    margin: 0;
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .filename {
    font-family: var(--font-sans);
    font-variant-numeric: tabular-nums;
  }
</style>
