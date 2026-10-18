<script lang="ts" module>
  /** `current` = this week's focus, `next` = next week's (shown as a ring). */
  export type FocusMark = 'none' | 'current' | 'next';
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';
  import { t } from '../../i18n';
  import type { MenuPoint } from './ContextMenu.svelte';

  interface Props {
    title: string;
    done?: boolean;
    focus?: FocusMark;
    selected?: boolean;
    muted?: boolean;
    /** Shows the round completion check when provided. */
    ontoggle?: () => void;
    /** Makes the title a button, e.g. to open a detail panel. */
    onselect?: () => void;
    /** Right-click, the Menu key, or the "more" button open the task menu. */
    onmenu?: (point: MenuPoint) => void;
    meta?: Snippet;
    trailing?: Snippet;
  }

  let {
    title,
    done = false,
    focus = 'none',
    selected = false,
    muted = false,
    ontoggle,
    onselect,
    onmenu,
    meta,
    trailing,
  }: Props = $props();

  const focusLabel = $derived(
    focus === 'current' ? t('taskLine.inFocus') : focus === 'next' ? t('taskLine.inNextFocus') : '',
  );

  function openMenu(event: MouseEvent): void {
    if (!onmenu) return;
    event.preventDefault();
    onmenu({ x: event.clientX, y: event.clientY });
  }

  function openMenuFromButton(event: MouseEvent): void {
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    onmenu?.({ x: rect.left, y: rect.bottom });
  }
</script>

<div
  class="line"
  class:focus={focus === 'current'}
  class:selected
  class:muted={muted || done}
  role="group"
  aria-label={title}
  oncontextmenu={openMenu}
>
  {#if ontoggle}
    <button
      type="button"
      class="check"
      class:done
      role="checkbox"
      aria-checked={done}
      aria-label={done ? t('common.reopen', { title }) : t('common.markDoneFor', { title })}
      onclick={/* toggle completion */ ontoggle}
    ></button>
  {/if}
  {#if onselect}
    <button type="button" class="title as-button" class:done onclick={/* select */ onselect}>
      {title}
    </button>
  {:else}
    <span class="title" class:done>{title}</span>
  {/if}
  {#if meta}
    <span class="meta">{@render meta()}</span>
  {/if}
  {#if focus !== 'none'}
    <span class="dot" class:ring={focus === 'next'} title={focusLabel}>
      <span class="sr-only">{focusLabel}</span>
    </span>
  {/if}
  {#if trailing}
    {@render trailing()}
  {/if}
  {#if onmenu}
    <button
      type="button"
      class="more"
      aria-label={t('taskLine.more', { title })}
      onclick={openMenuFromButton}
    >
      ⋯
    </button>
  {/if}
</div>

<style>
  .line {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    min-height: 2.25rem;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-control);
    transition: background-color var(--duration-fast) var(--ease-out);
  }

  .line:hover,
  .selected {
    background: var(--color-raised);
  }

  .focus {
    background: var(--color-gold-wash);
  }

  .muted {
    color: var(--color-ink-muted);
  }

  .check {
    flex-shrink: 0;
    width: 1rem;
    height: 1rem;
    padding: 0;
    border: 1.5px solid var(--color-ink-muted);
    border-radius: 50%;
    background: none;
    cursor: pointer;
  }

  .check:hover {
    border-color: var(--color-done);
  }

  .check.done {
    border-color: var(--color-done);
    background: var(--color-done);
  }

  .title {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .as-button {
    padding: 0;
    border: none;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .done {
    text-decoration: line-through;
  }

  .meta {
    display: flex;
    flex-shrink: 0;
    gap: var(--space-1);
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .dot {
    flex-shrink: 0;
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 50%;
    background: var(--color-gold);
  }

  .dot.ring {
    background: none;
    box-shadow: inset 0 0 0 1.5px var(--color-gold);
  }

  .more {
    flex-shrink: 0;
    padding: 0 var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: pointer;
    opacity: 0;
  }

  .line:hover .more,
  .more:focus-visible {
    opacity: 1;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
</style>
