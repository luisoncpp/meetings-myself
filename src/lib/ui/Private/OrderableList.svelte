<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';
  import {
    formatPositionAnnouncement,
    move,
    ordersMatch,
    reorderByDrop,
  } from './reorder';

  interface Props {
    items: T[];
    getId: (item: T) => string;
    onreorder: (order: string[]) => void;
    children: Snippet<[T]>;
    label?: string;
  }

  let {
    items,
    getId,
    onreorder,
    children,
    label = 'Reorderable list',
  }: Props = $props();

  let focusedIndex = $state(0);
  let statusMessage = $state('');
  let dragId = $state<string | null>(null);
  let rowRefs: (HTMLElement | undefined)[] = $state([]);

  function currentOrder(): string[] {
    return items.map(getId);
  }

  function applyReorder(next: string[], movedId: string): void {
    onreorder(next);
    statusMessage = formatPositionAnnouncement(movedId, next);
  }

  function focusRow(index: number): void {
    focusedIndex = index;
    rowRefs[index]?.focus();
  }

  function handleKeyboardReorder(index: number, direction: 'up' | 'down'): void {
    const order = currentOrder();
    const id = order[index]!;
    const next = move(order, { id, direction });
    if (ordersMatch(order, next)) return;
    focusRow(next.indexOf(id));
    applyReorder(next, id);
  }

  function handleKeydown(event: KeyboardEvent, index: number): void {
    if (event.altKey && event.key === 'ArrowUp') {
      event.preventDefault();
      handleKeyboardReorder(index, 'up');
      return;
    }
    if (event.altKey && event.key === 'ArrowDown') {
      event.preventDefault();
      handleKeyboardReorder(index, 'down');
      return;
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      if (index > 0) focusRow(index - 1);
      return;
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      if (index < items.length - 1) focusRow(index + 1);
    }
  }

  function handleDragStart(event: DragEvent, id: string): void {
    dragId = id;
    event.dataTransfer?.setData('text/plain', id);
    if (event.dataTransfer) event.dataTransfer.effectAllowed = 'move';
  }

  function handleDragOver(event: DragEvent): void {
    event.preventDefault();
  }

  function handleDrop(event: DragEvent, targetId: string): void {
    event.preventDefault();
    const sourceId = dragId;
    dragId = null;
    if (!sourceId) return;

    const order = currentOrder();
    const next = reorderByDrop(order, sourceId, targetId);
    if (ordersMatch(order, next)) return;
    applyReorder(next, sourceId);
  }

  function handleDragEnd(): void {
    dragId = null;
  }
</script>

<ul aria-label={label} class="list">
  {#each items as item, index (getId(item))}
  {@const id = getId(item)}
    <li
      bind:this={rowRefs[index]}
      aria-label={id}
      aria-grabbed={dragId === id ? true : undefined}
      tabindex={index === focusedIndex ? 0 : -1}
      class="item"
      class:dragging={dragId === id}
      onfocus={/*sync roving tabindex=*/() => {
        focusedIndex = index;
      }}
      onkeydown={(event) => handleKeydown(event, index)}
      ondragover={handleDragOver}
      ondrop={(event) => handleDrop(event, id)}
    >
      <button
        type="button"
        class="handle"
        tabindex={-1}
        aria-label="Reorder {id}"
        draggable={/*enable pointer reorder=*/true}
        ondragstart={(event) => handleDragStart(event, id)}
        ondragend={handleDragEnd}
      >
        <span class="grip" aria-hidden="true">⋮⋮</span>
      </button>
      <div class="content">
        {@render children(item)}
      </div>
    </li>
  {/each}
</ul>

<div role="status" aria-live="polite" class="status">{statusMessage}</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .item {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    border-radius: var(--radius-control);
    transition: opacity var(--duration-state) var(--ease-out);
  }

  .item:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .dragging {
    opacity: 0.6;
  }

  .handle {
    flex-shrink: 0;
    padding: var(--space-2);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: grab;
    transition: color var(--duration-fast) var(--ease-out);
  }

  .handle:hover {
    color: var(--color-ink);
  }

  .handle:active {
    cursor: grabbing;
  }

  .grip {
    display: block;
    font-size: var(--text-label);
    line-height: 1;
  }

  .content {
    flex: 1;
    min-width: 0;
  }

  .status {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
