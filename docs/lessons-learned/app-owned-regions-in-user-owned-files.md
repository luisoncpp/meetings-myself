# App-Owned Regions in User-Owned Files

**Date:** 2026-08-08

## What looks intuitive

The app writes a Markdown file, so the app owns the whole file. Regeneration means rewrite the document from a template. If the user edits externally, merge their changes in — or parse the body into sections and rebuild.

## What we did

Co-own the file with the human:

- **App owns:** YAML front matter (replaced wholesale) plus one region between `<!-- self-planning:summary:start -->` and `<!-- self-planning:summary:end -->` (replaced wholesale).
- **User owns:** every other byte — reflection, custom headings, preamble above the block, sections below, whitespace.

`SummaryBlock::replace` is **idempotent**: running it twice with the same content must not accumulate markers or blank lines.

## Missing or corrupt markers

If markers are absent, unpaired, or unterminated, the safe move is: **the whole body is theirs**. `replace` prepends a fresh summary block; it never rewrites or drops existing prose. `reflection()` returns the full body unchanged.

An unterminated start marker must not swallow the rest of the file.

## The counter-intuitive part

The dangerous operation is not **writing** — it is **parsing**.

A naive `split("---")` on the file text treats every Markdown horizontal rule as a YAML delimiter. A user who types `---` between reflection sections loses everything after the first rule, with no error anywhere. The failure looks like silent data loss.

Correct parsing consumes exactly the **first two** `---` lines that stand alone (opening and closing front matter). A `---` later in the body is ordinary prose.

## Generalizable lesson

When a file must stay editable in an external editor:

1. Own the smallest possible regions (metadata + one delimited block).
2. Make regeneration idempotent.
3. Treat parse failures as "preserve everything" — never guess.
4. Test the parser against user-authored edge cases (horizontal rules, deleted markers, text above the block) before testing the happy path.
