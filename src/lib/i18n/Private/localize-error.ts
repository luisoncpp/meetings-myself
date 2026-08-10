import { t } from './t';

type ErrorPayload = {
  code: string;
  params?: Record<string, string | number>;
};

function parsePayload(raw: string): ErrorPayload | null {
  try {
    const parsed: unknown = JSON.parse(raw);
    if (parsed === null || typeof parsed !== 'object' || !('code' in parsed)) return null;
    const code = (parsed as ErrorPayload).code;
    if (typeof code !== 'string') return null;
    return parsed as ErrorPayload;
  } catch {
    return null;
  }
}

const ERROR_KEY: Record<string, string> = {
  notReady: 'errors.notReady',
  noDatabase: 'errors.noDatabase',
  notFound: 'errors.notFound',
  notSelectable: 'errors.notSelectable',
  invalidOrder: 'errors.invalidOrder',
  invalidZone: 'errors.invalidZone',
  blankTitle: 'errors.blankTitle',
  unsupportedAssociation: 'errors.unsupportedAssociation',
  emptyCadence: 'errors.emptyCadence',
  invalidMonthDay: 'errors.invalidMonthDay',
  storeIo: 'errors.storeIo',
  storeCorrupt: 'errors.storeCorrupt',
  noConfigDirectory: 'errors.noConfigDirectory',
  storeDatabase: 'errors.storeDatabase',
  storeNotReady: 'errors.storeNotReady',
  reportIo: 'errors.reportIo',
  reportMissingFrontMatter: 'errors.reportMissingFrontMatter',
  reportMalformedFrontMatter: 'errors.reportMalformedFrontMatter',
  reportUnsupportedSchema: 'errors.reportUnsupportedSchema',
  invalidWeekLabel: 'errors.invalidWeekLabel',
};

/** Maps IPC error payloads (or legacy plain strings) to localized text. */
export function localizeError(raw: string): string {
  const payload = parsePayload(raw);
  if (!payload) return t('errors.unknown', { detail: raw });
  const key = ERROR_KEY[payload.code];
  if (!key) return t('errors.unknown', { detail: raw });
  const params = { ...payload.params };
  if (params.reason === 'the task is archived') {
    params.reason = t('errors.taskArchived');
  }
  return t(key, params);
}
