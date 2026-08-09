import { marked } from 'marked';

const SUMMARY_START = '<!-- self-planning:summary:start -->';
const SUMMARY_END = '<!-- self-planning:summary:end -->';

marked.setOptions({
  gfm: true,
  breaks: true,
});

export function stripSummaryMarkers(body: string): string {
  return body
    .replaceAll(`${SUMMARY_START}\n`, '')
    .replaceAll(`\n${SUMMARY_END}`, '')
    .replaceAll(SUMMARY_START, '')
    .replaceAll(SUMMARY_END, '');
}

export function renderReportBody(body: string): string {
  return marked.parse(stripSummaryMarkers(body), { async: false }) as string;
}
