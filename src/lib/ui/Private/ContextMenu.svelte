<script lang="ts" module>
  export interface MenuItem {
    id: string;
    label: string;
    onselect: () => void;
    /** The item swaps the menu's content (e.g. a rename form) instead of closing it. */
    keepOpen?: boolean;
  }

  export interface MenuPoint {
    x: number;
    y: number;
  }
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    point: MenuPoint;
    label: string;
    items: MenuItem[];
    onclose: () => void;
    /** Replaces the item list, e.g. for an inline rename form. */
    children?: Snippet;
  }

  let { point, label, items, onclose, children }: Props = $props();

  let menu: HTMLDivElement | undefined = $state();
  const returnFocus = document.activeElement as HTMLElement | null;

  const left = $derived(Math.max(0, Math.min(point.x, window.innerWidth - 240)));
  const top = $derived(Math.max(0, Math.min(point.y, window.innerHeight - 280)));

  $effect(/* focusFirstControlWhenContentChanges */ () => {
    void children;
    menu?.querySelector<HTMLElement>('button, input')?.focus();
  });

  $effect(/* restoreFocusOnClose */ () => () => returnFocus?.focus?.());

  function buttons(): HTMLButtonElement[] {
    return [...(menu?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]') ?? [])];
  }

  function moveFocus(step: number): void {
    const all = buttons();
    const index = all.indexOf(document.activeElement as HTMLButtonElement);
    all[(index + step + all.length) % all.length]?.focus();
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape' || event.key === 'Tab') {
      event.preventDefault();
      onclose();
      return;
    }
    if (children) return;
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    event.preventDefault();
    moveFocus(event.key === 'ArrowDown' ? 1 : -1);
  }

  function choose(item: MenuItem): void {
    if (!item.keepOpen) onclose();
    item.onselect();
  }

  function dismiss(event: Event): void {
    event.preventDefault();
    onclose();
  }
</script>

<div class="scrim" role="presentation" onclick={dismiss} oncontextmenu={dismiss}></div>
<div
  bind:this={menu}
  class="menu"
  role="menu"
  tabindex="-1"
  aria-label={label}
  style:left="{left}px"
  style:top="{top}px"
  onkeydown={handleKeydown}
>
  {#if children}
    {@render children()}
  {:else}
    {#each items as item (item.id)}
      <button type="button" role="menuitem" onclick={/* run item */ () => choose(item)}>
        {item.label}
      </button>
    {/each}
  {/if}
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 20;
  }

  .menu {
    position: fixed;
    z-index: 21;
    min-width: 12rem;
    max-width: 15rem;
    padding: var(--space-1);
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    box-shadow: var(--shadow-hover);
  }

  button {
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  button:hover,
  button:focus-visible {
    background: var(--color-lift);
  }
</style>
