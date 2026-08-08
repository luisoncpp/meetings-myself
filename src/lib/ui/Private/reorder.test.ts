import { describe, expect, it } from 'vitest';
import { move } from './reorder';

describe('move', () => {
  const order = ['a', 'b', 'c'];

  it('moves an item up and down', () => {
    expect(move(order, { id: 'b', direction: 'up' })).toEqual(['b', 'a', 'c']);
    expect(move(order, { id: 'b', direction: 'down' })).toEqual(['a', 'c', 'b']);
  });

  it('does nothing at the ends', () => {
    expect(move(order, { id: 'a', direction: 'up' })).toEqual(order);
    expect(move(order, { id: 'c', direction: 'down' })).toEqual(order);
  });

  it('ignores an unknown id', () => {
    expect(move(order, { id: 'z', direction: 'up' })).toEqual(order);
  });

  it('never loses or duplicates an entry', () => {
    const moved = move(order, { id: 'c', direction: 'up' });
    expect([...moved].sort()).toEqual([...order].sort());
  });
});
