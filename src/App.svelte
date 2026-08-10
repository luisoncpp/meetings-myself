<script lang="ts">
  import { AppShell, currentSurface } from './lib/shell';
  import { localeStore } from './lib/i18n';

  const surface = currentSurface(window.location.search);

  $effect(() => {
    void localeStore.load();
  });
</script>

{#if localeStore.ready}
  <AppShell {surface} />
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
