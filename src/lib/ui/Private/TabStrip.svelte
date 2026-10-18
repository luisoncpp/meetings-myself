<script lang="ts" generics="Id extends string">
  interface Tab {
    id: Id;
    label: string;
  }

  interface Props {
    tabs: Tab[];
    current: Id;
    label: string;
    onchange: (id: Id) => void;
  }

  let { tabs, current, label, onchange }: Props = $props();

  let strip: HTMLDivElement | undefined = $state();

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key !== 'ArrowRight' && event.key !== 'ArrowLeft') return;
    event.preventDefault();
    const step = event.key === 'ArrowRight' ? 1 : -1;
    const index = tabs.findIndex((tab) => tab.id === current);
    const next = tabs[(index + step + tabs.length) % tabs.length]!;
    onchange(next.id);
    strip?.querySelector<HTMLElement>(`[data-tab="${next.id}"]`)?.focus();
  }
</script>

<div bind:this={strip} class="tabs" role="tablist" aria-label={label} tabindex="-1" onkeydown={handleKeydown}>
  {#each tabs as tab (tab.id)}
    <button
      type="button"
      role="tab"
      data-tab={tab.id}
      aria-selected={tab.id === current}
      tabindex={tab.id === current ? 0 : -1}
      class:selected={tab.id === current}
      onclick={/* switch tab */ () => onchange(tab.id)}
    >
      {tab.label}
    </button>
  {/each}
</div>

<style>
  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }

  button {
    padding: var(--space-1) var(--space-3);
    border: none;
    border-radius: var(--radius-pill);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: pointer;
    transition: color var(--duration-fast) var(--ease-out);
  }

  button:hover {
    color: var(--color-ink);
  }

  .selected {
    background: var(--color-raised);
    color: var(--color-gold);
  }
</style>
