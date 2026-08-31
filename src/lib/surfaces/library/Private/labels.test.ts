import { beforeEach, describe, expect, it } from 'vitest';
import { LocaleStore } from '../../../i18n/Private/LocaleStore.svelte';
import { bindLocaleStore } from '../../../i18n/Private/t';
import {
  classificationLabel,
  strengthLabel,
  weekdayLabel,
} from './labels';

describe('library labels', () => {
  beforeEach(() => {
    bindLocaleStore(new LocaleStore());
  });

  it('localizes habit strength in Spanish', () => {
    const store = new LocaleStore();
    bindLocaleStore(store);
    store.locale = 'es';

    expect(strengthLabel('reminderDependent')).toBe('Dependiente del recordatorio');
    expect(strengthLabel('strengthening')).toBe('Fortaleciéndose');
  });

  it('localizes weekdays in Spanish', () => {
    const store = new LocaleStore();
    bindLocaleStore(store);
    store.locale = 'es';

    expect(weekdayLabel('mon')).toBe('Lunes');
    expect(weekdayLabel('fri')).toBe('Viernes');
  });

  it('localizes classification in Spanish', () => {
    const store = new LocaleStore();
    bindLocaleStore(store);
    store.locale = 'es';

    expect(classificationLabel('high')).toBe('Alta');
  });
});
