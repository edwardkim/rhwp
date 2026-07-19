# Task 397: High-Level Text Layout Technology Review (SkParagraph + cosmic-text)

## Objective

Technical review for systematizing rhwp's typesetting system. Analyze two industry-standard text layout engines and derive fundamental improvement directions for rhwp's typesetting system.

- **SkParagraph**: Google Skia's high-level text layout module (used in Flutter, Chrome)
- **cosmic-text**: Pure Rust text layout library developed by System76 (Pop!_OS)

## Background

### Problem Situation

rhwp started as an HWP viewer and has since implemented editor functionality. When documents created in Hancom are loaded and paragraph modifications, additions, or deletions change page layout, numerous detailed bugs occur.

**Root cause**: The text layout structure from the viewer era (pre-typeset LINE_SEG-based) doesn't match the editor's dynamic re-typesetting requirements. Line break recalculation, text measurement, and pagination don't accurately synchronize during paragraph editing, causing layout mismatches.

Systematizing the typesetting system is needed, and the first step is analyzing industry-standard text layout engines to explore fundamental solutions.

### rhwp Current Text Layout Structure

| Component | Current Implementation | Limitations |
|----------|-----------|------|
| Text measurement | 582 built-in font metrics + WASM JS Canvas bridge | Unregistered fonts use CJK=1.0, Latin=0.5 heuristic |
| Text shaping | None (per-character independent measurement) | Ligatures, kerning, contextual substitution not supported |
| Line breaking | Custom implementation (Korean syllable/English word/CJK character) | Has prohibited character handling but not fully UAX#14 compliant |
| Font fallback | None (built-in metrics-based) | Heuristic fallback for unregistered fonts |
| BiDi | None | RTL text not supported |
| Bold handling | Faux Bold correction (em+10)/20 | Empirical values, not actual glyph metrics |

### Technology Summary

| Item | SkParagraph | cosmic-text |
|------|-------------|-------------|
| Language | C++ (skia-safe Rust bindings available) | Pure Rust |
| Shaping | HarfBuzz + ICU | harfrust (HarfBuzz Rust port) |
| Line breaking | ICU-based | unicode-linebreak + language-specific handling |
| BiDi | ICU BiDi | unicode-bidi |
| Font fallback | Platform-specific + custom | fontdb + platform-specific fallback lists |
| WASM | Supported via CanvasKit | Supported (no_std option) |
| License | BSD-3 | Apache-2.0 / MIT |

## Implementation Plan

### Step 1: SkParagraph Deep Analysis (Research)

- SkParagraph architecture analysis
  - ParagraphBuilder → Paragraph → layout() → paint() pipeline
  - TextStyle / ParagraphStyle / StrutStyle property mapping
  - LineMetrics, getRectsForRange(), getGlyphPositionAtCoordinate() query APIs
- HWP text attribute mapping analysis
  - HWP CharShape ↔ TextStyle
  - HWP ParaShape ↔ ParagraphStyle/StrutStyle
  - HWP LINE_SEG ↔ LineMetrics
- skia-safe Rust binding status and WASM build feasibility investigation
- Pros/cons summary

### Step 2: cosmic-text Deep Analysis (Research)

- cosmic-text architecture analysis
  - FontSystem → Buffer → ShapeLine → LayoutLine pipeline
  - Attrs (attributes), Metrics, Wrap/Align options
  - Editor layers (basic/syntax highlighting/Vi)
- Core feature verification
  - Korean text shaping and line breaking behavior
  - Font fallback mechanism (fontdb-based)
  - BiDi support level
  - Caching strategy (Shape/Layout/Font 3-tier)
- WASM build feasibility and constraints investigation
- Pros/cons summary

### Step 3: Comparative Analysis and rhwp Application Plan

- SkParagraph vs cosmic-text comparison table
  - Feature coverage, performance, WASM compatibility, maintainability, license
- rhwp application scenario analysis
  - **Scenario A**: Adopt cosmic-text as text measurement/shaping engine
  - **Scenario B**: Full adoption of SkParagraph (skia-safe)
  - **Scenario C**: Partial cosmic-text adoption (shaping only, layout stays in rhwp)
  - **Scenario D**: Maintain current + selective improvements
- Impact scope, migration difficulty, and risk analysis per scenario
- Final technical review report (`mydocs/tech/`)

## Deliverables

- `mydocs/tech/investigations/issue-397/text_layout_review.md` — Technical review report
- `mydocs/working/task_397_step{N}.md` — Per-step completion reports

## Notes

- This is a research/analysis task with no code changes
- Follow-up implementation tasks to be registered based on conclusions
