import type { PlanningActionsHost } from './planning-actions-host';

export function bindPlanningActionsHost(source: PlanningActionsHost): PlanningActionsHost {
  return {
    achieveGoal(goalId: string) {
      return source.achieveGoal(goalId);
    },
  };
}
