import en from './catalogs/en.json';
import es from './catalogs/es.json';
import type { LocaleStore } from './LocaleStore.svelte';

import type { UiLanguage } from '../../api';

export type { UiLanguage };

type Catalog = typeof en;
type Params = Record<string, string | number>;

const CATALOGS: Record<UiLanguage, Catalog> = { en, es };

let storeRef: LocaleStore | null = null;

export function bindLocaleStore(store: LocaleStore): void {
  storeRef = store;
}

function lookup(catalog: Catalog, key: string): string | undefined {
  const parts = key.split('.');
  let node: unknown = catalog;
  for (const part of parts) {
    if (node === null || typeof node !== 'object' || !(part in node)) return undefined;
    node = (node as Record<string, unknown>)[part];
  }
  return typeof node === 'string' ? node : undefined;
}

function interpolate(text: string, params?: Params): string {
  if (!params) return text;
  return Object.entries(params).reduce(
    (result, [name, value]) => result.replaceAll(`{${name}}`, String(value)),
    text,
  );
}

/** Resolves a catalog key in the active locale, falling back to English. */
export function t(key: string, params?: Params): string {
  const locale = storeRef?.locale ?? 'en';
  const text = lookup(CATALOGS[locale], key) ?? lookup(CATALOGS.en, key) ?? key;
  return interpolate(text, params);
}

export function bcp47(locale: UiLanguage): string {
  return locale === 'es' ? 'es' : 'en-US';
}
