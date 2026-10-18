<script lang="ts">
  import { AppShell } from './lib/shell';
  import { localeStore } from './lib/i18n';

  $effect(() => {
    void localeStore.load();
  });
</script>

{#if localeStore.ready}
  <AppShell />
{:else}
  <p class="boot-loading" lang="en">Loading…</p>
{/if}

<style>
  .boot-loading {
    margin: 0;
    padding: var(--space-6);
    color: var(--color-ink-muted);
    font-family: var(--font-sans);
  }
</style>
