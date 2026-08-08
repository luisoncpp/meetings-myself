<script lang="ts">
  import type { CheckInOutcome } from '../../domain';
  import { nextOutcome, OUTCOME_OPTIONS } from './outcome-navigation';

  interface Props {
    value: CheckInOutcome | null;
    label: string;
    onchange: (next: CheckInOutcome) => void;
  }

  let { value, label, onchange }: Props = $props();

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
  aria-label="Check-in for {label}"
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
      tabindex={tabIndexFor(outcome.value)}
      class:selected={value === outcome.value}
      onclick={/* record this outcome */ () => onchange(outcome.value)}
    >
      {outcome.label}
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

  /* Selection is the accent's job. No celebration, no animation on check-in. */
  .selected {
    border-color: var(--color-gold);
    color: var(--color-gold);
  }
</style>
