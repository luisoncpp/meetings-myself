<script lang="ts">
  import type { TaskPoolView, TaskView } from '../../../domain';

  interface Props {
    pool: TaskPoolView;
    onselect: (taskId: string) => void;
  }

  let { pool, onselect }: Props = $props();
</script>

<div role="region" aria-label="Task pool" class="pool">
  <h2>Task Pool</h2>

  {#if pool.focus.length > 0}
    <h3>In this week's focus</h3>
    <ul class="tasks">
      {#each pool.focus as task (task.id)}
        <li>{@render poolRow(task)}</li>
      {/each}
    </ul>
  {/if}

  {#if pool.rest.length > 0}
    {#if pool.focus.length > 0}
      <h3 class="rest-heading">Everything else</h3>
    {/if}
    <ul class="tasks">
      {#each pool.rest as task (task.id)}
        <li>{@render poolRow(task)}</li>
      {/each}
    </ul>
  {/if}
</div>

{#snippet poolRow(task: TaskView)}
  <div class="row">
    <span class="title">{task.title}</span>
    <button
      type="button"
      class="add"
      aria-label="Add to today: {task.title}"
      onclick={/* add to today */ () => onselect(task.id)}
    >
      Add to today
    </button>
  </div>
{/snippet}

<style>
  h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  h3 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-body);
    font-weight: 600;
    color: var(--color-ink-muted);
  }

  .rest-heading {
    margin-top: var(--space-4);
  }

  .tasks {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .title {
    flex: 1;
    min-width: 0;
  }

  .add {
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-control);
    background: none;
    color: var(--color-ink-muted);
    font: inherit;
    cursor: pointer;
  }

  .add:hover {
    color: var(--color-ink);
  }
</style>
