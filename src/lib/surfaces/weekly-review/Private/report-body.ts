import { marked } from 'marked';
import type { HabitSummary } from '../../../domain';

const SUMMARY_START = '<!-- self-planning:summary:start -->';
const SUMMARY_END = '<!-- self-planning:summary:end -->';
const NONE_LINE = /^\*\*.+:\*\*\s*(none|ninguna)\s*$/i;
const STILL_OPEN_LINE = /^\*\*(?:Still open|Aún abiertas):\*\*\s*(\d+)\s*$/;
const HABITS_LABEL = /^\*\*(?:Habits|Hábitos):\*\*\s*$/;
const TITLE_LINE = /^## (?!Reflection$|Reflexión$)(.+)$/;
const HABIT_HEADER =
  /^\|\s*(?:Habit|Hábito)\s*\|\s*(?:Done|Hecho)\s*\|\s*(?:Skipped|Omitido)\s*\|\s*(?:Not completed|No completado)\s*\|\s*$/;
const TABLE_RULE = /^\|\s*-+/;
const REFLECTION_AT = /\n## (?:Reflection|Reflexión)\s*(?:\n|$)/;

marked.setOptions({
  gfm: true,
  breaks: true,
});

export interface PreviousReportParts {
  title: string | null;
  stillOpen: number | null;
  habits: HabitSummary[] | null;
  recordHtml: string;
  proseHtml: string;
}

export function stripSummaryMarkers(body: string): string {
  return body
    .replaceAll(`${SUMMARY_START}\n`, '')
    .replaceAll(`\n${SUMMARY_END}`, '')
    .replaceAll(SUMMARY_START, '')
    .replaceAll(SUMMARY_END, '');
}

export function renderReportBody(body: string): string {
  return renderMarkdown(stripEmptyFacts(stripSummaryMarkers(body)));
}

export function presentPreviousReport(body: string): PreviousReportParts {
  const stripped = stripSummaryMarkers(body).replace(/^\n+/, '');
  const splitAt = stripped.search(REFLECTION_AT);
  const summary = splitAt === -1 ? stripped : stripped.slice(0, splitAt);
  const prose = splitAt === -1 ? '' : stripped.slice(splitAt + 1);
  const parsed = parseSummary(summary);
  return {
    title: parsed.title,
    stillOpen: parsed.stillOpen,
    habits: parsed.habits,
    recordHtml: renderIfPresent(parsed.record),
    proseHtml: renderIfPresent(prose),
  };
}

function renderIfPresent(markdown: string): string {
  const trimmed = markdown.trim();
  if (!trimmed) return '';
  return renderMarkdown(trimmed);
}

function renderMarkdown(markdown: string): string {
  return marked.parse(markdown, { async: false }) as string;
}

function stripEmptyFacts(markdown: string): string {
  return markdown
    .split('\n')
    .filter((line) => !NONE_LINE.test(line.trim()))
    .join('\n');
}

function parseSummary(summary: string): Omit<PreviousReportParts, 'recordHtml' | 'proseHtml'> & {
  record: string;
} {
  const lines = stripEmptyFacts(summary).split('\n');
  const title = readTitle(lines);
  const stillOpen = readStillOpen(lines);
  const habits = takeHabitTable(lines);
  return { title, stillOpen, habits, record: lines.join('\n') };
}

function readTitle(lines: string[]): string | null {
  const index = lines.findIndex((line) => line.trim().length > 0);
  if (index < 0) return null;
  const match = TITLE_LINE.exec(lines[index].trim());
  if (!match) return null;
  lines.splice(index, 1);
  return match[1].trim();
}

function readStillOpen(lines: string[]): number | null {
  const index = lines.findIndex((line) => STILL_OPEN_LINE.test(line.trim()));
  if (index < 0) return null;
  const count = Number(STILL_OPEN_LINE.exec(lines[index].trim())?.[1]);
  lines.splice(index, 1);
  return count;
}

function takeHabitTable(lines: string[]): HabitSummary[] | null {
  const start = lines.findIndex((line) => HABIT_HEADER.test(line.trim()));
  if (start < 0) return null;
  let end = start + 1;
  while (end < lines.length && lines[end].trim().startsWith('|')) end += 1;
  const habits = parseHabitRows(lines.slice(start + 1, end));
  if (!habits) return null;
  lines.splice(start, end - start);
  dropHabitsLabel(lines, start);
  return habits;
}

function parseHabitRows(rows: string[]): HabitSummary[] | null {
  const habits: HabitSummary[] = [];
  for (const row of rows) {
    if (TABLE_RULE.test(row.trim())) continue;
    const habit = parseHabitRow(row);
    if (!habit) return null;
    habits.push(habit);
  }
  return habits;
}

function parseHabitRow(row: string): HabitSummary | null {
  const cells = row
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((cell) => cell.trim());
  if (cells.length !== 4) return null;
  const done = asCount(cells[1]);
  const skipped = asCount(cells[2]);
  const notCompleted = asCount(cells[3]);
  if (done === null || skipped === null || notCompleted === null) return null;
  return { title: cells[0], done, skipped, notCompleted };
}

function asCount(value: string): number | null {
  if (!/^\d+$/.test(value)) return null;
  return Number(value);
}

function dropHabitsLabel(lines: string[], from: number): void {
  for (let index = from - 1; index >= 0; index -= 1) {
    const trimmed = lines[index].trim();
    if (!trimmed) continue;
    if (HABITS_LABEL.test(trimmed)) lines.splice(index, 1);
    return;
  }
}
