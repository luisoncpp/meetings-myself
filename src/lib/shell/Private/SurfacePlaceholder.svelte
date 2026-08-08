<script lang="ts">
  interface Props {
    kind: 'setup' | 'daily-plan' | 'library' | 'weekly-review';
  }

  let { kind }: Props = $props();

  const HEADINGS: Record<Props['kind'], string> = {
    setup: 'Setup',
    'daily-plan': 'Daily Plan',
    library: 'Library',
    'weekly-review': 'Weekly Review',
  };

  const NOTES: Record<Props['kind'], string> = {
    setup: 'Choose your sync folder and home time zone to get started.',
    'daily-plan': 'Your home surface will load here.',
    library: 'Values, tasks, goals, and habits will appear here.',
    'weekly-review': 'Reflection and weekly summary will appear here.',
  };
</script>

<section class="placeholder">
  <h1>{HEADINGS[kind]}</h1>
  <p>{NOTES[kind]}</p>
</section>

<style>
  .placeholder {
    padding: var(--space-6);
  }

  h1 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-display);
    font-weight: 600;
    line-height: 1.15;
  }

  p {
    margin: 0;
    color: var(--color-ink-muted);
    font-size: var(--text-body);
  }
</style>
