# Stage 176 - Circled heading and RowBreak table page owner correction

## Scope

After rebasing the #3820 branch onto `upstream/devel`, the 76076 reference
ledger still had two real owner mismatches: the circled heading `③ 대안의 선택
및 근거` was painted at the bottom of rhwp p55 and p70, while Hancom begins the
heading and its explanatory table together on the following page.

## Source pattern

Both occurrences have the same native HWP5 structure:

1. A one-line circled subheading with no controls.
2. Exactly one real empty carrier paragraph.
3. A single, non-TAC, `TopAndBottom` 1x1 `RowBreak` table containing the body.

The heading has no explicit `keep-with-next` flag, so a generic style-based
rule would be unsound.  The correction is instead limited to this structural
pattern and only triggers when the current tail can hold the heading but cannot
hold the heading, carrier, and the established minimum visible table fragment.

## Guards

- A single column only; native HWP5 may omit the otherwise auxiliary carrier
  `LINE_SEG`, so that omission is not treated as contrary evidence.
- The current page already owns visible content.
- Exactly one semantically empty carrier paragraph and a table-only host.
- A declared positive table height, one cell, and non-character
  `TopAndBottom` `RowBreak` semantics.
- Fresh-page capacity for the grouped minimum.

This leaves ordinary section titles, explicit keep properties, multi-cell
tables, synthetic carriers, and unrelated table anchors on their existing
pagination paths.

## Validation target

Rebuild the release-test renderer, run the #3820 integration suite and the
76076 text-owner ledger.  The p55->p56 and p70->p71 early-owner entries must
disappear without reintroducing p4/p18/p35 RowBreak regressions.

## Validation result

- `issue_3820_rowbreak_rowspan_band`: 4 passed.
- `issue_3820_body_top_table_border_clip`: 2 passed.
- `issue_4490_4491_anchor_flow`: 2 passed.
- `issue_4090_hwpx_tail_page_break`: 1 passed.
- `76076-stage176-upstream-circled-owner-v2`: reference PDF, rhwp SVG, and
  render tree all have 82 pages.  The p55->p56 and p70->p71 owner candidates
  are absent.

The remaining p6->p7 and p38->p39 owner rows are intentionally retained as
conservative repeated-character-intersection candidates.  Their prior PDF/SVG
visual review shows the same table and heading/body owners on both sides of
each boundary; they are not page-boundary defects and this correction does not
alter either page pair.
