import type { LibraryView, WeeklyFocus, WeeklyReviewView } from '../../../domain';
import * as api from '../../../api';
import type { PlanningActionsHost } from '../../../planning-actions';
import { nextWeek } from './week-nav';
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

  get view(): WeeklyReviewView | null { return this.#view; }
  get library(): LibraryView | null { return this.#library; }
  get focus(): WeeklyFocus | null { return this.#focus; }
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

  async createGoal(title: string, targetDate?: string | null): Promise<void> {
    await this.#mutate(/*createGoal=*/ () => api.createGoal(title, targetDate));
  }

  async achieveGoal(goalId: string): Promise<void> {
    await this.#mutate(/*achieve=*/ () => api.achieveGoal(goalId));
  }

  async link(
    left: { kind: 'value' | 'goal' | 'habit' | 'task'; id: string },
    right: { kind: 'value' | 'goal' | 'habit' | 'task'; id: string },
  ): Promise<void> {
    await this.#mutate(/*link=*/ () => api.link(left, right));
  }

  async unlink(associationId: string): Promise<void> {
    await this.#mutate(/*unlink=*/ () => api.unlink(associationId));
  }

  async addToFocus(taskId: string): Promise<void> {
    const week = this.focusWeek;
    if (week === '') return;
    await this.#mutate(/*add=*/ () => api.addToFocus(week, taskId));
  }

  async removeFromFocus(taskId: string): Promise<void> {
    const week = this.focusWeek;
    if (week === '') return;
    await this.#mutate(/*remove=*/ () => api.removeFromFocus(week, taskId));
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
