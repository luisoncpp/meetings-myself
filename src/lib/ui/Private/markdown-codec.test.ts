import { describe, expect, it } from 'vitest';
import { htmlToMarkdown, isEmptyEditorHtml, markdownToHtml } from './markdown-codec';

describe('markdown-codec', () => {
  it('treats empty Quill shells as empty', () => {
    expect(isEmptyEditorHtml('<p><br></p>')).toBe(true);
    expect(isEmptyEditorHtml('<p></p>')).toBe(true);
    expect(htmlToMarkdown('<p><br></p>')).toBe('');
  });

  it('round-trips headings and emphasis', () => {
    const source = '## Reflection\n\nA **quiet** week with *notes*.';
    const html = markdownToHtml(source);
    const roundTrip = htmlToMarkdown(html);
    expect(roundTrip).toContain('## Reflection');
    expect(roundTrip).toContain('**quiet**');
    expect(roundTrip).toContain('_notes_');
  });

  it('round-trips lists', () => {
    const source = '- one\n- two';
    const roundTrip = htmlToMarkdown(markdownToHtml(source));
    expect(roundTrip).toMatch(/- {1,3}one/);
    expect(roundTrip).toMatch(/- {1,3}two/);
  });

  it('round-trips links and blockquotes', () => {
    const source = '> quoted\n\n[docs](https://example.com)';
    const roundTrip = htmlToMarkdown(markdownToHtml(source));
    expect(roundTrip).toContain('> quoted');
    expect(roundTrip).toContain('[docs](https://example.com)');
  });

  it('returns empty html for blank markdown', () => {
    expect(markdownToHtml('   ')).toBe('');
  });
});
