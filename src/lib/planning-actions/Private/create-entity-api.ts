import * as api from '../../api';
import type { CreatePayload } from './CreateEntity.svelte';

export function createEntityFromPayload(payload: CreatePayload): Promise<unknown> {
  if (payload.kind === 'value') {
    return api.createValue(payload.title);
  }
  if (payload.kind === 'goal') {
    return api.createGoal(payload.title, payload.targetDate);
  }
  if (payload.kind === 'habit') {
    return api.createHabit(payload.title, payload.cadence);
  }
  return api.createTask(payload.title, payload.oneOff);
}
