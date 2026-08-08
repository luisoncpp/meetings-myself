<script lang="ts">
  import { openWeeklyReviewWindow } from '../../api';

  interface Props {
    current: 'daily-plan' | 'library';
    onnavigate: (view: 'daily-plan' | 'library') => void;
  }

  let { current, onnavigate }: Props = $props();

  async function openReviewWindow(): Promise<void> {
    await openWeeklyReviewWindow();
  }
</script>

<nav class="nav" aria-label="Main">
  <button
    type="button"
    class:selected={current === 'daily-plan'}
    onclick={/* show daily plan */ () => onnavigate('daily-plan')}
  >
    Daily Plan
  </button>
  <button
    type="button"
    class:selected={current === 'library'}
    onclick={/* show library */ () => onnavigate('library')}
  >
    Library
  </button>
  <button type="button" onclick={openReviewWindow}>Weekly Review</button>
</nav>

<style>
  .nav {
    display: flex;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-6);
    border-bottom: 1px solid var(--color-hairline);
  }

  button {
    padding: var(--space-2) var(--space-3);
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-body);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  button:hover {
    color: var(--color-ink);
  }

  button.selected {
    color: var(--color-gold);
    font-weight: 600;
  }
</style>
