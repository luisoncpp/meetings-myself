<script lang="ts">
  import type { StoreHealth } from '../../../api';
  import { Button, Card } from '../../../ui';
  import { SetupStore } from './SetupStore.svelte';

  interface Props {
    health: StoreHealth;
    onready: () => void;
  }

  let { health, onready }: Props = $props();

  let store = $state<SetupStore>();

  $effect(() => {
    if (store) return;
    store = new SetupStore(health, onready);
    void store.loadTimeZones();
  });
</script>

{#if store}
{@const setup = store}
<section class="setup" aria-labelledby="setup-heading">
  <header class="intro">
    <h1 id="setup-heading">Setup</h1>
    <p>Choose where your planning data lives and how the app counts days.</p>
  </header>

  <div class="steps">
    <Card>
      <h2>Synchronization folder</h2>
      <p>
        Pick a folder inside Google Drive. Self-Planning keeps your values, tasks, and plans there so
        they follow you between devices.
      </p>
      <p>Only one device may edit the data at a time — close the app elsewhere before continuing here.</p>

      {#if setup.chosenFolder}
        <p class="chosen">{setup.chosenFolder}</p>
      {/if}

      <Button
        variant="primary"
        disabled={setup.step !== 'folder'}
        onclick={/* open folder picker */ () => void setup.chooseFolder()}
      >
        Choose folder
      </Button>
    </Card>

    <Card>
      <h2>Home time zone</h2>
      <p>
        This choice governs day and week boundaries on every device. The app suggests nothing — pick
        the zone where your days actually begin.
      </p>

      <label class="field" for="home-zone">
        <span class="label">Home time zone</span>
        <input
          id="home-zone"
          list="time-zones"
          bind:value={setup.zoneText}
          disabled={setup.step !== 'zone'}
          autocomplete="off"
        />
        <datalist id="time-zones">
          {#each setup.timeZones as zone (zone)}
            <option value={zone}></option>
          {/each}
        </datalist>
      </label>

      <Button
        variant="primary"
        disabled={setup.step !== 'zone' || setup.zoneText.trim() === ''}
        onclick={/* save home zone */ () => void setup.finishSetup()}
      >
        Finish setup
      </Button>
    </Card>
  </div>

  {#if setup.lastError}
    <p class="error" role="alert">{setup.lastError}</p>
  {/if}
</section>
{/if}

<style>
  .setup {
    padding: var(--space-6);
    max-width: 42rem;
  }

  .intro {
    margin-bottom: var(--space-6);
  }

  h1 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-display);
    font-weight: 600;
    line-height: 1.15;
  }

  .intro p {
    margin: 0;
    color: var(--color-ink-muted);
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-title);
    font-weight: 600;
  }

  .steps p {
    margin: 0 0 var(--space-3);
    line-height: 1.5;
  }

  .chosen {
    color: var(--color-ink-muted);
    font-size: var(--text-caption);
    word-break: break-all;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin-bottom: var(--space-4);
  }

  .label {
    font-size: var(--text-caption);
    color: var(--color-ink-muted);
  }

  input {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-hairline);
    border-radius: var(--radius-control);
    background: var(--color-raised);
    color: var(--color-ink);
    font: inherit;
  }

  input:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }

  input:focus-visible {
    outline: 2px solid var(--color-gold);
    outline-offset: 2px;
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
