import { call } from './Private/bridge';

export function appVersion(): Promise<string> {
  return call<string>('app_version');
}
