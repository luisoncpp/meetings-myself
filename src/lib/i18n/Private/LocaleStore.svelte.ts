import { setUiLanguage as persistUiLanguage, uiLanguage as fetchUiLanguage } from '../../api';
import { bindLocaleStore, type UiLanguage } from './t';

export class LocaleStore {
  locale = $state<UiLanguage>('en');
  ready = $state(false);

  constructor() {
    bindLocaleStore(this);
  }

  async load(): Promise<void> {
    const language = await fetchUiLanguage();
    this.setLocale(language, /*persist=*/ false);
    this.ready = true;
  }

  async setLocale(language: UiLanguage, persist = true): Promise<void> {
    this.locale = language;
    document.documentElement.lang = language;
    if (persist) await persistUiLanguage(language);
  }
}

export const localeStore = new LocaleStore();
