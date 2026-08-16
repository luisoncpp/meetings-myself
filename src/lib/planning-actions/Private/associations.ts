import type { AssociationEnd, LibraryView } from '../../domain';

export type EntityKind = AssociationEnd['kind'];

/** Order required for tabs: goal, habit, value, task */
const TARGET_ORDER: readonly EntityKind[] = ['goal', 'habit', 'value', 'task'];

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

export function orderedLinkTargetsFor(kind: EntityKind): EntityKind[] {
  const allowed = LINK_TARGETS[kind];
  return TARGET_ORDER.filter((target) => allowed.includes(target));
}

export interface LinkedEntity {
  associationId: string;
  end: AssociationEnd;
  kind: EntityKind;
  title: string;
}

function isSameEnd(a: AssociationEnd, b: AssociationEnd): boolean {
  return a.kind === b.kind && a.id === b.id;
}

export function linkedEntitiesFor(
  view: LibraryView,
  end: AssociationEnd,
): LinkedEntity[] {
  if (!view.associations) return [];
  const result: LinkedEntity[] = [];
  for (const link of view.associations) {
    if (link.lifecycle !== 'active') continue;
    if (isSameEnd(link.left, end)) {
      result.push({
        associationId: link.id,
        end: link.right,
        kind: link.right.kind,
        title: entityTitle(view, link.right),
      });
      continue;
    }
    if (isSameEnd(link.right, end)) {
      result.push({
        associationId: link.id,
        end: link.left,
        kind: link.left.kind,
        title: entityTitle(view, link.left),
      });
    }
  }
  return result;
}

export function unlinkedCandidatesFor(
  view: LibraryView,
  fromEnd: AssociationEnd,
  targetKind: EntityKind,
): { id: string; title: string }[] {
  const linked = linkedEntitiesFor(view, fromEnd);
  const linkedIds = new Set(
    linked.filter((item) => item.kind === targetKind).map((item) => item.end.id),
  );
  return entitiesOf(view, targetKind).filter((item) => !linkedIds.has(item.id));
}

function entitiesOf(
  view: LibraryView,
  kind: EntityKind,
): { id: string; title: string }[] {
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

function entityTitle(view: LibraryView, end: AssociationEnd): string {
  const item = entitiesOf(view, end.kind).find((e) => e.id === end.id);
  return item?.title ?? end.id;
}
