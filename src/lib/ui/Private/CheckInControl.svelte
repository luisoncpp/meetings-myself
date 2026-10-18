<script lang="ts">
  import type { CheckInOutcome } from '../../domain';
  import { t } from '../../i18n';
  import { nextOutcome, OUTCOME_OPTIONS } from './outcome-navigation';

  interface Props {
    value: CheckInOutcome | null;
    label: string;
    onchange: (next: CheckInOutcome) => void;
    /** Symbols instead of words; the full outcome stays as the accessible name. */
    compact?: boolean;
  }

  let { value, label, onchange, compact = false }: Props = $props();

  const SYMBOLS: Record<CheckInOutcome, string> = { done: '✓', skipped: '↷', notCompleted: '✕' };

  function tabIndexFor(outcome: CheckInOutcome): number {
    if (value === outcome) return 0;
    if (value === null && outcome === 'done') return 0;
    return -1;
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
    event.preventDefault();
    const direction = event.key === 'ArrowRight' ? 'next' : 'prev';
    onchange(nextOutcome(value, direction));
  }
</script>

<div
  role="radiogroup"
  aria-label={t('domain.checkIn.for', { label })}
  class="group"
  tabindex={-1}
  onkeydown={handleKeydown}
>
  {#each OUTCOME_OPTIONS as outcome (outcome.value)}
    <button
      type="button"
      role="radio"
      aria-label={outcome.label}
      aria-checked={value === outcome.value}
      title={outcome.label}
      tabindex={tabIndexFor(outcome.value)}
      class:selected={value === outcome.value}
      class:compact
      onclick={/* record this outcome */ () => onchange(outcome.value)}
    >
      {compact ? SYMBOLS[outcome.value] : outcome.label}
    </button>
  {/each}
</div>

<style>
  .group {
    display: flex;
    gap: var(--space-1);
  }

  button {
    padding: var(--space-1) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    font-size: var(--text-label);
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  .compact {
    min-width: 1.75rem;
    padding: var(--space-1) var(--space-2);
  }

  /* Selection is the accent's job. No celebration, no animation on check-in. */
  .selected {
    border-color: var(--color-gold);
    color: var(--color-gold);
  }
</style>
