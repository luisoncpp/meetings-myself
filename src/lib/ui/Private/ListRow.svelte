<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    leading?: Snippet;
    children: Snippet;
    trailing?: Snippet;
    muted?: boolean;
    onactivate?: () => void;
  }

  let {
    leading,
    children,
    trailing,
    muted = false,
    onactivate,
  }: Props = $props();

  function handleKeydown(event: KeyboardEvent): void {
    if (!onactivate) return;
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onactivate();
  }
</script>

<div
  class="row"
  class:muted
  class:interactive={!!onactivate}
  role={onactivate ? 'button' : undefined}
  tabindex={onactivate ? 0 : undefined}
  onclick={onactivate}
  onkeydown={handleKeydown}
>
  {#if leading}
    <div class="leading">{@render leading()}</div>
  {/if}
  <div class="content">{@render children()}</div>
  {#if trailing}
    <div class="trailing">{@render trailing()}</div>
  {/if}
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-3);
  }

  .muted {
    color: var(--color-ink-muted);
  }

  .interactive {
    cursor: pointer;
  }

  .content {
    flex: 1;
    min-width: 0;
  }
</style>
