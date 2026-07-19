# Task 261 Plan: Page Reflow Fundamental Fix

## Symptoms

After page break/paragraph insertion, subsequent paragraph y-positions are abnormal:
- "A. Personnel Deployment Plan" disappears
- Abnormal spacing between tables and paragraphs
- E2E reproduction: `e2e/page-break.test.mjs` (biz_plan.hwp, paragraph 68)

## Root Cause

Inconsistency in the `vertical_pos` (vpos) system:
- Original HWP: Each paragraph's `line_segs.vertical_pos` is an absolute value relative to **section start**
- `reflow_line_segs`: Recalculates vpos cumulatively from the original first LineSeg value
- When a page break moves paragraphs to different pages, original vpos and actual in-page position don't match
- layout.rs vpos correction doesn't handle this inconsistency

## Past Related Work

- **Task 142**: Introduced Break Token data structure
- **Task 198**: Extended vpos correction from page_index==0 only to all pages (vpos_page_base / vpos_lazy_base)
- **Task 215**: typeset_block_table() Break Token-based row splitting
- **Research document**: `mydocs/tech/investigations/issue-397/layout_engine_research.md` — Analysis of MS Word, LibreOffice, Chromium LayoutNG, Typst

Since the Break Token pattern was already introduced to resolve measurement-layout mismatches,
applying the same pattern to the vpos issue is a consistent approach.

## Solution Direction

### Option B: Recalculate vpos of subsequent paragraphs after editing (Recommended)
- After paragraph insert/delete/page break, sequentially recalculate vpos for **all affected paragraphs**
- Propagate previous paragraph's `vpos_end` as the next paragraph's `vpos_start`
- For table paragraphs, preserve `line_height` and only update vpos
- Pros: Guarantees vpos system consistency, consistent with Break Token pattern
- Cons: Need to preserve line_height for special paragraphs like tables/shapes

## Reference Files

| File | Role |
|------|------|
| `src/renderer/layout.rs` (1010~1043) | vpos correction logic |
| `src/renderer/composer/line_breaking.rs` | reflow_line_segs (vpos recalculation) |
| `src/renderer/pagination/engine.rs` | Page splitting |
| `src/document_core/commands/text_editing.rs` | insert_page_break_native |

## E2E Tests

- `e2e/page-break.test.mjs`: biz_plan.hwp page break verification
- `e2e/footnote-vpos.test.mjs`: footnote-01.hwp vpos jump verification
