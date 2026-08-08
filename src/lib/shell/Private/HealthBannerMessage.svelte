<script lang="ts">
  import type { StoreHealth } from '../../api';

  interface Props {
    health: StoreHealth;
  }

  let { health }: Props = $props();
</script>

{#if health.status === 'setupIncomplete'}
  <p>Finish setup before using your planning data.</p>
{:else if health.status === 'folderMissing'}
  <p>
    The sync folder could not be found at <strong>{health.path}</strong>. Check that Google Drive is
    running and the folder still exists.
  </p>
{:else if health.status === 'lockedByAnotherDevice'}
  <p>
    Another device (<strong>{health.deviceName}</strong>) has the planning data open since
    {health.since}. Close Self-Planning on that device, then try again.
  </p>
{:else if health.status === 'syncConflict'}
  <p>Google Drive has conflicting copies of your planning files. Resolve them in Drive first:</p>
  <ul>
    {#each health.artifacts as artifact (artifact)}
      <li>{artifact}</li>
    {/each}
  </ul>
  <p>After Drive finishes syncing, try again here.</p>
{:else if health.status === 'unreadable'}
  <p>The planning data could not be read: {health.detail}</p>
{/if}

<style>
  p,
  ul {
    margin: 0;
    font-size: var(--text-body);
    line-height: 1.5;
  }

  ul {
    padding-left: var(--space-4);
  }
</style>
