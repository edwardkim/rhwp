---
kind: reference
status: active
canonical: mydocs/tech/canvaskit-parity-implementation.md
last_verified: 2026-09-03
---

# CanvasKit M07-pack fallback matrix

This is the public contract for issue #5448. Policy lives in
`src/renderer/canvaskit_policy.rs`. Runtime replay lives in
`rhwp-studio/src/view/canvaskit-renderer.ts`. The machine-readable
matrix is `tests/fixtures/m07_pack/reason-matrix.jsonl` and the
envelope transcripts are `tests/fixtures/m07_pack/envelopes.jsonl`.

## Direct promotions

| reason | was | now | proof |
|---|---|---|---|
| `lineArrow` | overlay | Direct | `drawArrowHead` |
| `compoundLine` | overlay | Direct | `drawCompoundLine` |
| `shapeShadow` | overlay | Direct | `resolvedShadow` + translate |
| `lineShadow` | overlay | Direct | same, then compound stroke |
| `patternFill` | overlay | Direct | `drawPatternFill` |
| `unsupportedTextDecoration` | overlay on shape>12 | Direct local no-op | default solid |
| `invalidTabLeader` fill>11 | overlay | Direct skip | switch default |
| `footnoteMarker` | already Direct | Direct | `renderTextRun` |
| `visualItemLimitExceeded` | overlay | unchanged | 4096 shared bound |

Matrix rows: 4308.

## Family counts

| family | rows |
|---|---:|
| `compoundLine` | 50 |
| `crossVector` | 360 |
| `footnoteMarker` | 320 |
| `invalidTabLeader` | 128 |
| `lineArrow` | 1458 |
| `lineShadow` | 576 |
| `patternFill` | 144 |
| `shapeShadow` | 864 |
| `unsupportedTextDecoration` | 384 |
| `visualItemLimitExceeded` | 24 |

## Fail-closed remainder

- `invalidGeometry` for non-finite bbox/baseline/fontSize/ratio.
- `invalidTabLeader` for inverted or non-finite leader ranges.
- `visualItemLimitExceeded` above 4096 projected display items.
- `verticalText` / `rotatedText` on special-visual ops.
- `lineTransform` / `shapeTransform` / `gradientFill` / `imageFill`.
- `scriptTextRequiresShaping` for complex scripts without cluster authority.

## Envelope fields

Each envelope row is a one-item `CanvasKitReplayPlan` transcript:
`mode`, `hiddenCanvas2dOverlayAllowed`, `directReplayRequired`,
`summary`, and `items[0].{opType,status,reason,detail}`.
The Rust loader rebuilds the same paint op and compares the live plan.
