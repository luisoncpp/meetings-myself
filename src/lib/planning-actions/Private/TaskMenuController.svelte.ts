import type { MenuPoint } from '../../ui';
import type { TaskRef } from './task-actions';

/**
 * Which task's menu is open and where. One per surface; rows call `open` from
 * their context-menu handler and `TaskMenu` renders whatever is open.
 */
export class TaskMenuController {
  #task = $state<TaskRef | null>(null);
  #point = $state<MenuPoint>({ x: 0, y: 0 });

  get task(): TaskRef | null {
    return this.#task;
  }

  get point(): MenuPoint {
    return this.#point;
  }

  open(task: TaskRef, point: MenuPoint): void {
    this.#task = task;
    this.#point = point;
  }

  close(): void {
    this.#task = null;
  }
}
