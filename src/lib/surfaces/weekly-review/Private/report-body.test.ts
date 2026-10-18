import { describe, expect, it } from 'vitest';
import { presentPreviousReport, renderReportBody, stripSummaryMarkers } from './report-body';

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
  it('renders markdown headings and drops empty facts', () => {
    const html = renderReportBody(sampleBody);
    expect(html).toContain('<h2>Week in review</h2>');
    expect(html).not.toContain('Completed');
    expect(html).not.toContain('none');
    expect(html).toContain('<strong>Still open:</strong>');
    expect(html).toContain('<h2>Reflection</h2>');
    expect(html).toContain('A quiet week.');
  });

  it('does not expose marker comments in the output', () => {
    const html = renderReportBody(sampleBody);
    expect(html).not.toContain('self-planning:summary');
  });
});

const spanishReport = `## Semana en revisión

**Completadas:** ninguna
**Aún abiertas:** 7

**Hábitos:**

| Hábito | Hecho | Omitido | No completado |
| --- | --- | --- | --- |
| Lagartijas | 1 | 0 | 1 |

## Reflexión

Fui consistente.
`;

describe('presentPreviousReport', () => {
  it('lifts the title, the open count, and habit rows out of the summary', () => {
    const parts = presentPreviousReport(spanishReport);
    expect(parts.title).toBe('Semana en revisión');
    expect(parts.stillOpen).toBe(7);
    expect(parts.habits).toEqual([{ title: 'Lagartijas', done: 1, skipped: 0, notCompleted: 1 }]);
    expect(parts.recordHtml).not.toContain('ninguna');
    expect(parts.recordHtml).not.toContain('<table>');
    expect(parts.proseHtml).toContain('Fui consistente.');
    expect(parts.proseHtml).toContain('<h2>Reflexión</h2>');
  });

  it('leaves an unrecognized table in the record', () => {
    const parts = presentPreviousReport(`## Week in review

| Thing | Count |
| --- | --- |
| A | 1 |
`);

    expect(parts.habits).toBeNull();
    expect(parts.recordHtml).toContain('<table>');
    expect(parts.recordHtml).toContain('Thing');
  });
});
