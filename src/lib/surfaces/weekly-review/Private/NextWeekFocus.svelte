<script lang="ts">
  import type { TaskView } from '../../../domain';

  import { t } from '../../../i18n';

  interface Props {
    tasks: TaskView[];
  }

  let { tasks }: Props = $props();
</script>

<section class="focus" aria-labelledby="next-focus-heading">
  <h2 id="next-focus-heading">{t('weeklyReview.nextWeekFocus')}</h2>
  {#if tasks.length === 0}
    <p class="empty">{t('weeklyReview.nothingChosen')}</p>
  {:else}
    <ul>
      {#each tasks as task (task.id)}
        <li>{task.title}</li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .focus {
    margin-bottom: var(--space-4);
  }

  h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .empty {
    margin: 0;
    color: var(--color-ink-muted);
  }

  ul {
    margin: 0;
    padding-left: var(--space-5);
  }
</style>
