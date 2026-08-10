import type { LibraryView, WeeklyFocus, WeeklyReviewView } from '../../../domain';
import { localizeError } from '../../../i18n';
import * as api from '../../../api';
import { nextWeek } from './week-nav';

export type SaveState = 'saved' | 'unsaved' | 'saving';

export const REFLECTION_DEBOUNCE_MS = 2000;

export function errorMessage(failure: unknown): string {
  return localizeError(failure instanceof Error ? failure.message : String(failure));
}

export async function loadReviewSideData(week: string): Promise<{
  library: LibraryView;
  focus: WeeklyFocus;
}> {
  const [library, focus] = await Promise.all([
    api.library(/*includeArchived=*/false),
    api.weeklyFocus(nextWeek(week)),
  ]);
  return { library, focus };
}

async function persistReflection(
  week: string,
  draft: string,
  savedReflection: string,
): Promise<{ ok: true; reflection: string } | { ok: false; error: string }> {
  if (draft === savedReflection) {
    return { ok: true, reflection: savedReflection };
  }
  try {
    await api.saveReflection(week, draft);
    return { ok: true, reflection: draft };
  } catch (failure) {
    return { ok: false, error: errorMessage(failure) };
  }
}

async function reloadReviewView(
  week: string,
  preserveReflection: string,
): Promise<WeeklyReviewView> {
  const review = await api.openWeeklyReview(week);
  return { ...review, reflection: preserveReflection };
}

export function createDebounce(delayMs: number) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return {
    schedule(callback: () => void): void {
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(callback, delayMs);
    },
    clear(): void {
      if (timer === null) return;
      clearTimeout(timer);
      timer = null;
    },
  };
}

export type WeeklyReviewMutationContext = {
  reloadLibrary: () => Promise<void>;
  week: string | undefined;
  preserveReflection: string;
};

export async function runWeeklyReviewMutation(
  mutation: () => Promise<unknown>,
  context: WeeklyReviewMutationContext,
): Promise<{ error: string | null; view: WeeklyReviewView | null }> {
  try {
    await mutation();
    await context.reloadLibrary();
    if (!context.week) return { error: null, view: null };
    const view = await reloadReviewView(context.week, context.preserveReflection);
    return { error: null, view };
  } catch (failure) {
    return { error: errorMessage(failure), view: null };
  }
}

export async function commitReflectionSave(
  week: string,
  draft: string,
  savedReflection: string,
): Promise<{ status: 'saved'; reflection: string } | { status: 'failed'; error: string }> {
  const result = await persistReflection(week, draft, savedReflection);
  if (!result.ok) return { status: 'failed', error: result.error };
  return { status: 'saved', reflection: result.reflection };
}
