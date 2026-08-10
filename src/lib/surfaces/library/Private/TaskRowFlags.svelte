<script lang="ts">
  import type { TaskView } from '../../../domain';
  import { StateFlag } from '../../../ui';

  interface Props {
    task: TaskView;
  }

  let { task }: Props = $props();
</script>

<div class="flags">
  {#if task.archived}
    <StateFlag kind="archived" />
  {/if}
  {#if task.overdue}
    <StateFlag kind="overdue" />
  {/if}
</div>

<style>
  .flags {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
  }
</style>
