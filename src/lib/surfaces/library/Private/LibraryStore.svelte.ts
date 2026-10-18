import { SvelteSet } from 'svelte/reactivity';
import type {
  AssociationEnd,
  Cadence,
  Classification,
  HabitStrength,
  LibraryView,
  Recurrence,
  RecurringTask,
  WeeklyFocus,
} from '../../../domain';
import { nextWeek } from '../../../domain';
import * as api from '../../../api';
import { createEntityFromPayload, TaskActions, type CreatePayload } from '../../../planning-actions';
import { libraryStoreMessage, runLibraryMutation } from './library-store-change';

export type LibraryTab = 'tasks' | 'habits' | 'goals' | 'values' | 'recurring' | 'archive';

/**
 * Loads everything, archived included, once: active tabs filter archived rows
 * out and the Archive tab shows only them, so switching tabs never refetches.
 */
export class LibraryStore {
  #view = $state<LibraryView | null>(null);
  #recurring = $state<RecurringTask[]>([]);
  #loading = $state(false);
  #error = $state<string | null>(null);
  #focus = $state<WeeklyFocus | null>(null);
  #nextFocus = $state<WeeklyFocus | null>(null);
  #tab = $state<LibraryTab>('tasks');
  #selected = $state<AssociationEnd | null>(null);

  /** Task menu operations; focus edits target this week or next. */
  readonly actions = new TaskActions({
    run: (mutation) => this.#change(mutation),
    focusTargets: () => [
      { week: this.#focus?.week ?? '', which: 'thisWeek', taskIds: this.focusIds },
      { week: this.#nextFocus?.week ?? '', which: 'nextWeek', taskIds: this.nextFocusIds },
    ],
  });

  get view(): LibraryView | null { return this.#view; }
  get loading(): boolean { return this.#loading; }
  // Used from Library.svelte template.
  // fallow-ignore-next-line unused-class-member
  get error(): string | null { return this.#error; }
  get tab(): LibraryTab { return this.#tab; }
  get selected(): AssociationEnd | null { return this.#selected; }
  get focusIds(): ReadonlySet<string> { return new SvelteSet(this.#focus?.tasks ?? []); }
  get nextFocusIds(): ReadonlySet<string> { return new SvelteSet(this.#nextFocus?.tasks ?? []); }

  get recurringTasks(): RecurringTask[] {
    return this.#recurring.filter((rule) => rule.lifecycle === 'active');
  }

  get archivedRecurringTasks(): RecurringTask[] {
    return this.#recurring.filter((rule) => rule.lifecycle === 'archived');
  }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [view, today, recurring] = await Promise.all([
        api.library(/*includeArchived=*/ true),
        api.todayView(),
        api.recurringTasks(),
      ]);
      const [focus, next] = await Promise.all([
        api.weeklyFocus(today.week),
        api.weeklyFocus(nextWeek(today.week)),
      ]);
      this.#view = view;
      this.#recurring = recurring;
      this.#focus = focus;
      this.#nextFocus = next;
      this.#error = null;
    } catch (failure) {
      this.#error = libraryStoreMessage(failure);
    } finally {
      this.#loading = false;
    }
  }

  setTab(tab: LibraryTab): void {
    this.#tab = tab;
    this.#selected = null;
  }

  select(end: AssociationEnd | null): void {
    this.#selected = end;
  }

  async createEntity(payload: CreatePayload): Promise<void> {
    await this.#change(/* createEntity= */ () => createEntityFromPayload(payload));
  }

  async createRecurringTask(title: string, recurrence: Recurrence): Promise<void> {
    await this.#change(/* createRecurringTask= */ () => api.createRecurringTask(title, recurrence));
  }

  async setRecurringArchived(id: string, archived: boolean): Promise<void> {
    await this.#change(/* archiveOrRestoreRule= */ () =>
      archived ? api.archiveRecurringTask(id) : api.restoreRecurringTask(id),
    );
  }

  async renameRecurringTask(id: string, title: string): Promise<void> {
    await this.#change(/* renameRecurringTask= */ () => api.renameRecurringTask(id, title));
  }

  async setArchived(end: AssociationEnd, archived: boolean): Promise<void> {
    await this.#change(/* archiveOrRestore= */ () =>
      archived ? api.archiveEntity(end) : api.restoreEntity(end),
    );
  }

  async setGoalAchieved(goalId: string, achieved: boolean): Promise<void> {
    await this.#change(/* achieveOrUnachieve= */ () =>
      achieved ? api.achieveGoal(goalId) : api.unachieveGoal(goalId),
    );
  }

  async classifyTask(taskId: string, importance: Classification, urgency: Classification): Promise<void> {
    await this.#change(/* classify= */ () => api.classifyTask(taskId, importance, urgency));
  }

  async setDeadline(taskId: string, deadline: string | null): Promise<void> {
    await this.#change(/* setDeadline= */ () => api.setTaskDeadline(taskId, deadline));
  }

  async setTaskOneOff(taskId: string, oneOff: boolean): Promise<void> {
    await this.#change(/* setTaskOneOff= */ () => api.setTaskOneOff(taskId, oneOff));
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

  async renameHabit(habitId: string, title: string): Promise<void> {
    await this.#change(/* renameHabit= */ () => api.renameHabit(habitId, title));
  }

  async link(left: AssociationEnd, right: AssociationEnd): Promise<void> {
    await this.#change(/* link= */ () => api.link(left, right));
  }

  async unlink(associationId: string): Promise<void> {
    await this.#change(/* unlink= */ () => api.unlink(associationId));
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
