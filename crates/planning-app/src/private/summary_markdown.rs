use planning_store::UiLanguage;

pub fn render(summary: &super::weekly_summary::WeeklySummary, language: UiLanguage) -> String {
    let strings = ReportStrings::for_language(language);
    let mut out = format!("{}\n\n", strings.week_in_review);
    out.push_str(&section(
        strings.completed,
        &summary.completed,
        strings.none,
    ));
    out.push_str(&section(strings.overdue, &summary.overdue, strings.none));
    out.push_str(&section(
        strings.goals_achieved,
        &summary.goals_achieved,
        strings.none,
    ));
    out.push_str(&format!(
        "**{}:** {}\n\n",
        strings.still_open, summary.still_open
    ));
    out.push_str(&habits(&summary.habits, &strings));
    out
}

struct ReportStrings {
    week_in_review: &'static str,
    completed: &'static str,
    overdue: &'static str,
    goals_achieved: &'static str,
    still_open: &'static str,
    none: &'static str,
    habits: &'static str,
    no_check_ins: &'static str,
    habit_col: &'static str,
    done_col: &'static str,
    skipped_col: &'static str,
    not_completed_col: &'static str,
}

impl ReportStrings {
    fn for_language(language: UiLanguage) -> Self {
        match language {
            UiLanguage::Es => Self {
                week_in_review: "## Semana en revisión",
                completed: "Completadas",
                overdue: "Atrasadas",
                goals_achieved: "Metas logradas",
                still_open: "Aún abiertas",
                none: "ninguna",
                habits: "Hábitos",
                no_check_ins: "sin registros",
                habit_col: "Hábito",
                done_col: "Hecho",
                skipped_col: "Omitido",
                not_completed_col: "No completado",
            },
            UiLanguage::En => Self {
                week_in_review: "## Week in review",
                completed: "Completed",
                overdue: "Overdue",
                goals_achieved: "Goals achieved",
                still_open: "Still open",
                none: "none",
                habits: "Habits",
                no_check_ins: "no check-ins recorded",
                habit_col: "Habit",
                done_col: "Done",
                skipped_col: "Skipped",
                not_completed_col: "Not completed",
            },
        }
    }
}

fn section(heading: &str, items: &[String], none: &str) -> String {
    if items.is_empty() {
        return format!("**{heading}:** {none}\n\n");
    }
    let lines: Vec<String> = items.iter().map(|item| format!("- {item}")).collect();
    format!("**{heading}:**\n\n{}\n\n", lines.join("\n"))
}

fn habits(entries: &[super::weekly_summary::HabitSummary], strings: &ReportStrings) -> String {
    if entries.is_empty() {
        return format!("**{}:** {}\n", strings.habits, strings.no_check_ins);
    }
    let mut out = format!(
        "**{}:**\n\n| {} | {} | {} | {} |\n",
        strings.habits,
        strings.habit_col,
        strings.done_col,
        strings.skipped_col,
        strings.not_completed_col,
    );
    out.push_str("| --- | --- | --- | --- |\n");
    for entry in entries {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            entry.title, entry.done, entry.skipped, entry.not_completed
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use planning_core::CalendarWeek;
    use planning_store::UiLanguage;

    #[test]
    fn spanish_summary_uses_spanish_headings() {
        let summary = super::super::weekly_summary::WeeklySummary {
            week: CalendarWeek::parse("2026-W32").unwrap(),
            completed: vec!["Tarea".into()],
            overdue: vec![],
            goals_achieved: vec![],
            still_open: 1,
            habits: vec![],
        };
        let markdown = render(&summary, UiLanguage::Es);
        assert!(markdown.contains("## Semana en revisión"));
        assert!(markdown.contains("**Completadas:**"));
    }
}
