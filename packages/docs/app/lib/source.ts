import { loader } from 'fumadocs-core/source';
import { createI18nProvider } from 'fumadocs-ui/i18n';
import { docs } from 'fumadocs-mdx:collections/server';

export const { utils: i18n } = createI18nProvider({
  languages: [
    { name: 'English', locale: 'en' },
    { name: '简体中文', locale: 'zh' },
  ],
  defaultLanguage: 'en',
});

export const source = loader({
  source: docs.toFumadocsSource(),
  baseUrl: '/docs',
  i18n,
});
