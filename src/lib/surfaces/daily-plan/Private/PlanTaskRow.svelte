<script lang="ts">
  import type { PlanTaskView } from '../../../domain';
  import { TaskChips, TaskLine, type MenuPoint } from '../../../ui';

  interface Props {
    task: PlanTaskView;
    inFocus?: boolean;
    ontoggle: () => void;
    onmenu?: (point: MenuPoint) => void;
  }

  let { task, inFocus = false, ontoggle, onmenu }: Props = $props();
</script>

<TaskLine
  title={task.title}
  done={task.completed}
  focus={inFocus ? 'current' : 'none'}
  muted={task.archived}
  {ontoggle}
  {onmenu}
>
  {#snippet meta()}
    <TaskChips importance={task.importance} urgency={task.urgency} overdue={task.overdue} />
  {/snippet}
</TaskLine>
