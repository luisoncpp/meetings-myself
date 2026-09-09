# Flex on `<summary>` hides the disclosure in WebView2

**Date:** 2026-09-08

## What to know before starting

Tauri on Windows uses WebView2 (Chromium). `summary { display: flex }` plus hiding the details marker can leave a collapsed `<details>` with **no visible label** — an empty card, not a broken data load.

Put layout (`display: flex`) on an inner wrapper or a real `<button>`, not on `<summary>`. Prefer `aria-expanded` on a button when the control must stay visible while closed.
