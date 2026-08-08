export type Surface = 'main' | 'weekly-review';

/** Pure so the routing rule is testable without a window. */
export function currentSurface(search: string): Surface {
  return new URLSearchParams(search).get('surface') === 'weekly-review'
    ? 'weekly-review'
    : 'main';
}
