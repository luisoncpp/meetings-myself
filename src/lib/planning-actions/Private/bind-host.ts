import type { PlanningActionsHost } from './planning-actions-host';

export function bindPlanningActionsHost(source: PlanningActionsHost): PlanningActionsHost {
  return {
    get focus() {
      return source.focus;
    },
    achieveGoal(goalId: string) {
      return source.achieveGoal(goalId);
    },
    addToFocus(taskId: string) {
      return source.addToFocus(taskId);
    },
    removeFromFocus(taskId: string) {
      return source.removeFromFocus(taskId);
    },
  };
}

