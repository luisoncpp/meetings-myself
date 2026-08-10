<script lang="ts">
  import type { StoreHealth } from '../../../api';
  import { t } from '../../../i18n';
  import { Button, Card, Field, Input, LanguageSelect, SurfaceLayout } from '../../../ui';
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
  <SurfaceLayout aria-labelledby="setup-heading">
    <section class="setup">
      <header class="intro">
        <h1 id="setup-heading">{t('setup.title')}</h1>
        <p>{t('setup.intro')}</p>
        <LanguageSelect />
      </header>

      <div class="steps">
        <Card>
          <h2>{t('setup.syncFolderTitle')}</h2>
          <p>{t('setup.syncFolderBody1')}</p>
          <p>{t('setup.syncFolderBody2')}</p>

          {#if setup.chosenFolder}
            <p class="chosen">{setup.chosenFolder}</p>
          {/if}

          <Button
            variant="primary"
            disabled={setup.step !== 'folder'}
            onclick={/* open folder picker */ () => void setup.chooseFolder()}
          >
            {t('setup.chooseFolder')}
          </Button>
        </Card>

        <Card>
          <h2>{t('setup.homeZoneTitle')}</h2>
          <p>{t('setup.homeZoneBody')}</p>

          <div class="zone-field">
            <Field label={t('setup.homeZoneLabel')} forId="home-zone">
              <Input
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
            </Field>
          </div>

          <Button
            variant="primary"
            disabled={setup.step !== 'zone' || setup.zoneText.trim() === ''}
            onclick={/* save home zone */ () => void setup.finishSetup()}
          >
            {t('setup.finishSetup')}
          </Button>
        </Card>
      </div>

      {#if setup.lastError}
        <p class="error" role="alert">{setup.lastError}</p>
      {/if}
    </section>
  </SurfaceLayout>
{/if}

<style>
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

  .intro :global(.language) {
    margin-top: var(--space-3);
  }

  .steps {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-headline);
    font-weight: 600;
  }

  .steps p {
    margin: 0 0 var(--space-3);
    line-height: 1.5;
  }

  .chosen {
    color: var(--color-ink-muted);
    font-size: var(--text-label);
    word-break: break-all;
  }

  .zone-field {
    margin-bottom: var(--space-4);
  }

  .error {
    margin: var(--space-4) 0 0;
    color: var(--color-overdue);
  }
</style>
