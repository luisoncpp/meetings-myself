export interface MoveRequest {
  id: string;
  direction: 'up' | 'down';
}

/** Pure and total: an impossible move returns the input unchanged. */
export function move(order: readonly string[], request: MoveRequest): string[] {
  const from = order.indexOf(request.id);
  if (from === -1) return [...order];

  const to = request.direction === 'up' ? from - 1 : from + 1;
  if (to < 0 || to >= order.length) return [...order];

  const next = [...order];
  [next[from], next[to]] = [next[to]!, next[from]!];
  return next;
}

export function reorderByDrop(
  order: readonly string[],
  sourceId: string,
  targetId: string,
): string[] {
  if (sourceId === targetId) return [...order];

  const from = order.indexOf(sourceId);
  const to = order.indexOf(targetId);
  if (from === -1 || to === -1) return [...order];

  const next = [...order];
  next.splice(from, 1);
  next.splice(to, 0, sourceId);
  return next;
}

export function formatPositionAnnouncement(id: string, order: readonly string[]): string {
  const position = order.indexOf(id) + 1;
  return `${id} position ${position} of ${order.length}`;
}

export function ordersMatch(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((id, index) => id === right[index]);
}
