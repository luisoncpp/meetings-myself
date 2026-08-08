const WEEK_RE = /^(\d{4})-W(\d{2})$/;

export function parseWeek(week: string): { year: number; num: number } {
  const match = WEEK_RE.exec(week);
  if (!match) throw new Error(`invalid week: ${week}`);
  return { year: Number(match[1]), num: Number(match[2]) };
}

export function formatWeek(year: number, num: number): string {
  return `${year}-W${String(num).padStart(2, '0')}`;
}

function weeksInYear(year: number): number {
  const dec28 = new Date(Date.UTC(year, 11, 28));
  const day = dec28.getUTCDay() || 7;
  const thursday = new Date(dec28);
  thursday.setUTCDate(dec28.getUTCDate() + 4 - day);
  const jan1 = new Date(Date.UTC(thursday.getUTCFullYear(), 0, 1));
  return Math.ceil(((thursday.getTime() - jan1.getTime()) / 86_400_000 + 1) / 7);
}

export function prevWeek(week: string): string {
  const { year, num } = parseWeek(week);
  if (num > 1) return formatWeek(year, num - 1);
  const priorYear = year - 1;
  return formatWeek(priorYear, weeksInYear(priorYear));
}

export function nextWeek(week: string): string {
  const { year, num } = parseWeek(week);
  const last = weeksInYear(year);
  if (num < last) return formatWeek(year, num + 1);
  return formatWeek(year + 1, 1);
}
