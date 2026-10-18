export interface PlanningActionsHost {
  achieveGoal(goalId: string): Promise<void>;
}
