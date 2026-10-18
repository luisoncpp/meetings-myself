<script lang="ts">
  import { t } from '../../i18n';
  import { ContextMenu } from '../../ui';
  import type { TaskActions, TaskRef } from './task-actions';
  import { taskMenuItems } from './task-menu-items';
  import type { TaskMenuController } from './TaskMenuController.svelte';

  interface Props {
    controller: TaskMenuController;
    actions: TaskActions;
  }

  let { controller, actions }: Props = $props();

  let renaming = $state(false);
  let draft = $state('');

  function startRename(task: TaskRef): void {
    draft = task.title;
    renaming = true;
  }

  function close(): void {
    renaming = false;
    controller.close();
  }

  function submitRename(event: SubmitEvent, task: TaskRef): void {
    event.preventDefault();
    const title = draft.trim();
    close();
    if (title === '' || title === task.title) return;
    void actions.rename(task.id, title);
  }
</script>

{#snippet renameForm()}
  {#if controller.task}
    {@const task = controller.task}
    <form class="rename" onsubmit={(event) => submitRename(event, task)}>
      <input class="control-field" aria-label={t('taskMenu.newTitle')} bind:value={draft} autocomplete="off" />
    </form>
  {/if}
{/snippet}

{#if controller.task}
  {@const task = controller.task}
  <ContextMenu
    point={controller.point}
    label={t('taskMenu.label', { title: task.title })}
    items={taskMenuItems(task, actions, /* onrename= */ () => startRename(task))}
    onclose={close}
    children={renaming ? renameForm : undefined}
  />
{/if}

<style>
  .rename {
    padding: var(--space-1);
  }

  .rename input {
    width: 100%;
  }
</style>
