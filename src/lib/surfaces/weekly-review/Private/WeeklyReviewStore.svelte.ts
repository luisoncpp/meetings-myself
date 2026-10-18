import { SvelteSet } from 'svelte/reactivity';
import type { LibraryView, WeeklyFocus, WeeklyReviewView } from '../../../domain';
import * as api from '../../../api';
import {
  createEntityFromPayload,
  TaskActions,
  type CreatePayload,
  type PlanningActionsHost,
} from '../../../planning-actions';

import { nextWeek } from '../../../domain';
import {
  commitReflectionSave,
  createDebounce,
  errorMessage,
  loadReviewSideData,
  REFLECTION_DEBOUNCE_MS,
  runWeeklyReviewMutation,
  type SaveState,
} from './weekly-review-store-support';

export type { SaveState };

export class WeeklyReviewStore implements PlanningActionsHost {
  #view = $state<WeeklyReviewView | null>(null);
  #currentWeek = $state('');
  #library = $state<LibraryView | null>(null);
  #focus = $state<WeeklyFocus | null>(null);
  #draftReflection = $state('');
  #saveState = $state<SaveState>('saved');
  #error = $state<string | null>(null);
  #loading = $state(false);
  #autosave = createDebounce(REFLECTION_DEBOUNCE_MS);

  /** Task menu operations; focus edits target the week after the one reviewed. */
  readonly actions = new TaskActions({
    run: (mutation) => this.#mutate(mutation),
    focusTargets: () => [{ week: this.focusWeek, which: 'nextWeek', taskIds: this.focusIds }],
  });

  get view(): WeeklyReviewView | null { return this.#view; }
  get library(): LibraryView | null { return this.#library; }
  get focusIds(): ReadonlySet<string> { return new SvelteSet(this.#focus?.tasks ?? []); }
  get draftReflection(): string { return this.#draftReflection; }
  get saveState(): SaveState { return this.#saveState; }
  // Used from WeeklyReview.svelte template.
  // fallow-ignore-next-line unused-class-member
  get loading(): boolean { return this.#loading; }
  // Used from WeeklyReview.svelte template.
  // fallow-ignore-next-line unused-class-member
  get error(): string | null { return this.#error; }
  get isHistorical(): boolean {
    return this.#view !== null && this.#currentWeek !== '' && this.#view.week !== this.#currentWeek;
  }
  get focusWeek(): string { return this.#view ? nextWeek(this.#view.week) : ''; }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [review, today] = await Promise.all([api.openCurrentReview(), api.todayView()]);
      this.#currentWeek = today.week;
      await this.#applyReview(review);
      this.#error = null;
    } catch (failure) {
      this.#error = errorMessage(failure);
    } finally {
      this.#loading = false;
    }
  }

  async openWeek(week: string): Promise<void> {
    this.#loading = true;
    try {
      const review = await api.openWeeklyReview(week);
      if (this.#currentWeek === '') {
        const today = await api.todayView();
        this.#currentWeek = today.week;
      }
      await this.#applyReview(review);
      this.#error = null;
    } catch (failure) {
      this.#error = errorMessage(failure);
    } finally {
      this.#loading = false;
    }
  }

  setReflection(text: string): void {
    this.#draftReflection = text;
    if (text === this.#view?.reflection) {
      this.#saveState = 'saved';
      this.#autosave.clear();
      return;
    }
    this.#saveState = 'unsaved';
    this.#autosave.schedule(/*autosave*/ () => {
      void this.#saveNow();
    });
  }

  // Used from WeeklyReview.svelte template.
  // fallow-ignore-next-line unused-class-member
  onReflectionBlur(): void {
    this.#autosave.clear();
    void this.#saveNow();
  }

  destroy(): void {
    this.#autosave.clear();
    if (this.#saveState !== 'unsaved') return;
    void this.#saveNow();
  }

  async reloadLibrary(): Promise<void> {
    if (!this.#view) return;
    try {
      const { library, focus } = await loadReviewSideData(this.#view.week);
      this.#library = library;
      this.#focus = focus;
      this.#error = null;
    } catch (failure) {
      this.#error = errorMessage(failure);
    }
  }

  async createEntity(payload: CreatePayload): Promise<void> {
    await this.#mutate(/*createEntity=*/ () => createEntityFromPayload(payload));
  }

  /** A task created while reviewing is meant for next week, so it lands in that focus. */
  async createTaskInFocus(title: string): Promise<void> {
    const week = this.focusWeek;
    await this.#mutate(/*createThenFocus=*/ async () => {
      const task = await api.createTask(title);
      if (week !== '') await api.addToFocus(week, task.id);
    });
  }

  async achieveGoal(goalId: string): Promise<void> {
    await this.#mutate(/*achieve=*/ () => api.achieveGoal(goalId));
  }

  async #applyReview(review: WeeklyReviewView): Promise<void> {
    this.#autosave.clear();
    this.#view = review;
    this.#draftReflection = review.reflection;
    this.#saveState = 'saved';
    const { library, focus } = await loadReviewSideData(review.week);
    this.#library = library;
    this.#focus = focus;
  }

  async #saveNow(): Promise<void> {
    const week = this.#view?.week;
    const savedReflection = this.#view?.reflection ?? '';
    if (!week || this.#draftReflection === savedReflection) {
      this.#saveState = 'saved';
      return;
    }
    this.#saveState = 'saving';
    const outcome = await commitReflectionSave(week, this.#draftReflection, savedReflection);
    if (outcome.status === 'failed') {
      this.#saveState = 'unsaved';
      this.#error = outcome.error;
      return;
    }
    if (this.#view) this.#view = { ...this.#view, reflection: outcome.reflection };
    this.#saveState = 'saved';
    this.#error = null;
  }

  async #mutate(mutation: () => Promise<unknown>): Promise<void> {
    const result = await runWeeklyReviewMutation(mutation, {
      reloadLibrary: () => this.reloadLibrary(),
      week: this.#view?.week,
      preserveReflection: this.#draftReflection,
    });
    if (result.error) {
      this.#error = result.error;
      return;
    }
    this.#error = null;
    if (result.view) this.#view = result.view;
  }
}
