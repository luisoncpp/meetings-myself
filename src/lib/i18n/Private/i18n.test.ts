import { beforeEach, describe, expect, it } from 'vitest';
import { LocaleStore } from './LocaleStore.svelte';
import { bindLocaleStore, t } from './t';
import { localizeError } from './localize-error';

describe('t', () => {
  beforeEach(() => {
    bindLocaleStore(new LocaleStore());
  });

  it('resolves English keys', () => {
    expect(t('nav.dailyPlan')).toBe('Daily Plan');
  });

  it('falls back to English for missing Spanish keys', () => {
    const store = new LocaleStore();
    bindLocaleStore(store);
    store.locale = 'es';
    expect(t('nav.dailyPlan')).toBe('Plan diario');
  });

  it('interpolates parameters', () => {
    expect(t('health.folderMissing', { path: '/tmp' })).toContain('/tmp');
  });
});

describe('localizeError', () => {
  beforeEach(() => {
    bindLocaleStore(new LocaleStore());
  });

  it('localizes structured error codes', () => {
    const message = localizeError(
      JSON.stringify({ code: 'notFound', params: { table: 'task', id: 'abc' } }),
    );
    expect(message).toContain('task');
    expect(message).toContain('abc');
  });

  it('wraps unknown strings', () => {
    expect(localizeError('plain failure')).toContain('plain failure');
  });
});
