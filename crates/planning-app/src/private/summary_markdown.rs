use super::weekly_summary::{HabitSummary, WeeklySummary};

/// Renders the app-owned region of a Weekly Report.
///
/// Counts and titles only: no percentages, no streaks, no evaluative language.
/// PRODUCT.md is explicit that reflection must not become scoring.
pub fn render(summary: &WeeklySummary) -> String {
    let mut out = String::from("## Week in review\n\n");
    out.push_str(&section("Completed", &summary.completed));
    out.push_str(&section("Overdue", &summary.overdue));
    out.push_str(&section("Goals achieved", &summary.goals_achieved));
    out.push_str(&format!("**Still open:** {}\n\n", summary.still_open));
    out.push_str(&habits(&summary.habits));
    out
}

fn section(heading: &str, items: &[String]) -> String {
    if items.is_empty() {
        return format!("**{heading}:** none\n\n");
    }
    let lines: Vec<String> = items.iter().map(|item| format!("- {item}")).collect();
    format!("**{heading}:**\n\n{}\n\n", lines.join("\n"))
}

fn habits(entries: &[HabitSummary]) -> String {
    if entries.is_empty() {
        return "**Habits:** no check-ins recorded\n".to_string();
    }
    let mut out = String::from("**Habits:**\n\n| Habit | Done | Skipped | Not completed |\n");
    out.push_str("| --- | --- | --- | --- |\n");
    for entry in entries {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            entry.title, entry.done, entry.skipped, entry.not_completed
        ));
    }
    out
}
