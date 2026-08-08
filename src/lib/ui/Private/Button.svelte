<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'secondary' | 'quiet';
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: () => void;
    children: Snippet;
  }

  let {
    variant = 'secondary',
    disabled = false,
    type = 'button',
    onclick,
    children,
  }: Props = $props();
</script>

<button class={variant} {type} {disabled} {onclick}>
  {@render children()}
</button>

<style>
  button {
    padding: var(--space-2) var(--space-4);
    border: 1px solid transparent;
    border-radius: var(--radius-control);
    font: inherit;
    font-size: var(--text-body);
    cursor: pointer;
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.55;
  }

  /* The One Accent Rule: solid gold is rare, and only ever a primary action. */
  .primary {
    background: var(--color-gold);
    color: var(--color-base);
    font-weight: 600;
  }

  .primary:hover:not(:disabled) {
    background: var(--color-gold-deep);
  }

  .secondary {
    background: var(--color-raised);
    color: var(--color-ink);
    border-color: var(--color-hairline);
  }

  .secondary:hover:not(:disabled) {
    background: var(--color-lift);
  }

  .quiet {
    background: none;
    color: var(--color-ink-muted);
  }

  .quiet:hover:not(:disabled) {
    color: var(--color-ink);
  }
</style>
