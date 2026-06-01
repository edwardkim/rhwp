# 최종 보고서 — Task M100-1167: SVG 렌더러 BehindText 워터마크 z-order

## 1. 이슈

#1167 (PR #1163 / #1017 후속). SVG 출력에서 `samples/복학원서.hwp` 중앙 baked watermark(`wrap=BehindText`)가 본문 텍스트를 덮음. PNG(native Skia)/웹캔버스(CanvasKit)는 PR #1163 의 PaintOp replay plane 으로 정정됐으나 SVG 만 누락.

## 2. 근본 원인

`src/renderer/svg.rs` 의 `render_node` 가 RenderNode 트리를 **단순 DFS 순회**하여, 문단 순서대로 트리에 push 된 워터마크 ImageNode 가 본문 TextRun 노드 뒤(SVG 후순위=위)에 그려짐. PaintOp replay plane 같은 z-order 재정렬 부재.

## 3. 정정 (SVG plane multi-pass)

### `src/renderer/layout.rs:4642`
ImageNode 생성 시 `text_wrap: Some(pic.common.text_wrap)` 채움 (종전 None). plane 판별 입력. PaintOp 경로는 별도로 image.text_wrap 을 set 하므로 독립 — 무회귀 확인.

### `src/renderer/svg.rs`
- `node_z_plane()`: `PageBackground(0) → BehindText 그림(1) → Flow(2) → InFrontOfText 그림(3)`. PaintOp `paint_op_replay_plane()` 과 동일 의미.
- `children_need_plane_reorder()`: plane 섞임 시에만 안정 정렬 (Flow-only 비용 회피, 같은 plane 내 순서 보존).
- `render_node` 자식 순회를 plane 순서로.

## 4. 1차 회귀 발견·정정 (작업지시자 시각 판정 게이트)

초기 구현은 `node_z_plane` 에 PageBackground 분류가 없어, root 레벨에서 BehindText 워터마크(plane 1)가 흰 배경 PageBackground(plane 2 오분류)보다 먼저 정렬 → **흰 배경 rect 가 워터마크를 덮어 워터마크 안 보임**.

- 시각 판정: "워터마크가 보이지 않습니다" → `PageBackground → plane 0` 추가 정정 → "이제 보입니다 / actual.svg 정답" 확정.

## 5. 검증

| 항목 | 결과 |
|------|------|
| issue_1167 회귀 가드 | ✅ BehindText `<image>` < 본문 첫 `<text>` |
| svg_snapshot (8) | ✅ issue_677 골든 정답 상태로 갱신 |
| `cargo test --tests` 전수 | ✅ 실패 없음 |
| native-skia skia | ✅ 32 passed (PNG 무회귀) |
| fmt / clippy --lib | ✅ clean |
| **복학원서.hwp 시각** | ✅ 워터마크 본문 뒤 + 가독 (작업지시자) |
| **143E433F503322BD33.hwp 시각** | ✅ 배경 워터마크 opacity 0.26 유지 + 본문/차트 정상 (#1156 _v2 무회귀, 작업지시자) |

### 두 워터마크 유형 동시 정합
- 복학원서: **BehindText 본문 그림** 워터마크 → plane 1 로 본문 뒤
- 143E: **배경 채우기(borderFill imgBrush)** 워터마크 → plane 0(PageBackground) opacity 0.26 유지

## 6. 변경 파일

- `src/renderer/layout.rs` — ImageNode.text_wrap 채움
- `src/renderer/svg.rs` — node_z_plane / children_need_plane_reorder / plane multi-pass 순회
- `tests/issue_1167_svg_behindtext_zorder.rs` (신규) — 회귀 가드
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg` — 정답 상태로 갱신

## 7. 메모리 룰 정합

- `feedback_image_renderer_paths_separate` — svg 가 PaintOp replay plane(skia/canvaskit)과 별도 경로임을 확인하고 SVG 단독 정정. layout text_wrap 공통 변경이 타 경로 무회귀 검증.
- `feedback_visual_regression_grows` — 정량(image/text 줄 순서) + 작업지시자 시각 판정 병행. 1차 회귀(흰배경 덮음)는 정량만으론 못 잡고 시각 판정이 게이트.
- `feedback_rhwp_visual_authority` — 두 fixture 시각 판정으로 plane 정합 확정.

## 8. scope 밖

- InFrontOfText(직인) plane 3 분류는 구현했으나 본 두 fixture 에 직인 케이스 없음 — 향후 직인 샘플로 추가 검증 가능.
