---
kind: implementation
status: completed
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-08-13
---

# Task M100 #3820 Stage 170 - saved object flow anchor vs paint inset

## Regression

Completing the carried Stage 168 strict saved-object fit predicates exposed a
real regression in `issue_3820_body_top_table_border_clip`:

```text
p168 table_y=698.24, expected=86.93, body clip_y=83.16
```

The p33 paint-only top-frame case still passed. The affected successor fragment
was `pi=1775` in
`samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`.

`RHWP_DIAG_SCAN=1` recorded:

```text
pi=1775 cur_h=607.5 declared=314.2 avail=956.2
saved=(611.3, 921.7) bottom_fits=false
```

The 3.8px gap is not arbitrary drift. It is exactly the stored `283 HU`
physical top paint inset preserved by the p168/p214 regression contract. Stage
168 compared that physical object top to the flow cursor, so a source-owned
continuation was wrongly emitted at the current page tail and pushed its real
continuation to the next page.

## Repair

`saved_span` now records three values:

1. the unshifted LineSeg flow anchor;
2. the physical object top after positive `vertical_offset`;
3. the physical object bottom.

The declared-fit and split predicates compare the current flow cursor to the
unshifted anchor, while still requiring the physical object bottom to stay
inside (or cross) the body boundary. Existing native HWP5 near-anchor and
internal-reset resync paths keep their intentionally physical-top behavior.

This is source-derived and does not restore a broad pixel tolerance.

## Verification

- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_3820_body_top_table_border_clip -- --nocapture`
  - 2 passed, 0 failed (`p33`, `p168`, and `p214` contracts covered)
- `cargo test --profile release-test --target-dir target/task-3820-stage168 --test issue_4490_4491_anchor_flow -- --nocapture`
  - 2 passed, 0 failed

## Result

Stage 168's saved-object strictness now distinguishes a flow anchor from a
paint-only stored inset. The p168 successor table again starts at the body top
without moving owner geometry or clipping its outer top stroke.
