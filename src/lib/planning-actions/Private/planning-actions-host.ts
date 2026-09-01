import type { WeeklyFocus } from '../../domain';

export interface PlanningActionsHost {
  readonly focus: WeeklyFocus | null;
  achieveGoal(goalId: string): Promise<void>;
  addToFocus(taskId: string): Promise<void>;
  removeFromFocus(taskId: string): Promise<void>;
}

