# Task M100 #1017 Stage 5 보고서 — CanvasKit replay plan 진단 갱신

## 1. 범위

Stage 5에서는 Rust `CanvasKitReplayPlan` 진단 item에 replay plane 정보를 추가했다.

렌더러 동작 변경은 Stage 3/4에서 완료했고, 이번 단계는 진단 JSON이 적용된 z-order policy를 드러내게 하는 작업이다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/paint/replay_order.rs` | `PaintReplayPlane`에 `Serialize` 추가 |
| `src/renderer/canvaskit_policy.rs` | `CanvasKitReplayItem.replayPlane` 추가 |
| `tests/issue_1017.rs` | `복학원서.hwp` replay plan fixture 테스트 추가 |

## 3. 구현 내용

`CanvasKitReplayItem`에 다음 필드를 추가했다.

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub replay_plane: Option<PaintReplayPlane>
```

PaintOp 기반 item은 `paint_op_replay_plane(op)` 결과를 `replayPlane`으로 직렬화한다.

clip/cache hint item은 특정 paint op가 아니라 tree context이므로 `replayPlane`을 생략한다.

직렬화 값:

| PaintReplayPlane | JSON |
|------------------|------|
| `Background` | `"background"` |
| `BehindText` | `"behindText"` |
| `Flow` | `"flow"` |
| `InFrontOfText` | `"inFrontOfText"` |

## 4. 회귀 테스트

추가/갱신 테스트:

| 테스트 | 검증 |
|--------|------|
| `replay_plan_items_expose_paint_replay_planes` | page background / behind image / text / front image item의 plane |
| `replay_plan_serializes_mode_and_summary` | replay plan JSON에 `"replayPlane":"flow"` 직렬화 |
| `issue_1017_canvaskit_replay_plan_exposes_baked_watermark_plane` | `복학원서.hwp` baked watermark item이 `replayPlane="behindText"` |

`issue_1017` fixture 테스트는 함께 다음을 확인한다.

- page background item: `background`
- 일반 textRun item: `flow`
- 중앙 baked watermark image item: `behindText`
- baked watermark detail에 `wrap=behindText` 유지

## 5. 검증

실행:

```text
cargo fmt --check
cargo test --lib canvaskit_policy
cargo test --test issue_1017
cargo test --lib paint::replay_order
```

결과:

```text
cargo test --lib canvaskit_policy
13 passed

cargo test --test issue_1017
2 passed

cargo test --lib paint::replay_order
5 passed
```

비고:

- 기존 코드의 warning 6개가 출력되었으나 Stage 5 변경과 무관하다.

## 6. 다음 단계

Stage 6에서는 통합 검증과 최종 보고서를 작성한다.

Stage 6 진행 전 작업지시자 승인이 필요하다.
