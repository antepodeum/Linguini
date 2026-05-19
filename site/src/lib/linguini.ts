export type Locale = 'en' | 'ru';

const dictionaries = {
  en: {
    nav: ['Why', 'Language', 'Codegen', 'Web'],
    heroEyebrow: 'Typed localization for product teams',
    heroTitle: 'Linguini',
    heroCopy:
      'A compiled localization language that keeps schemas, locale grammar, CLDR formatting, and generated TypeScript in one typed pipeline.',
    primaryCta: 'Read the docs',
    secondaryCta: 'View GitHub',
    schemaChip: 'schema owns contract',
    localeChip: 'locale owns language',
    generatedChip: 'app imports native code',
    proofTitle: 'One source of truth from schema to runtime',
    features: [
      'Schema-defined messages, enums, primitive TypeKind values, and formatter aliases.',
      'Locale forms and functions for plural, gender, case, and custom agreement.',
      'CLDR-backed plural rules, number formatting, currency formatting, and date formatting.',
      'TypeScript, Svelte, SvelteKit, Vite, LSP, formatter, analyzer, and VS Code support.'
    ],
    sampleTitle: 'Schema formatting without locale boilerplate',
    outputLabel: 'Generated call',
    localeToggle: 'Locale'
  },
  ru: {
    nav: ['Зачем', 'Язык', 'Кодген', 'Web'],
    heroEyebrow: 'Типизированная локализация для продуктовых команд',
    heroTitle: 'Linguini',
    heroCopy:
      'Компилируемый язык локализации, где схемы, грамматика локалей, CLDR-форматирование и TypeScript проходят один типизированный пайплайн.',
    primaryCta: 'Документация',
    secondaryCta: 'GitHub',
    schemaChip: 'схема задает контракт',
    localeChip: 'локаль задает язык',
    generatedChip: 'приложение импортирует код',
    proofTitle: 'Один источник истины от схемы до рантайма',
    features: [
      'Сообщения, enum, TypeKind и formatter alias задаются в схеме.',
      'Формы и функции локали покрывают plural, gender, case и свои согласования.',
      'CLDR дает plural rules, number, currency и date formatting.',
      'Есть TypeScript, Svelte, SvelteKit, Vite, LSP, formatter, analyzer и VS Code.'
    ],
    sampleTitle: 'Форматирование в схеме, без шума в локали',
    outputLabel: 'Сгенерированный вызов',
    localeToggle: 'Локаль'
  }
} as const;

export function createLinguini(locale: Locale) {
  return dictionaries[locale];
}
