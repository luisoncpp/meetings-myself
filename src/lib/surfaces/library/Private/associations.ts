import type { AssociationEnd, LibraryView } from '../../../domain';

export type EntityKind = AssociationEnd['kind'];

/** Pairs CONTEXT.md permits — unsupported kinds never appear in the picker. */
const LINK_TARGETS: Record<EntityKind, EntityKind[]> = {
  value: ['goal'],
  goal: ['value', 'habit', 'task'],
  habit: ['goal', 'task'],
  task: ['goal', 'habit'],
};

export function linkTargetsFor(kind: EntityKind): EntityKind[] {
  return LINK_TARGETS[kind];
}

export function candidatesFor(
  view: LibraryView,
  targetKind: EntityKind,
): AssociationEnd[] {
  const items = entitiesOf(view, targetKind);
  return items.map((item) => ({ kind: targetKind, id: item.id }));
}

function entitiesOf(
  view: LibraryView,
  kind: EntityKind,
): { id: string }[] {
  switch (kind) {
    case 'value':
      return view.values;
    case 'goal':
      return view.goals;
    case 'habit':
      return view.habits;
    case 'task':
      return view.tasks;
  }
}

export function entityTitle(view: LibraryView, end: AssociationEnd): string {
  switch (end.kind) {
    case 'value':
      return view.values.find((v) => v.id === end.id)?.title ?? end.id;
    case 'goal':
      return view.goals.find((g) => g.id === end.id)?.title ?? end.id;
    case 'habit':
      return view.habits.find((h) => h.id === end.id)?.title ?? end.id;
    case 'task':
      return view.tasks.find((t) => t.id === end.id)?.title ?? end.id;
  }
}
