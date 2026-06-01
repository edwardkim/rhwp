# 구현계획서 — Task M100-1167: SVG BehindText 워터마크 z-order (3단계)

## 설계 요약

- **ImageNode 생성: layout.rs:4642 단일 지점** — `..ImageNode::new()` 로 `text_wrap=None`. `text_wrap: Some(pic.common.text_wrap)` 추가.
- **SVG 순회**: `render_node` 가 노드 자신 렌더 후 `for child in &node.children` 재귀. plane 순회는 **자식을 plane 순서(Background/BehindText → Flow → InFrontOfText)로 정렬**하여 재귀.
- plane 판별: `paint_op_replay_plane()` 과 동일 규칙을 RenderNode 용으로 — ImageNode.text_wrap 이 BehindText/InFrontOfText 인지로 분류. 그 외 노드(TextRun/Table/일반 그림)는 Flow.
- 참고: layout 은 이미 Task #347 에서 InFrontOfText 표 앞에 그림을 끼워넣는 부분 z-order 처리 보유 — plane 정렬이 이를 일반화하되 회귀 없도록 점검.

## Stage 1 — layout text_wrap 채움 + RED 회귀 테스트

**목표**: ImageNode 에 wrap 정보 보존 + 결함 박제(RED).

- `src/renderer/layout.rs:4642` ImageNode 에 `text_wrap: Some(pic.common.text_wrap)` 추가.
- `tests/issue_1167_svg_behindtext_zorder.rs` (신규):
  - `samples/복학원서.hwp` SVG 렌더 → BehindText `<image>` 출현 위치 < 본문 첫 `<text>` 위치 단언 (현재 실패 = RED).
  - 정량 helper: SVG 문자열에서 `<image` / `<text ` 출현 순서 파싱.
- 검증: 테스트가 **현재 실패**(RED)함을 확인 (결함 재현). `cargo build` + ImageNode.text_wrap 채움이 다른 렌더러(skia/canvaskit/json) 회귀 없는지 `cargo test --tests` 확인.
- 보고서: `mydocs/working/task_m100_1167_stage1.md`

## Stage 2 — svg.rs plane multi-pass 순회 (GREEN)

**목표**: SVG z-order plane 정합.

- `src/renderer/svg.rs`:
  - RenderNode → plane 분류 helper 추가 (ImageNode.text_wrap 기준; PaintReplayPlane 규칙 재사용 또는 동형 로직).
  - `render_node` 의 자식 순회를 plane 순서로 변경: Background/BehindText 이미지 먼저 → Flow(텍스트·표·어울림 그림) → InFrontOfText 이미지 마지막.
  - 안정 정렬로 같은 plane 내 기존 순서 보존 (Flow 내부 본문 순서 불변).
- 검증: Stage 1 테스트 GREEN. 복학원서 SVG 정량(`<image>` BehindText < 첫 `<text>`). `cargo test --tests` 회귀 없음.
- 산출물: `output/poc/issue_1167/복학원서.svg` (시각 확인용).
- 보고서: `mydocs/working/task_m100_1167_stage2.md`

## Stage 3 — 통합 검증 + 시각 판정 + 최종 보고서

**목표**: 전 경로 회귀 가드 + 작업지시자 시각 게이트.

- 검증: `cargo test --tests` 전수 + native-skia skia + issue_1017/1167/516 + `cargo fmt --check` + `cargo clippy --lib`.
- PNG/CanvasKit 무영향 재확인 (skia/canvaskit 테스트 유지, PaintOp 경로 image.text_wrap 별도).
- InFrontOfText(복학원서 (인) 서명) 도 plane 정합 확인.
- **작업지시자 한컴 정답지 시각 판정** (SVG 워터마크 본문 뒤, PNG 와 동일).
- 최종 보고서: `mydocs/report/task_m100_1167_report.md`.

## 단계별 커밋 규칙

각 stage 소스 + `_stage{N}.md` 보고서를 `local/task1167` 에서 함께 커밋. 기능/포맷 분리, 무관 rustfmt diff 금지.

## 리스크 점검 항목

- ImageNode.text_wrap 채움 → json.rs paint 변환이 RenderNode.text_wrap 을 읽는지(아니면 PaintOp 별도 경로인지) Stage 1 에서 확인. PaintOp image.text_wrap 은 paint 변환에서 별도 set(`replay_order.rs:58`) → layout ImageNode 변경과 독립 예상.
- Task #347 insert_pos z-order 처리와 plane 정렬 중복 → Flow plane 내부에서 기존 순서 보존되면 무해.
