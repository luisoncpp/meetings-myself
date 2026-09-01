import {
  availableTimeZones,
  chooseSyncFolder,
  pickSyncFolder,
  setHomeZone,
  type StoreHealth,
} from '../../../api';

import { localizeError } from '../../../i18n';

type SetupStep = 'folder' | 'zone';

export class SetupStore {
  chosenFolder = $state<string | null>(null);
  zoneText = $state('');
  lastError = $state<string | null>(null);
  timeZones = $state<string[]>([]);
  step = $state<SetupStep>('folder');

  constructor(
    health: StoreHealth,
    private readonly onready: () => void,
  ) {
    if (health.status !== 'setupIncomplete') return;
    if (health.reason.kind === 'NoHomeZone') {
      this.step = 'zone';
    }
  }

  // Used from Setup.svelte on mount.
  // fallow-ignore-next-line unused-class-member
  async loadTimeZones(): Promise<void> {
    try {
      this.timeZones = await availableTimeZones();
    } catch (error) {
      this.lastError = localizeError(error instanceof Error ? error.message : String(error));
    }
  }

  // Used from Setup.svelte template.
  // fallow-ignore-next-line unused-class-member
  async chooseFolder(): Promise<void> {
    this.lastError = null;
    try {
      const folder = await pickSyncFolder();
      if (folder === null) return;

      this.chosenFolder = folder;
      const health = await chooseSyncFolder(folder);
      if (health.status === 'setupIncomplete' && health.reason.kind === 'NoHomeZone') {
        this.step = 'zone';
        return;
      }
      this.onready();
    } catch (error) {
      this.lastError = localizeError(error instanceof Error ? error.message : String(error));
    }
  }

  // Used from Setup.svelte template.
  // fallow-ignore-next-line unused-class-member
  async finishSetup(): Promise<void> {
    this.lastError = null;
    try {
      const health = await setHomeZone(this.zoneText);
      if (health.status === 'ready') {
        this.onready();
      }
    } catch (error) {
      this.lastError = localizeError(error instanceof Error ? error.message : String(error));
    }
  }
}
