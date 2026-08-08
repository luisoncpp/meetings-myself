import type { CheckInOutcome, DailyPlanView, PlanTaskView, TaskPoolView } from '../../../domain';
import * as api from '../../../api';

/**
 * State for the Daily Plan surface.
 *
 * A class rather than a bundle of hooks (docs/GUIDELINES.md): the surface has one
 * coherent state machine, and hooks would scatter it across effects.
 */
export class DailyPlanStore {
  #plan = $state<DailyPlanView | null>(null);
  #pool = $state<TaskPoolView | null>(null);
  #loading = $state(false);
  #error = $state<string | null>(null);

  get plan(): DailyPlanView | null {
    return this.#plan;
  }

  get pool(): TaskPoolView | null {
    return this.#pool;
  }

  get loading(): boolean {
    return this.#loading;
  }

  get error(): string | null {
    return this.#error;
  }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [plan, pool] = await Promise.all([api.todayView(), api.taskPool()]);
      this.#plan = plan;
      this.#pool = pool;
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
    } finally {
      this.#loading = false;
    }
  }

  /**
   * Optimistic: dragging must feel immediate. The server order wins on failure,
   * so a rejected permutation snaps back rather than lying.
   */
  async reorder(order: string[]): Promise<void> {
    const plan = this.#plan;
    if (!plan) return;

    const previous = plan.tasks;
    this.#plan = { ...plan, tasks: reindex(order, previous) };
    try {
      await api.reorderPlan(plan.date, order);
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
      this.#plan = { ...plan, tasks: previous };
    }
  }

  async select(taskId: string): Promise<void> {
    await this.#change(/* selectTheTask= */ () => api.selectIntoPlan(this.#date(), taskId));
  }

  // Used from DailyPlan.svelte template.
  // fallow-ignore-next-line unused-class-member
  async quickAdd(title: string): Promise<void> {
    await this.#change(/* createAndSelect= */ () => api.quickAddTask(title));
  }

  // Used from DailyPlan.svelte template.
  // fallow-ignore-next-line unused-class-member
  async checkIn(habitId: string, outcome: CheckInOutcome): Promise<void> {
    await this.#change(/* recordTheOutcome= */ () =>
      api.recordCheckIn(habitId, this.#date(), outcome),
    );
  }

  /** Completion is reversible, so this is a toggle, not a one-way action. */
  async toggleCompletion(task: PlanTaskView): Promise<void> {
    const reopening = task.state === 'completed';
    await this.#change(/* toggleTheOutcome= */ () =>
      reopening ? api.reopenTask(task.id) : api.completeTask(task.id),
    );
  }

  /** Runs a mutation, then reloads so every projection stays authoritative. */
  async #change(mutation: () => Promise<unknown>): Promise<void> {
    try {
      await mutation();
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
      return;
    }
    await this.load();
  }

  #date(): string {
    return this.#plan?.date ?? '';
  }
}

function reindex(order: string[], tasks: PlanTaskView[]): PlanTaskView[] {
  const byId = new Map(tasks.map((task) => [task.id, task]));
  return order
    .map((id, position) => {
      const task = byId.get(id);
      return task ? { ...task, position } : null;
    })
    .filter((task): task is PlanTaskView => task !== null);
}

function message(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure);
}
