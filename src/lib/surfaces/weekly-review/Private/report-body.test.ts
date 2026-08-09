import { describe, expect, it } from 'vitest';
import { renderReportBody, stripSummaryMarkers } from './report-body';

const sampleBody = `<!-- self-planning:summary:start -->
## Week in review

**Completed:** none
**Overdue:** none
**Goals achieved:** none
**Still open:** 0
**Habits:** no check-ins recorded
<!-- self-planning:summary:end -->

## Reflection

A quiet week.
`;

describe('stripSummaryMarkers', () => {
  it('removes the app-owned region markers', () => {
    const stripped = stripSummaryMarkers(sampleBody);
    expect(stripped).not.toContain('self-planning:summary');
    expect(stripped).toContain('## Week in review');
    expect(stripped).toContain('## Reflection');
  });
});

describe('renderReportBody', () => {
  it('renders markdown headings and emphasis', () => {
    const html = renderReportBody(sampleBody);
    expect(html).toContain('<h2>Week in review</h2>');
    expect(html).toContain('<strong>Completed:</strong>');
    expect(html).toContain('<h2>Reflection</h2>');
    expect(html).toContain('A quiet week.');
  });

  it('does not expose marker comments in the output', () => {
    const html = renderReportBody(sampleBody);
    expect(html).not.toContain('self-planning:summary');
  });
});
