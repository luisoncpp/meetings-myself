import type { LibraryView, WeeklyFocus, WeeklyReviewView } from '../../../domain';
import * as api from '../../../api';
import { nextWeek } from './week-nav';

export type SaveState = 'saved' | 'unsaved' | 'saving';

export class WeeklyReviewStore {
  #view = $state<WeeklyReviewView | null>(null);
  #currentWeek = $state('');
  #library = $state<LibraryView | null>(null);
  #focus = $state<WeeklyFocus | null>(null);
  #draftReflection = $state('');
  #saveState = $state<SaveState>('saved');
  #error = $state<string | null>(null);
  #loading = $state(false);
  #debounceTimer: ReturnType<typeof setTimeout> | null = null;

  get view(): WeeklyReviewView | null {
    return this.#view;
  }

  get currentWeek(): string {
    return this.#currentWeek;
  }

  get library(): LibraryView | null {
    return this.#library;
  }

  get focus(): WeeklyFocus | null {
    return this.#focus;
  }

  get draftReflection(): string {
    return this.#draftReflection;
  }

  get saveState(): SaveState {
    return this.#saveState;
  }

  get loading(): boolean {
    return this.#loading;
  }

  get error(): string | null {
    return this.#error;
  }

  get isHistorical(): boolean {
    return this.#view !== null && this.#currentWeek !== '' && this.#view.week !== this.#currentWeek;
  }

  get focusWeek(): string {
    return this.#view ? nextWeek(this.#view.week) : '';
  }

  async load(): Promise<void> {
    this.#loading = true;
    try {
      const [review, today] = await Promise.all([api.openCurrentReview(), api.todayView()]);
      this.#currentWeek = today.week;
      await this.#applyReview(review);
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
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
      this.#error = message(failure);
    } finally {
      this.#loading = false;
    }
  }

  setReflection(text: string): void {
    this.#draftReflection = text;
    if (text === this.#view?.reflection) {
      this.#saveState = 'saved';
      this.#clearDebounce();
      return;
    }
    this.#saveState = 'unsaved';
    this.#scheduleDebounce();
  }

  onReflectionBlur(): void {
    this.#clearDebounce();
    void this.#saveNow();
  }

  destroy(): void {
    this.#clearDebounce();
    if (this.#saveState !== 'unsaved') return;
    void this.#saveNow();
  }

  async reloadLibrary(): Promise<void> {
    if (!this.#view) return;
    try {
      const [library, focus] = await Promise.all([
        api.library(/*includeArchived=*/false),
        api.weeklyFocus(this.focusWeek),
      ]);
      this.#library = library;
      this.#focus = focus;
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
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
    this.#clearDebounce();
    this.#view = review;
    this.#draftReflection = review.reflection;
    this.#saveState = 'saved';
    const [library, focus] = await Promise.all([
      api.library(/*includeArchived=*/false),
      api.weeklyFocus(nextWeek(review.week)),
    ]);
    this.#library = library;
    this.#focus = focus;
  }

  #scheduleDebounce(): void {
    this.#clearDebounce();
    this.#debounceTimer = setTimeout(/*autosave*/ () => {
      void this.#saveNow();
    }, /*delayInMs=*/2000);
  }

  #clearDebounce(): void {
    if (this.#debounceTimer === null) return;
    clearTimeout(this.#debounceTimer);
    this.#debounceTimer = null;
  }

  async #saveNow(): Promise<void> {
    const week = this.#view?.week;
    if (!week || this.#draftReflection === this.#view?.reflection) {
      this.#saveState = 'saved';
      return;
    }
    this.#saveState = 'saving';
    try {
      await api.saveReflection(week, this.#draftReflection);
      if (this.#view) {
        this.#view = { ...this.#view, reflection: this.#draftReflection };
      }
      this.#saveState = 'saved';
      this.#error = null;
    } catch (failure) {
      this.#saveState = 'unsaved';
      this.#error = message(failure);
    }
  }

  async #mutate(mutation: () => Promise<unknown>): Promise<void> {
    try {
      await mutation();
      this.#error = null;
    } catch (failure) {
      this.#error = message(failure);
      return;
    }
    await this.reloadLibrary();
    if (!this.#view) return;
    const review = await api.openWeeklyReview(this.#view.week);
    this.#view = { ...review, reflection: this.#draftReflection };
  }
}

function message(failure: unknown): string {
  return failure instanceof Error ? failure.message : String(failure);
}
