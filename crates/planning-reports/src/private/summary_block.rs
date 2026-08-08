/// The app owns exactly one region of the report body. Everything outside it
/// belongs to the user and is preserved byte for byte.
pub struct SummaryBlock;

impl SummaryBlock {
    pub const START: &'static str = "<!-- self-planning:summary:start -->";
    pub const END: &'static str = "<!-- self-planning:summary:end -->";

    pub fn replace(body: &str, markdown: &str) -> String {
        let block = format!("{}\n{}\n{}\n", Self::START, markdown.trim_end(), Self::END);
        let Some((before, after)) = Self::split(body) else {
            // No usable block: prepend one. Never rewrite or drop existing prose.
            return format!("{block}\n{body}");
        };
        format!("{before}{block}{after}")
    }

    pub fn reflection(body: &str) -> String {
        let Some((before, after)) = Self::split(body) else {
            return body.to_string();
        };
        format!("{before}{after}")
    }

    pub fn extract(body: &str) -> Option<String> {
        let start = body.find(Self::START)?;
        let end_marker = body[start..].find(Self::END)? + start;
        let content_start = start + Self::START.len();
        let content = body[content_start..end_marker].strip_prefix('\n').unwrap_or(&body[content_start..]);
        Some(content.trim_end().to_string())
    }

    /// Splits into (text before the block, text after it). Returns None when the
    /// markers are absent or the start marker is unterminated — in both cases the
    /// safe move is to treat the whole body as the user's.
    fn split(body: &str) -> Option<(&str, &str)> {
        let start = body.find(Self::START)?;
        let end_marker = body[start..].find(Self::END)? + start;
        let after = end_marker + Self::END.len();
        let after = body[after..].strip_prefix('\n').map_or(after, |_| after + 1);
        Some((&body[..start], &body[after..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body_with_block() -> String {
        format!(
            "{}\n## Week in review\n\nOld numbers.\n{}\n\n## Reflection\n\nMy own words.\n",
            SummaryBlock::START,
            SummaryBlock::END
        )
    }

    #[test]
    fn replacing_the_summary_leaves_every_other_byte_alone() {
        let updated = SummaryBlock::replace(&body_with_block(), "## Week in review\n\nNew numbers.");
        assert!(updated.contains("New numbers."));
        assert!(!updated.contains("Old numbers."));
        assert!(
            updated.ends_with("\n## Reflection\n\nMy own words.\n"),
            "the reflection must survive exactly: {updated:?}"
        );
    }

    #[test]
    fn replacing_twice_is_stable() {
        let once = SummaryBlock::replace(&body_with_block(), "A");
        let twice = SummaryBlock::replace(&once, "A");
        assert_eq!(once, twice, "regeneration must not accumulate markers or blank lines");
    }

    #[test]
    fn text_the_user_wrote_above_the_block_survives() {
        let body = format!("My preamble.\n\n{}\nold\n{}\n", SummaryBlock::START, SummaryBlock::END);
        let updated = SummaryBlock::replace(&body, "new");
        assert!(updated.starts_with("My preamble.\n\n"));
        assert!(updated.contains("new"));
    }

    #[test]
    fn a_body_whose_markers_were_deleted_gets_them_back_without_losing_prose() {
        let body = "## Reflection\n\nI deleted the app's block.\n";
        let updated = SummaryBlock::replace(body, "regenerated");
        assert!(updated.contains("regenerated"));
        assert!(
            updated.contains("I deleted the app's block."),
            "restoring the block must never cost the user a word"
        );
        // And it is stable from then on.
        assert_eq!(SummaryBlock::replace(&updated, "regenerated"), updated);
    }

    #[test]
    fn the_reflection_is_the_body_without_the_block() {
        let reflection = SummaryBlock::reflection(&body_with_block());
        assert!(!reflection.contains("Old numbers."));
        assert!(!reflection.contains(SummaryBlock::START));
        assert_eq!(reflection.trim(), "## Reflection\n\nMy own words.");
    }

    #[test]
    fn an_unterminated_start_marker_does_not_swallow_the_rest_of_the_file() {
        let body = format!("{}\nunclosed\n\n## Reflection\n\nMine.\n", SummaryBlock::START);
        let updated = SummaryBlock::replace(&body, "new");
        assert!(updated.contains("Mine."), "a corrupt marker must not delete prose");
    }

    #[test]
    fn extract_returns_the_summary_region_contents() {
        let body = body_with_block();
        assert_eq!(
            SummaryBlock::extract(&body),
            Some("## Week in review\n\nOld numbers.".into())
        );
    }
}
