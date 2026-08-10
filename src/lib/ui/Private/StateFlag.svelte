<script lang="ts">
  import { t } from '../../i18n';

  interface Props {
    kind: 'archived' | 'overdue' | 'unpinned' | 'completed';
  }

  let { kind }: Props = $props();
</script>

<span class="flag {kind}">{t(`domain.stateFlag.${kind}`)}</span>

<style>
  .flag {
    display: inline-block;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-pill);
    font-size: var(--text-label);
  }

  .archived,
  .unpinned {
    color: var(--color-ink-muted);
  }

  .overdue {
    color: var(--color-overdue);
  }

  .completed {
    color: var(--color-done);
  }
</style>
