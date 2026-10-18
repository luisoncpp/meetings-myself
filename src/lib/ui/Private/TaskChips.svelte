<script lang="ts">
  import type { Classification } from '../../domain';
  import { t } from '../../i18n';

  interface Props {
    importance: Classification;
    urgency: Classification;
    overdue: boolean;
  }

  let { importance, urgency, overdue }: Props = $props();
</script>

<!-- Only the signals worth a glance; low/unclassified stay in the detail panel. -->
{#if overdue}
  <span class="overdue">{t('domain.stateFlag.overdue')}</span>
{/if}
{#if importance === 'high'}
  <span>{t('taskLine.important')}</span>
{/if}
{#if urgency === 'high'}
  <span>{t('taskLine.urgent')}</span>
{/if}

<style>
  span {
    font-size: var(--text-label);
    color: var(--color-ink-muted);
  }

  .overdue {
    color: var(--color-overdue);
  }
</style>
