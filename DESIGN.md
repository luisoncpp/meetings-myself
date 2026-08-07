<!-- SEED: re-run $impeccable document once there's code to capture the actual tokens and components. -->
---
name: Self-Planning
description: A calm, restrained planning surface — organized like a card desk, quiet like a focused reading room.
---

# Design System: Self-Planning

## 1. Overview

**Creative North Star: "The Quiet Ledger"**

The interface is a personal ledger, not a performance arena. Organization comes from clear card surfaces and scannable lists — the spatial logic of Trello boards and Google Keep notes — held inside a restrained dark environment with rare gold accent moments, echoing the calm focus of a late-night reading app. The Daily Plan is the visual and navigational center; Library and Weekly Review are peers in capability but never compete for attention on the home surface.

Density is earned, not decorative. Users see real state — archived, overdue, corrected — without dashboard theatrics. Structure feels supportive: ordering, cadence, and lifecycle are visible but never punitive. Motion confirms state changes; it does not choreograph arrival.

This system explicitly rejects Duolingo energy: no streaks, no confetti, no mascot theatrics, no leaderboard psychology. It also rejects SaaS productivity dashboards — widget grids, hero metrics, purple-gradient accents, and workspace-OS density that treats planning as analytics.

**Key Characteristics:**
- Restrained palette: tinted neutrals carry the surface; accent appears on ≤10% of any screen.
- Single sans family across all roles — headings, labels, body, data.
- Card-and-list organization for Tasks, Habits, and Library entries; columns and groupings where order matters.
- Flat surfaces at rest; depth appears only on interaction (hover, focus, elevation).
- Responsive motion: fast state feedback, no page-load choreography.
- Daily Plan prominence; Weekly Review and Library stay accessible without visual competition.

## 2. Colors: The Focused Night Palette

The palette is **Restrained**: a dark charcoal foundation with cool-tinted neutrals and a single muted gold accent used sparingly. Warmth and personality live in typography rhythm and accent rarity, not in cream body backgrounds or saturated decorative fills.

**Anchor reference:** deep charcoal base, soft gold accent, translucent grey layering — as in the MidnightReads reference — adapted for a planning tool, not a reading-stats dashboard.

### Primary
- **Muted Ledger Gold** ([to be resolved during implementation]): Primary actions, current selection, progress indicators, and focus rings. Never backgrounds, never decorative fills, never large hero surfaces.
- **Gold Deep** ([to be resolved during implementation]): Hover and pressed states on primary actions only.

### Neutral
- **Charcoal Base** ([to be resolved during implementation]): Primary app background — the default reading surface for Daily Plan work.
- **Surface Lift** ([to be resolved during implementation]): Cards, list rows, and panel backgrounds one step above base.
- **Ink Primary** ([to be resolved during implementation]): Headings and primary body text — must meet ≥4.5:1 against its surface.
- **Ink Muted** ([to be resolved during implementation]): Secondary labels, metadata, timestamps — must still meet ≥4.5:1; never washed-out grey for elegance.
- **Hairline** ([to be resolved during implementation]): Dividers and subtle borders — prefer tonal separation over visible lines where possible.

### Named Rules
**The One Accent Rule.** The gold accent appears on ≤10% of any given screen. Its rarity is the point. If everything glows, nothing is primary.

**The Real State Rule.** Overdue, archived, skipped, and corrected states use semantic neutrals and restrained tint shifts — never punitive red storms, never celebratory green floods.

## 3. Typography: One Voice

**Display Font:** [single sans family to be chosen at implementation — humanist or neutral technical sans]
**Body Font:** [same family as display]
**Label/Mono Font:** [same family; tabular figures for counts and dates if the chosen family supports them]

**Character:** One family, many weights. Quiet confidence — no display pairing, no editorial contrast for decoration. The type should feel as familiar as Trello labels and Keep note titles: immediately readable, never shouting.

### Hierarchy
- **Display** ([weight to be chosen], [fixed rem size — not fluid clamp], [line-height ~1.1–1.2]): Screen titles only — Daily Plan date, Weekly Review week label. Rare; never on buttons or data cells.
- **Headline** ([weight], [size], [line-height]): Section headers within a surface — "Today's Tasks", "Weekly Focus", "Habits".
- **Title** ([weight], [size], [line-height]): Card titles, Task names, Habit names, Goal names.
- **Body** ([weight], [size], [line-height ~1.5]): Descriptions, reflection prose, helper text. Max line length 65–75ch in prose blocks.
- **Label** ([weight], [size], [letter-spacing], [case]): Metadata, cadence chips, importance/urgency badges, navigation labels. Uppercase only when the label is genuinely categorical — not as a default section eyebrow on every block.

