<script lang="ts">
  import { t } from '../../i18n';

  interface Props {
    completed: boolean;
    taskTitle: string;
    ontoggle: () => void;
  }

  let { completed, taskTitle, ontoggle }: Props = $props();

  const label = $derived(completed ? t('common.done') : t('common.markDone'));
  const ariaLabel = $derived(
    completed ? t('common.reopen', { title: taskTitle }) : t('common.markDoneFor', { title: taskTitle }),
  );
</script>

<button
  type="button"
  class="toggle"
  class:selected={completed}
  aria-pressed={completed}
  aria-label={ariaLabel}
  onclick={/* toggle completion */ ontoggle}
>
  {label}
</button>

<style>
  .toggle {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-label);
    white-space: nowrap;
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }

  .toggle:hover {
    border-color: var(--color-gold);
    color: var(--color-ink);
  }

  .selected {
    border-color: var(--color-done);
    color: var(--color-done);
  }

  .selected:hover {
    border-color: var(--color-gold);
    color: var(--color-gold);
  }
</style>
