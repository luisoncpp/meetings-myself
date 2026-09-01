import type {
  AssociationEnd,
  Cadence,
  Classification,
  HabitStrength,
  LibraryView,
  Recurrence,
  RecurringTask,
  TaskView,
  WeeklyFocus,
} from '../../../domain';
import * as api from '../../../api';
import {
  createEntityFromPayload,
  type CreatePayload,
  type PlanningActionsHost,
} from '../../../planning-actions';
import { libraryStoreMessage, runLibraryMutation } from './library-store-change';

export class LibraryStore implements PlanningActionsHost {
  #view = $state<LibraryView | null>(null);
  #recurringTasks = $state<RecurringTask[]>([]);
  #includeArchived = $state(false);
  #loading = $state(false);
  #error = $state<string | null>(null);
  #week = $state('');
  #focus = $state<WeeklyFocus | null>(null);

  get view(): LibraryView | null {
    return this.#view;
  }

  // Used from Library.svelte template.
  // fallow-ignore-next-line unused-class-member
  get includeArchived(): boolean {
    return this.#includeArchived;
  }

  get loading(): boolean {
    return this.#loading;
  }

  get error(): string | null {
    return this.#error;
  }

  get week(): string {
    return this.#week;
  }

  get focus(): WeeklyFocus | null {
    return this.#focus;
  }

  // Used from RecurringTaskSection.svelte template.
  get recurringTasks(): RecurringTask[] {
    if (this.#includeArchived) {
      return this.#recurringTasks;
    }
    return this.#recurringTasks.filter((task) => task.lifecycle === 'active');
  }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [view, today, recurring] = await Promise.all([
        api.library(this.#includeArchived),
        api.todayView(),
        api.recurringTasks(),
      ]);
      this.#view = view;
      this.#recurringTasks = recurring;
      this.#week = today.week;
      this.#focus = await api.weeklyFocus(today.week);
      this.#error = null;
    } catch (failure) {
      this.#error = libraryStoreMessage(failure);
    } finally {
      this.#loading = false;
    }
  }

  async setIncludeArchived(show: boolean): Promise<void> {
    this.#includeArchived = show;
    await this.load();
  }

  async createEntity(payload: CreatePayload): Promise<void> {
    await this.#change(/* createEntity= */ () => createEntityFromPayload(payload));
  }

  // Used from Library.svelte script.
  // fallow-ignore-next-line unused-class-member
  async createValue(title: string): Promise<void> {
    await this.createEntity({ kind: 'value', title });
  }

  // Used from Library.svelte script.
  // fallow-ignore-next-line unused-class-member
  async createGoal(title: string, targetDate?: string | null): Promise<void> {
    await this.createEntity({ kind: 'goal', title, targetDate: targetDate ?? null });
  }

  // Used from Library.svelte script.
  // fallow-ignore-next-line unused-class-member
  async createHabit(title: string, cadence: Cadence): Promise<void> {
    await this.createEntity({ kind: 'habit', title, cadence });
  }

  // Used from Library.svelte script.
  async createTask(title: string, oneOff: boolean): Promise<void> {
    await this.createEntity({ kind: 'task', title, oneOff });
  }

  async createRecurringTask(title: string, recurrence: Recurrence): Promise<void> {
    await this.#change(/* createRecurringTask= */ () => api.createRecurringTask(title, recurrence));
  }

  async archiveRecurringTask(id: string): Promise<void> {
    await this.#change(/* archiveRecurringTask= */ () => api.archiveRecurringTask(id));
  }

  async restoreRecurringTask(id: string): Promise<void> {
    await this.#change(/* restoreRecurringTask= */ () => api.restoreRecurringTask(id));
  }

  async renameRecurringTask(id: string, title: string): Promise<void> {
    await this.#change(/* renameRecurringTask= */ () => api.renameRecurringTask(id, title));
  }

  async archive(end: AssociationEnd): Promise<void> {
    await this.#change(/* archive= */ () => api.archiveEntity(end));
  }

  async restore(end: AssociationEnd): Promise<void> {
    await this.#change(/* restore= */ () => api.restoreEntity(end));
  }

  async achieveGoal(goalId: string): Promise<void> {
    await this.#change(/* achieve= */ () => api.achieveGoal(goalId));
  }

  async unachieveGoal(goalId: string): Promise<void> {
    await this.#change(/* unachieve= */ () => api.unachieveGoal(goalId));
  }

  async classifyTask(
    taskId: string,
    importance: Classification,
    urgency: Classification,
  ): Promise<void> {
    await this.#change(/* classify= */ () => api.classifyTask(taskId, importance, urgency));
  }

  async setDeadline(taskId: string, deadline: string | null): Promise<void> {
    await this.#change(/* setDeadline= */ () => api.setTaskDeadline(taskId, deadline));
  }

  async setTaskOneOff(taskId: string, oneOff: boolean): Promise<void> {
    await this.#change(/* setTaskOneOff= */ () => api.setTaskOneOff(taskId, oneOff));
  }

  async toggleTask(task: TaskView): Promise<void> {
    const reopening = task.state === 'completed';
    await this.#change(/* toggleTask= */ () =>
      reopening ? api.reopenTask(task.id) : api.completeTask(task.id),
    );
  }

  async setHabitStrength(habitId: string, strength: HabitStrength): Promise<void> {
    await this.#change(/* setStrength= */ () => api.setHabitStrength(habitId, strength));
  }

  async setHabitCadence(habitId: string, cadence: Cadence): Promise<void> {
    await this.#change(/* setCadence= */ () => api.setHabitCadence(habitId, cadence));
  }

  async setHabitPinned(habitId: string, pinned: boolean): Promise<void> {
    await this.#change(/* setPinned= */ () => api.setHabitPinned(habitId, pinned));
  }

  async link(left: AssociationEnd, right: AssociationEnd): Promise<void> {
    await this.#change(/* link= */ () => api.link(left, right));
  }

  async unlink(associationId: string): Promise<void> {
    await this.#change(/* unlink= */ () => api.unlink(associationId));
  }

  async addToFocus(taskId: string): Promise<void> {
    await this.#change(/* addToFocus= */ () => api.addToFocus(this.#week, taskId));
  }

  async removeFromFocus(taskId: string): Promise<void> {
    await this.#change(/* removeFromFocus= */ () => api.removeFromFocus(this.#week, taskId));
  }

  async #change(mutation: () => Promise<unknown>): Promise<void> {
    await runLibraryMutation(
      mutation,
      /* setError= */ (error) => {
        this.#error = error;
      },
      /* reload= */ () => this.load(),
    );
  }
}
