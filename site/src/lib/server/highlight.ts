import { createHighlighterCore, type LanguageRegistration } from 'shiki/core';
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript';
import svelteGrammar from 'shiki/langs/svelte.mjs';
import darkPlusTheme from 'shiki/themes/dark-plus.mjs';
import linguiniLocaleGrammar from '../../../../editors/vscode/syntaxes/linguini-locale.tmLanguage.json';
import linguiniSchemaGrammar from '../../../../editors/vscode/syntaxes/linguini-schema.tmLanguage.json';

const highlighter = createHighlighterCore({
  engine: createJavaScriptRegexEngine(),
  themes: [darkPlusTheme],
  langs: [
    svelteGrammar,
    {
      ...(linguiniLocaleGrammar as unknown as LanguageRegistration),
      name: 'linguini-locale',
      aliases: ['lgl']
    },
    {
      ...(linguiniSchemaGrammar as unknown as LanguageRegistration),
      name: 'linguini-schema',
      aliases: ['lgs']
    }
  ]
});

export async function highlightCode(code: string, lang: string) {
  return (await highlighter).codeToHtml(code, {
    lang,
    theme: 'dark-plus'
  });
}
