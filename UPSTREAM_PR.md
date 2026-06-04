# Fix hit-test caret snapping to line start/end on leading-gap (inter-line) clicks

## Summary

`DocumentCore::hit_test_native` (in `src/document_core/queries/cursor_rect.rs`)
maps a click `(x, y)` on a rendered page to a caret position
(`sectionIndex` / `paragraphIndex` / `charOffset` + `cursorRect`). It resolved
the click **x** to the precise character under the cursor **only when the click
y fell inside a TextRun's glyph bounding box**.

When the click instead landed in a line's **leading gap** — the few pixels of
inter-line leading above the glyph box top, where real pointer clicks routinely
land — the cell-run, body `same_line_runs`, and closest-line fallbacks all
ignored x and snapped the caret to the **line start** (`char_start`) or **line
end** (`char_start + char_count`). Multi-run lines had a related defect: any x
past the first run snapped to the **line end**, because those fallbacks never
performed per-run x resolution and the gap between two runs of one visual line
was never handled.

Observed on `samples/exam_social.hwp`, page 0, the left-column body line whose
glyph box top is `y ≈ 482.7`:

| click (x, y)            | upstream caret x | expected (tracks x) |
|-------------------------|------------------|---------------------|
| (200, 488) — in box     | ≈ 204 (correct)  | ≈ 204               |
| (200, 481.5) — gap      | ≈ 120 (line start) | ≈ 204             |

i.e. a click ~1px above the glyph box snaps the caret ~80px to the start of the
line instead of landing under the cursor.

## The fix

Decouple x-resolution from "y is inside a glyph bbox". The three divergent
fallback paths (cell nearest-run, body `same_line_runs`, closest-line) are
consolidated into a single helper, `resolve_x_on_line`, that takes the runs of
one visual line (sorted by `bbox_x`) and maps the click x to the nearest
character regardless of y:

- x inside a run bbox  → exact character via `find_char_at_x`;
- x in the gap between two runs → the nearer boundary (left run end / right run
  start);
- x left of the first run → first-run start;
- x right of the last run → last-run end.

The cell path now picks the target line by glyph bbox first and, for leading-gap
clicks, falls back to the **vertically nearest run's line**, then resolves x
across that whole line. So the line is still chosen by y, but x is always
resolved within the chosen line — the click no longer has to be inside the glyph
box to get a correct caret column.

`resolve_x_on_line` is lifted to module scope (operating on a small
`LineRunView` borrow of the fields it needs) so the line-level x logic is unit
testable without building a document / page tree. The call sites are unchanged
behaviorally and continue to pass `&RunInfo` data via the lightweight view; the
per-character mapping keeps the exact same semantics as before.

One additional guard: the resolved in-run character index is clamped to the
run's `char_count`. An empty input cell (e.g. an answer-sheet name field) has a
run with `char_count = 0` but a `bbox_w` spanning the whole cell, and a single
`char_positions = [0.0]`; the underlying `find_char_at_x` returns
`positions.len()` (= 1) for a click inside that wide box, which would push the
caret to `char_start + 1` — one past the (empty) content. Clamping keeps such
clicks at `char_start`, matching the prior behavior for empty cells (verified
against `tests/issue_850_answer_sheet_name_hit_test.rs`, which inserts text at
the hit offset and reads it back).

## Regression test

Two layers, both kept:

1. **Unit tests** in the `cursor_rect` test module (`#[cfg(test)] mod tests`)
   exercise `resolve_x_on_line` directly: a mid-line x resolves to a mid-line
   offset (not start/end), an interior x sweep is monotonic and spans many
   distinct offsets, an out-of-line click clamps to start/end, an inter-run-gap
   click snaps to the nearer run boundary rather than the line end, and an empty
   run with a wide bbox clamps to `char_start` (the empty-input-cell guard).

2. **Integration test** `tests/hit_test_leading_gap.rs` drives the public
   `hit_test_native` on `samples/exam_social.hwp` (already in the repo — no new
   fixture). It sweeps the click x across the target line at two click y values:
   one inside the glyph box (`y = 488.0`) and one in the leading gap
   (`y = 481.5`), asserting the caret x tracks the click x, the offsets are
   monotonic, and the sweep resolves to many distinct offsets (a snap-to-edge
   bug collapses the whole interior to one offset).

   The `inside_glyph_box` case passes on unmodified `devel`; the `leading_gap`
   case **fails on unmodified `devel`** (caret x snaps to ≈120 / Δ≈80px at the
   line start) and **passes with this change**.

## Optional follow-up (flagging only)

The web frontend's JS hit-test
(`web/text_selection.js`, `TextLayoutManager.hitTest`) currently re-implements
click → caret resolution in JavaScript. It could migrate to this now-correct
rust `hitTest` for a single source of truth — a larger change, just flagging it.
