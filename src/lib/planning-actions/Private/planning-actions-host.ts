import type { AssociationEnd, WeeklyFocus } from '../../domain';

export interface PlanningActionsHost {
  readonly focus: WeeklyFocus | null;
  achieveGoal(goalId: string): Promise<void>;
  link(left: AssociationEnd, right: AssociationEnd): Promise<void>;
  unlink(associationId: string): Promise<void>;
  addToFocus(taskId: string): Promise<void>;
  removeFromFocus(taskId: string): Promise<void>;
}
