import { t } from '../../i18n';
import type { MenuItem } from '../../ui';
import type { TaskActions, TaskRef } from './task-actions';

/** Menu entries for one task, in reading order. `onrename` switches the menu to its rename form. */
export function taskMenuItems(
  task: TaskRef,
  actions: TaskActions,
  onrename: () => void,
): MenuItem[] {
  const items: MenuItem[] = [doneItem(task, actions)];
  if (!task.archived) items.push(...focusItems(task, actions));
  items.push(...todayItems(task, actions));
  items.push({ id: 'rename', label: t('taskMenu.rename'), onselect: onrename, keepOpen: true });
  items.push({
    id: 'archive',
    label: task.archived ? t('common.restore') : t('common.archive'),
    onselect: () => void actions.setArchived(task.id, !task.archived),
  });
  return items;
}

function doneItem(task: TaskRef, actions: TaskActions): MenuItem {
  if (task.done) {
    return { id: 'reopen', label: t('taskMenu.reopen'), onselect: () => void actions.reopen(task.id) };
  }
  return { id: 'finish', label: t('taskMenu.doneAndArchive'), onselect: () => void actions.finish(task) };
}

function focusItems(task: TaskRef, actions: TaskActions): MenuItem[] {
  return actions.focusTargets.map((target) => {
    const inFocus = target.taskIds.has(task.id);
    const key = `taskMenu.${inFocus ? 'removeFrom' : 'addTo'}${target.which === 'thisWeek' ? 'This' : 'Next'}WeekFocus`;
    return {
      id: `focus-${target.week}`,
      label: t(key),
      onselect: () => void actions.setFocus(target, task.id),
    };
  });
}

function todayItems(task: TaskRef, actions: TaskActions): MenuItem[] {
  if (!actions.canPlanToday) return [];
  if (task.inToday) {
    return [
      { id: 'remove-today', label: t('taskMenu.removeFromToday'), onselect: () => void actions.removeFromToday(task.id) },
    ];
  }
  if (task.done || task.archived) return [];
  return [{ id: 'add-today', label: t('taskMenu.addToToday'), onselect: () => void actions.addToToday(task.id) }];
}
