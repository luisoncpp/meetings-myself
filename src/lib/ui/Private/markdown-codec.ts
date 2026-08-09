import { marked } from 'marked';
import TurndownService from 'turndown';

marked.setOptions({
  gfm: true,
  breaks: true,
});

const turndown = new TurndownService({
  headingStyle: 'atx',
  bulletListMarker: '-',
});

const EMPTY_HTML = new Set(['', '<p></p>', '<p><br></p>']);

function normalizeHtml(html: string): string {
  return html.trim().replace(/\s+/g, '');
}

export function isEmptyEditorHtml(html: string): boolean {
  return EMPTY_HTML.has(normalizeHtml(html));
}

export function markdownToHtml(markdown: string): string {
  if (markdown.trim() === '') return '';
  return marked.parse(markdown, { async: false }) as string;
}

export function htmlToMarkdown(html: string): string {
  if (isEmptyEditorHtml(html)) return '';
  return turndown.turndown(html).trimEnd();
}
