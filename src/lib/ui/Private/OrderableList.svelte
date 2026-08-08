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
  }

  let { items, getId, onreorder, children }: Props = $props();

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

<div role="listbox" class="list">
  {#each items as item, index (getId(item))}
    <div
      bind:this={rowRefs[index]}
      role="option"
      aria-label={getId(item)}
      aria-selected={index === focusedIndex}
      tabindex={index === focusedIndex ? 0 : -1}
      class="option"
      class:dragging={dragId === getId(item)}
      draggable={/*enable pointer reorder=*/true}
      onfocus={/*sync roving tabindex=*/() => {
        focusedIndex = index;
      }}
      onkeydown={(event) => handleKeydown(event, index)}
      ondragstart={(event) => handleDragStart(event, getId(item))}
      ondragover={handleDragOver}
      ondrop={(event) => handleDrop(event, getId(item))}
      ondragend={handleDragEnd}
    >
      {@render children(item)}
    </div>
  {/each}
</div>

<div role="status" aria-live="polite" class="status">{statusMessage}</div>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .option {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-lift);
    color: var(--color-ink);
    cursor: grab;
    transition: opacity var(--duration-state) var(--ease-out);
  }

  .option:focus-visible {
    outline: none;
    box-shadow: var(--focus-ring);
  }

  .dragging {
    opacity: 0.6;
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
