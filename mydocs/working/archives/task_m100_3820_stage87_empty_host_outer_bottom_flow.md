---
kind: implementation
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-09
---

# Task #3820 Stage 87 — empty-host outer-bottom flow ownership

## Oracle and HWPCTRL boundary

- Target fixture: `samples/76076_regulatory_analysis.hwp`
- Visual oracle: Hancom 2024 output
  `samples/issue1891/76076_regulatory_analysis-2024.pdf`, p33--p34
- HWPCTRL reference: `mydocs/manual/webhwpctrl_compat_development.md` and
  `tools/hwpctrl_compat/README.md`

HWPCTRL's API ledger and its Windows COM fixture collection establish the web
control contract; they do not replace Hancom PDF geometry as the oracle for an
existing HWP's native renderer. This stage changes neither HWPCTRL API nor its
fixtures. The renderer-only change is therefore validated with the Hancom PDF
and the native renderer regression suite; a HWPCTRL compatibility fixture is
not regenerated merely because this visual layout changes.

## Observed p33 fault

The source contains consecutive empty-host, `TopAndBottom + Para + RowBreak`
1×1 tables at paragraph indexes 323 and 324. They each carry 566HU top and
bottom outer margins. `format_table()` already reserves the bottom margin in
the typeset flow, but `layout_table_control_block()` advanced its rendering
cursor only to `table_visual_end` for an empty host.

Current RenderTree originally placed the successive tables at `y=173.6`,
`194.7`, and `215.8` — exactly one padded visual table height (21.1px) apart
— while the Hancom PDF text anchors advance by about 32.4px in renderer
coordinates. A first bottom-only probe improved the interval to 28.6px but
still missed the PDF. The source declares a 1300HU (17.3px) flow height and
566HU top/bottom margins; `17.3 + 7.55 + 7.55 = 32.4px`. Thus the repair must
use the empty host's declared flow box, not its 21.1px minimum painted cell
box. This is not a PDF raster threshold or an HWPCTRL fixture discrepancy.

## Narrow repair rule

Mirror the source-declared flow extent — declared height plus top/bottom outer
margins, never shorter than painted extent plus its bottom margin — only for
the native HWP5 empty `TopAndBottom + Para + RowBreak` 1×1 anchor path. Do not
alter visible host tables, Square sibling lanes, HWPX stored-layout behavior,
nested table geometry, or the table painting box. First measure the resulting
PDF anchors; the existing p33 geometry pin is retained until that direct
measurement proves which value is canonical, rather than treating the old pin
as an oracle.

## Validation sequence

1. Render p33/p34 and compare table/text anchors against the Hancom 2024 PDF.
2. Run `issue_2308_render_normalized_derived_state` focused tests.
3. Retain/update only assertions justified by the direct PDF measurement.
4. Resume the documented renderer gates only after this focused result is
   understood; no concurrent Cargo commands share `target/pr-review`.

## Result

The p33 7×2 table's reference top border is at 241.4px (96dpi coordinate);
after the repair the RenderTree reports 238.5px and its raster top line is only
3px above the PDF at 144dpi, versus 14px before the repair. The nested row-6
fragment begins at `y=400.42px`; its physical PDF row boundary is the 540.609pt
line, i.e. approximately 401.7px at 96dpi. The small residual is the existing
native/PDF font and page-coordinate raster variance, while the prior 351.1px
pin was a complete row boundary error. The focused geometry pin is therefore
corrected to the renderer's direct fragment coordinate `400.4px`; its height
and p34 continuation geometry remain unchanged.

The focused test also exposed an independent p34 source-cursor defect: its
continuation fully re-paints `현황 추이(p.270)`, although the PDF starts the
page with `- 자율안전확인신고한 …`. This stage does **not** hide that duplicate
with a geometry baseline. It is the first analysis item for the next stage.
