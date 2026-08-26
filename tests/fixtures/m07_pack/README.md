# M07-pack CanvasKit fallback fixtures

Machine-readable contract for issue #5448.

| file | role |
|---|---|
| `reason-matrix.jsonl` | one unique style combination per line, with expected policy status/detail |
| `envelopes.jsonl` | matching one-item `CanvasKitReplayPlan` transcripts |

`src/renderer/canvaskit_m07_pack_contract.rs` rebuilds each matrix row as a
`PaintOp` and compares the live plan. Regenerate with
`python scripts/gen_m07_pack_fixtures.py` after changing the family set.
Do not hand-edit the JSONL files.
