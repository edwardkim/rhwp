# Task M100 #1017 Stage 3 보고서 — native Skia replay plane 적용

## 1. 범위

Stage 3에서는 native Skia PageLayerTree direct replay에 Stage 2에서 확정한 replay plane 순서를 적용했다.

적용 순서:

```text
background -> behindText -> flow -> inFrontOfText
```

CanvasKit direct renderer와 replay plan 진단은 아직 변경하지 않았다.

## 2. 변경 파일

| 파일 | 내용 |
|------|------|
| `src/renderer/skia/renderer.rs` | root를 `PaintReplayPlane::ORDERED` 순서로 multi-pass 순회 |
| `src/renderer/skia/renderer.rs` | leaf op를 현재 plane과 맞을 때만 replay |
| `src/renderer/skia/renderer.rs` | BehindText / InFrontOfText z-order 픽셀 테스트 2개 추가 |

## 3. 구현 내용

`SkiaLayerRenderer::render_raster_with_options()`에서 기존 단일 root 순회를 제거하고, 다음 순서로 `render_node()`를 반복 호출하도록 변경했다.

```text
PaintReplayPlane::Background
PaintReplayPlane::BehindText
PaintReplayPlane::Flow
PaintReplayPlane::InFrontOfText
```

`render_node()`에는 현재 `replay_plane` 인자를 추가했다.

leaf 처리에서는 두 곳에서 `paint_op_replay_plane(op) != replay_plane`인 op를 건너뛴다.

- glyph variant 후보 수집
- 실제 op replay

따라서 non-flow pass에서는 `TextRun`이 처리되지 않고, `next_text_source_id`도 증가하지 않는다. 기존 text source id 순서는 Flow pass에서만 유지된다.

## 4. 회귀 테스트

추가 테스트:

| 테스트 | 검증 |
|--------|------|
| `behind_text_image_replays_below_flow_across_tree_branches` | raw tree order에서 flow rect 뒤에 있는 `BehindText` image가 최종 픽셀에서는 flow 아래에 있음 |
| `in_front_of_text_image_replays_above_flow_when_raw_order_is_earlier` | raw op order에서 flow rect 앞에 있는 `InFrontOfText` image가 최종 픽셀에서는 flow 위에 있음 |

첫 테스트는 #1017의 `root/g3` 중앙 watermark처럼 서로 다른 tree branch에 있는 behind image를 겨냥한다.

## 5. 검증

실행:

```text
cargo fmt --check
cargo test --features native-skia skia --lib
cargo test --lib paint::replay_order
cargo test --test issue_1017
cargo run --features native-skia --bin rhwp -- export-png samples/복학원서.hwp -p 0 -o output/task1017-stage3
```

결과:

```text
cargo test --features native-skia skia --lib
32 passed

cargo test --lib paint::replay_order
5 passed

cargo test --test issue_1017
1 passed

cargo run --features native-skia --bin rhwp -- export-png ...
output/task1017-stage3/복학원서.png (198675 bytes)
```

비고:

- 최초 `cargo test --features native-skia skia --lib`와 `cargo run --features native-skia ...`는 sandbox 네트워크 제한으로 Skia binary 다운로드에 실패했다.
- 승인된 escalation으로 재실행해 통과했다.
- `export-png` 중 기존 layout overflow 진단 1건이 출력되었다.
  - `LAYOUT_OVERFLOW: page=0, sec=0, col=0, para=16, type=Shape, overflow=2.5px`
  - Stage 3 z-order 변경과 직접 관련 없는 기존 레이아웃 진단으로 판단했다.
- 기존 코드의 warning 6개가 출력되었으나 Stage 3 변경과 무관하다.

## 6. 다음 단계

Stage 4에서는 `rhwp-studio` CanvasKit direct renderer에 같은 replay plane 정책을 적용한다.

Stage 4 진행 전 작업지시자 승인이 필요하다.
