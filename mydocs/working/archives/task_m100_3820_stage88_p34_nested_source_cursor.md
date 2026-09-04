---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 88 — p34 nested-table source cursor

## Scope and oracle

- Target HWP: `samples/76076_regulatory_analysis.hwp`
- Visual oracle: `samples/issue1891/76076_regulatory_analysis-2024.pdf`, p33--p34
- HWPCTRL references: `mydocs/manual/webhwpctrl_compat_development.md`,
  `tools/hwpctrl_compat/README.md`

This remains a native HWP renderer/PDF fidelity correction. HWPCTRL API ledger,
COM fixture collection, and WASM control compatibility are not pixel or source
cursor oracles here; no HWPCTRL fixture is updated unless the public control
contract itself changes.

## Stage 87 handoff observation

Stage 87 correctly moved p33's row-6 nested fragment to `y=400.42px` and
preserved p34's fragment geometry (`y=77.1`, `h=426.9`). Direct PDF raster
inspection nevertheless shows a separate semantic failure:

| page | Hancom 2024 PDF | current native render |
| --- | --- | --- |
| 33 | final source line `현황 추이(p.270)` | same source line, fully visible |
| 34 | starts `- 자율안전확인신고한 …` | repeats `현황 추이(p.270)`, then continues |

Thus p34's nested fragment box is correct but its source-unit cursor is one
line too early. The existing test only rejected a *partially clipped* duplicate,
so a fully repainted duplicate escaped it. This stage must add a semantic
continuation assertion and repair the cursor; changing fragment geometry or
baseline coordinates is not an acceptable workaround.

## Investigation and correction

The outer p33 cut is `rows=0..7`, `end_cut=[1,37]`; the p34 continuation is
`rows=6..7`, `start_cut=[1,37]`. In both fragments the relevant child is a
native HWP5, non-TAC, 1×1 nested table in the final row of a TopAndBottom,
Para-relative `RowBreak` parent. Its child source is already consumed through
the p33 `현황 추이(p.270)` line.

The existing terminal-tail guard disabled `fragment_cut_units` for p34. The
renderer therefore restarted the child from source unit zero, moved the old
lines above the physical clip, and happened to leave `현황 추이(p.270)` visible
at p34's top. It was a full duplicate rather than a clipped residue.

`native_terminal_rowbreak_child_source_cursor_eligible` now recognizes only
that existing native final-row 1×1 block-child shape. It does not make a new
row splittable and does not alter HWPCTRL/WASM public contracts. For this
already-source-cut terminal continuation it enables the child source start
cursor and resets only the redundant physical start offset; retaining both
would skip p34's next source line as well.

The existing short-child condition remains in force for p81→p82 and continues
to keep that different owner-content-box contract intact.

## Verification

- Direct render-tree inspection: p33 retains a fully painted `현황 추이`, while
  p34 no longer contains that p33-owned text and begins
  `자율안전확인신고한 …`.
- Direct raster comparison at 144 DPI against the Hancom 2024 PDF confirms the
  p34 semantic page boundary now agrees. This stage deliberately does not
  claim unrelated font metrics or broad p33/p34 table-fidelity differences are
  resolved.
- Focused regression:
  `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/pr-review cargo test --profile release-test --test issue_2308_render_normalized_derived_state -- --nocapture`
  → 5 passed, 0 failed.

## Next stage

Continue PDF-led review of the remaining p33--p34 table geometry and text
metrics as a separate defect. Do not weaken the source-boundary assertion or
alter HWPCTRL fixtures for this renderer-only correction.
