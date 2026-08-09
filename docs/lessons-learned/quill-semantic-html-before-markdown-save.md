# Quill semantic HTML before Markdown save

**Date:** 2026-08-09

## What to know

When persisting Quill Next content as Markdown, call `getSemanticHTML()` — not `root.innerHTML`. Semantic HTML keeps list types (`ul` vs `ol`) and block structure stable for Turndown.

Turndown may normalize emphasis (`*text*` → `_text_`) and list indentation. That is fine: the storage contract is Markdown prose, not byte-identical round-trip.

Normalize empty editor shells (`<p><br></p>`) to `''` before saving so blank reflections do not become stray newlines in the weekly report file.

Mount Quill once per editor instance in Svelte. If the mount `$effect` reads `value` or unstable callback props, parent re-renders (e.g. after a heading change) recreate Quill on the same node and stack toolbars. Keep mount dependencies to the DOM node only; sync markdown in a separate effect and route callbacks through a ref.

## Where this applies

- `src/lib/ui/Private/QuillMarkdownHost.ts`
- `src/lib/ui/Private/markdown-codec.ts`
- Weekly Review reflection autosave (`save_reflection`)
