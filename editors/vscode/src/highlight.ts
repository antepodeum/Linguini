import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import { linguiniFences } from './markdown';

interface HighlightResult {
  version: number;
  legend: string[];
  tokens: Array<{ line: number; start: number; length: number; type: string }>;
}

export async function registerMarkdownHighlighting(
  client: LanguageClient,
  enabled: () => boolean
): Promise<vscode.Disposable> {
  const highlight = await client.sendRequest<HighlightResult>('linguini/highlight', {
    language: 'lgl', source: ''
  });
  if (highlight.version !== 1) throw new Error('Unsupported native highlight version');
  const legend = new vscode.SemanticTokensLegend(highlight.legend);
  return vscode.languages.registerDocumentSemanticTokensProvider(
    [{ language: 'markdown', scheme: 'file' }, { language: 'markdown', scheme: 'untitled' }],
    {
      async provideDocumentSemanticTokens(document, cancellation) {
        if (!enabled()) return null;
        const builder = new vscode.SemanticTokensBuilder(legend);
        for (const fence of linguiniFences(document.getText())) {
          if (cancellation.isCancellationRequested) return null;
          const result = await client.sendRequest<HighlightResult>('linguini/highlight', {
            language: fence.language, source: fence.source
          }, cancellation);
          for (const item of result.tokens) {
            builder.push(fence.startLine + item.line, fence.indentation[item.line] + item.start,
              item.length, legend.tokenTypes.indexOf(item.type), 0);
          }
        }
        return builder.build();
      }
    }, legend
  );
}
