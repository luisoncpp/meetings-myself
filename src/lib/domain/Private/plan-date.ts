/** Parses `YYYY-MM-DD` as a local calendar day — never UTC midnight. */
export function parsePlanDate(isoDate: string): Date {
  const [year, month, day] = isoDate.split('-').map(Number) as [number, number, number];
  return new Date(year, month - 1, day);
}

/** Long-form label for a plan date in the given BCP-47 locale. */
export function formatPlanDate(isoDate: string, locale: string): string {
  return new Intl.DateTimeFormat(locale, { dateStyle: 'long' }).format(parsePlanDate(isoDate));
}