### Named Rules
**The Fixed Scale Rule.** Product UI uses a fixed rem type scale (ratio ~1.125–1.2). No fluid clamp headings — sidebars and dense panels must not shrink hero text into illegibility.

**The No Eyebrow Scaffold Rule.** Do not put a tiny uppercase tracked kicker above every section. Section identity comes from title weight and spacing, not repeated "PROCESS / TASKS / HABITS" grammar.

## 4. Elevation: Flat Until Touched

Surfaces are **flat at rest**. Depth is conveyed through tonal layering — charcoal base, lifted card surfaces, slightly brighter hover states — not ambient shadow soup. Shadows appear only as a response to state: hover lift on interactive cards, focus elevation on keyboard-targeted controls, floating action areas that must separate from content beneath.

Translucent overlays (backdrop blur) are permitted **only** for transient layers — bottom action bars, sort/filter popovers, modal scrims — not as the default card treatment. Permanent content cards are solid surfaces.

### Shadow Vocabulary
- **[to be resolved during implementation]** — all shadow tokens will be defined when component styles exist. Expect one low hover shadow and one focus-ring-adjacent elevation, not a multi-step Material ladder.

### Named Rules
**The Flat-By-Default Rule.** Cards do not ship with border-plus-wide-shadow pairs. Pick tonal lift OR a tight shadow (≤8px blur), never both as decoration.

**The Transient Glass Rule.** Backdrop blur is for overlays in motion or focus — not for every list card. If the whole screen looks frosted, the glass has become wallpaper.

## 5. Components

[To be documented when UI components exist. Seed expectations below guide the first implementation pass.]

Expected primitives once built:
- **Task / Habit cards** — Keep-like scannability; Trello-like reorder affordance on Daily Plan lists.
- **Library list rows** — dense but calm; archived state visible inline, not hidden.
- **Primary action button** — solid gold on dark; rare on screen.
- **Check-in controls** — three clear outcomes (Done / Skipped / Not completed) without streak counters or celebration animation.
- **Navigation** — Daily Plan home prominent; Library and Weekly Review reachable without competing chrome.

## 6. Do's and Don'ts

### Do:
- **Do** land users on the Daily Plan — it is home; other surfaces are peers, not co-equal dashboard tiles.
- **Do** organize work as cards and ordered lists — Trello column logic for sequencing, Keep simplicity for quick scan.
- **Do** use the gold accent only for primary actions, current selection, and meaningful progress — ≤10% of the screen.
- **Do** show archived, overdue, skipped, and corrected entries in context with honest labels — mirror domain truth.
- **Do** use one sans family with a tight fixed scale; tabular figures for dates and counts where available.
- **Do** animate state changes in 150–250ms with ease-out curves; respect `prefers-reduced-motion`.
- **Do** meet WCAG 2.1 AA — including placeholder and muted text contrast, keyboard operability, and visible focus states.

### Don't:
- **Don't** feel like **Duolingo** — no streaks, badges, confetti, mascot energy, leaderboard psychology, or streak-anxiety loops.
- **Don't** build **SaaS productivity dashboards** — no dense widget grids, hero metrics, KPI cards, purple-gradient accents, or workspace-OS density.
- **Don't** gamify habits — no performance scoring, no celebration on check-in, no "you're on fire" messaging.
- **Don't** use gradient text, side-stripe callout borders, or hero-metric templates (big number + small label + supporting stats).
- **Don't** default to glassmorphism on cards — blur is for transient overlays only.
- **Don't** pair 1px borders with wide soft shadows on the same element — the ghost-card pattern is prohibited.
- **Don't** over-round containers — card radius tops out at 12–16px; pills are for tags and compact buttons only.
- **Don't** put display-sized fluid headings on UI chrome — labels and data stay at label/title/body sizes.
- **Don't** hide messy history — if something is archived or overdue, show it honestly instead of cleaning the UI into fiction.
