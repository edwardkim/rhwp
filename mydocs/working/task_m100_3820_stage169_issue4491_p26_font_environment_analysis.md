---
kind: investigation
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-12
---

# Task M100 #3820 Stage 169 - #4491 p26 font-comparable fidelity boundary

## Purpose

Determine whether the residual #4491 physical p26 PDF difference is a renderer
layout regression that can justify another pagination change after Stage 168.

## Inputs and execution

- Source: `samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp`
- Reference: `pdf/issue4491/30213_1-혼합단지등-제도개선-방안-hancom2020.pdf`
- Renderer: `target/task-3820-stage168/release-test/rhwp`
- Comparison: `tools/fidelity_compare/fidelity_compare.py 25 25` with the direct
  source/reference pair. The fidelity tool is zero based, so index `25` is
  physical p26. `dump-pages -p` is also zero based despite its older help text.
- Artifact root: `output/task-3820-stage169-issue4491-p26/`

## Observations

| Check | Result |
| --- | --- |
| p26 pixel difference | `26.31%` |
| physical page owner | Same page and same item order as the HWP 2020 PDF |
| p26 flow | Tables `pi=337`, `341`, and `344` are inline (`treatAsChar:true`); no empty-host RowBreak declared-height gate participates |
| p26 geometry | The comparison sheet keeps the three table boxes, heading, body paragraphs, and footer on their source-owned page without overlap or clipping |
| reference PDF fonts | `Gulim`, `HCRDotum`, and `DejaVuSerif` |
| SVG source face | `한양중고딕` routes through an `HCR Dotum` local-face candidate |
| local face availability | `fc-match 'HCR Dotum'` resolves to `Verdana`; `fc-match 'HY중고딕'` resolves to the installed `HYGothic-Medium` (`H2GTRM.TTF`) |
| portable face input | `RHWP_FONT_PATH_DIR` is unset and no HCR/Haansoft Dotum asset is tracked under `ttfs`, `fonts`, or `resources` |

The p26 comparison sheet shows matching page ownership and box geometry but
different glyph weight and raster width. `HYGothic-Medium` can be selected after
the unavailable HCR candidate, but it is not the HCR Dotum face embedded by the
reference PDF. Stage 75 independently established that selecting this real HY
face does not, by itself, repair the related 76076 p33--p36 fidelity gap. The
PDF text layer also has no extracted p26 text while the SVG has 790 characters,
so the text ledger is not usable as an owner or line-break oracle for this page.

## Stage 168 compile completion

The carried Stage 168 implementation had four compile errors unrelated to a
new p26 pagination hypothesis:

- three `saved_bounds_fit_at_flow_tail` call sites omitted the new
  `bottom_spill` argument; all ordinary saved-line fits now explicitly pass
  `0.0`;
- the mid-page RowBreak scope now recomputes its saved object bottom from the
  same first real LineSeg and table vertical offset used by the declared-height
  branch, instead of referring to the branch-local `saved_span`.

## Verification

- `cargo build --profile release-test --target-dir target/task-3820-stage168`
  completed successfully.
- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_4490_4491_anchor_flow -- --nocapture`
  passed: 2 passed, 0 failed.

## Decision

Do not alter pagination, table splitting, or text measurement based on this
p26 raster comparison. A future p26 acceptance comparison must provide the
same HCR Dotum face to Chrome through the documented font-path contract or an
equivalent licensed local installation. Continue #3820 with an independent
candidate whose defect remains visible under the available font environment.
