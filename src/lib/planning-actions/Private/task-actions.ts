import * as api from '../../api';
import type { PlanTaskView, TaskView } from '../../domain';

/** What a task menu needs to know about the row it opened on. */
export interface TaskRef {
  id: string;
  title: string;
  done: boolean;
  archived: boolean;
  /** Only meaningful where a Daily Plan is shown. */
  inToday?: boolean;
}

export function taskRef(task: TaskView): TaskRef {
  return { id: task.id, title: task.title, done: task.completedOn !== null, archived: task.archived };
}

export function planTaskRef(task: PlanTaskView): TaskRef {
  return { id: task.id, title: task.title, done: task.completed, archived: task.archived, inToday: true };
}

/** A Weekly Focus the menu can add to or remove from. */
export interface FocusTarget {
  week: string;
  which: 'thisWeek' | 'nextWeek';
  taskIds: ReadonlySet<string>;
}

export interface TaskActionsConfig {
  /** Runs a mutation, then reloads the owning surface (pessimistic). */
  run: (mutation: () => Promise<unknown>) => Promise<void>;
  focusTargets: () => FocusTarget[];
  /** Date of the Daily Plan in view; enables add/remove today. */
  planDate?: () => string;
}

/**
 * Task operations shared by every surface's task menu and rows. Each surface
 * owns one, wired to its own reload, so the menu never mutates state itself.
 */
export class TaskActions {
  readonly #config: TaskActionsConfig;

  constructor(config: TaskActionsConfig) {
    this.#config = config;
  }

  get focusTargets(): FocusTarget[] {
    return this.#config.focusTargets();
  }

  get canPlanToday(): boolean {
    return this.#config.planDate !== undefined;
  }

  /** Done, then archived — one-offs archive themselves; ongoing tasks need the second call. */
  async finish(task: TaskRef): Promise<void> {
    const on = this.#config.planDate?.();
    await this.#config.run(/* completeThenArchive= */ async () => {
      await api.completeTask(task.id, on);
      await api.archiveEntity({ kind: 'task', id: task.id });
    });
  }

  /** Row checkbox: a done one-off archives itself; reopening restores it. */
  async toggleDone(task: TaskRef): Promise<void> {
    const on = this.#config.planDate?.();
    await this.#config.run(/* toggleOutcome= */ () =>
      task.done ? api.reopenTask(task.id) : api.completeTask(task.id, on),
    );
  }

  async reopen(taskId: string): Promise<void> {
    await this.#config.run(/* reopen= */ () => api.reopenTask(taskId));
  }

  async setFocus(target: FocusTarget, taskId: string): Promise<void> {
    const remove = target.taskIds.has(taskId);
    await this.#config.run(/* toggleFocus= */ () =>
      remove ? api.removeFromFocus(target.week, taskId) : api.addToFocus(target.week, taskId),
    );
  }

  async rename(taskId: string, title: string): Promise<void> {
    await this.#config.run(/* rename= */ () => api.renameTask(taskId, title));
  }

  async setArchived(taskId: string, archived: boolean): Promise<void> {
    const end = { kind: 'task' as const, id: taskId };
    await this.#config.run(/* archiveOrRestore= */ () =>
      archived ? api.archiveEntity(end) : api.restoreEntity(end),
    );
  }

  async addToToday(taskId: string): Promise<void> {
    const date = this.#config.planDate?.() ?? '';
    await this.#config.run(/* select= */ () => api.selectIntoPlan(date, taskId));
  }

  async removeFromToday(taskId: string): Promise<void> {
    const date = this.#config.planDate?.() ?? '';
    await this.#config.run(/* unselect= */ () => api.removeFromPlan(date, taskId));
  }
}
