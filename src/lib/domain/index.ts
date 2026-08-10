export { formatPlanDate } from './Private/plan-date';

export type Classification = 'unclassified' | 'low' | 'high';
export type TaskState = 'open' | 'completed' | 'archived';
export type HabitStrength =
  | 'reminderDependent'
  | 'cueTriggered'
  | 'strengthening'
  | 'established';
export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';

export type Cadence = { kind: 'everyDay' } | { kind: 'onWeekdays'; days: Weekday[] };

export type AssociationEnd =
  | { kind: 'value'; id: string }
  | { kind: 'goal'; id: string }
  | { kind: 'habit'; id: string }
  | { kind: 'task'; id: string };

export interface TaskView {
  id: string;
  title: string;
  state: TaskState;
  importance: Classification;
  urgency: Classification;
  deadline: string | null;
  overdue: boolean;
  archived: boolean;
  oneOff: boolean;
}

export interface ValueView {
  id: string;
  title: string;
  archived: boolean;
}

export interface GoalView {
  id: string;
  title: string;
  achieved: boolean;
  targetDate: string | null;
  archived: boolean;
}

export interface HabitView {
  id: string;
  title: string;
  cadence: Cadence;
  strength: HabitStrength;
  pinned: boolean;
  archived: boolean;
}

export interface LibraryView {
  values: ValueView[];
  goals: GoalView[];
  habits: HabitView[];
  tasks: TaskView[];
}

export interface Association {
  id: string;
  left: AssociationEnd;
  right: AssociationEnd;
  lifecycle: 'active' | 'archived';
  createdAt: string;
}

export type CheckInOutcome = 'done' | 'skipped' | 'notCompleted';

export type Recurrence =
  | { kind: 'daily' }
  | { kind: 'weekdays' }
  | { kind: 'weekly'; weekday: Weekday }
  | { kind: 'monthlyDay'; day: number };

export interface PlanTaskView {
  id: string;
  title: string;
  state: TaskState;
  importance: Classification;
  urgency: Classification;
  deadline: string | null;
  overdue: boolean;
  archived: boolean;
  position: number;
}

export interface PlanHabitView {
  id: string;
  title: string;
  cadence: Cadence;
  archived: boolean;
  unpinned: boolean;
  outcome: CheckInOutcome | null;
}

export interface DailyPlanView {
  date: string;
  week: string;
  tasks: PlanTaskView[];
  habits: PlanHabitView[];
}

export interface TaskPoolView {
  focus: TaskView[];
  rest: TaskView[];
}

export interface WeeklyFocus {
  id: string;
  week: string;
  tasks: string[];
  createdAt: string;
}

export interface RecurringTask {
  id: string;
  title: string;
  recurrence: Recurrence;
  lifecycle: 'active' | 'archived';
  startsOn: string;
  materializedThrough: string | null;
  createdAt: string;
}

export interface HabitSummary {
  title: string;
  done: number;
  skipped: number;
  notCompleted: number;
}

export interface WeeklySummary {
  week: string;
  completed: string[];
  stillOpen: number;
  overdue: string[];
  habits: HabitSummary[];
  goalsAchieved: string[];
}

export interface WeeklyReviewView {
  week: string;
  summary: WeeklySummary;
  reflection: string;
  previousReport: string | null;
  nextWeekFocus: TaskView[];
  reportPath: string;
}
