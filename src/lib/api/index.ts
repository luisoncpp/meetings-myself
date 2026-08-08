import { call } from './Private/bridge';

export function appVersion(): Promise<string> {
  return call<string>('app_version');
}

export type StoreHealth =
  | { status: 'ready' }
  | { status: 'setupIncomplete'; reason: { kind: 'NoSyncFolder' | 'NoHomeZone' } }
  | { status: 'folderMissing'; path: string }
  | { status: 'lockedByAnotherDevice'; deviceName: string; since: string }
  | { status: 'syncConflict'; artifacts: string[] }
  | { status: 'unreadable'; detail: string };

export function storeHealth(): Promise<StoreHealth> {
  return call<StoreHealth>('store_health');
}

export function chooseSyncFolder(folder: string): Promise<StoreHealth> {
  return call<StoreHealth>('choose_sync_folder', { folder });
}

export function setHomeZone(zone: string): Promise<StoreHealth> {
  return call<StoreHealth>('set_home_zone', { zone });
}
