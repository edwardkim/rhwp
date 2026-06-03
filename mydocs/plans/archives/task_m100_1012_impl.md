# Task #1012 구현 계획서

이슈: [#1012](https://github.com/edwardkim/rhwp/issues/1012)
수행 계획서: [`task_m100_1012.md`](task_m100_1012.md)

## Stage 1 진단 — Root cause 분리

`test-image.hwp` page 1 분석 결과 **두 가지 독립적인 결함** 식별:

### 결함 A — 텍스트 y 좌표 누락 (메인)

- pi=0 의 단일 line_seg: `ts=0, vpos=15180 HU (=202.4 px)`
- 의도: 텍스트 라인은 vpos=15180 위치 (Picture[2] TopAndBottom 의 bottom 이후) 에 배치
- 현재 SVG: 텍스트 y=143.6 (body_top=132.28 직후, Picture[2] 영역 내부 시작)
- → **line_seg.vpos 가 paragraph layout 의 첫 line y 계산에 반영되지 않음**

### 결함 B — z-order (보조)

- SVG 출력 순서: 모든 `<text>` 먼저, 모든 `<image>` 나중 → 모든 라벨이 image 뒤
- 특히 BehindText (글뒤로) Picture[4] 는 spec 상 image 가 text 뒤여야 하나 동일하게 image 가 위 → 잘못된 z-layer
- 결함 A 가 해결되어 텍스트가 picture 아래로 배치되면 visual overlap 자체가 사라져 z-order 영향 축소되지만, BehindText/InFrontOfText 의 본래 의미를 따르려면 별도 정정 필요

## Stage 2 구현 — 결함 A fix (line_seg.vpos 반영)

### 조사 대상

`src/renderer/layout/paragraph_layout.rs` 의 paragraph rendering pass 에서:
- paragraph 의 첫 line y 좌표 계산 로직
- `spacing_before` + `current_y` 누적과 별개로 `line_seg.vertical_pos` 를 합산하는지 확인
- TopAndBottom wrap control 의 wrap zone 처리가 line_seg vpos 와 일치하는지 확인

### 가설 + 검증 방법

가설 1: paragraph_layout 의 line y advance 가 `cumulative line_height` 만 사용 (line_seg.vpos 무시)
- 검증: 다른 sample 의 line_seg.vpos 활용 여부 grep
- fix: paragraph 첫 line 의 시작 y 를 `body_top + hwpunit_to_px(line_seg[0].vertical_pos)` 로 보정 (paragraph 안 wrap object 가 있을 때 한정)

가설 2: TopAndBottom Picture 의 wrap zone 이 텍스트 흐름에 반영 안됨
- 검증: TopAndBottom shape 의 wrap_around_paras 등록 확인
- fix: TopAndBottom Picture 의 bottom y 까지 텍스트 흐름 push-down

가설 3: paragraph spacing_before 0 + paragraph vpos start 0 으로 시작점이 비정상
- 검증: spacing_before 계산 trace
- fix: line_seg[0].vpos 를 spacing_before 로 흡수

Stage 2 진입 시 가설 1 부터 검증.

## Stage 3 구현 — 결함 B fix (z-order)

paragraph 내 controls 의 wrap 모드별 SVG/RenderTree 순서:
- **BehindText**: text 이전 push (image 뒤에 배치)
- **InFrontOfText**: text 이후 push (image 앞에 배치) — 기존 동작
- **TopAndBottom / Square / Tight**: 본문 흐름 외부, z 무관

### 변경 위치

`src/renderer/layout.rs::layout_shape_item` 또는 `paragraph_layout.rs` 의 control push 분기. BehindText control 을 `paper_images` 가 아닌 별도 `behind_images` vector 에 수집 후 paragraph rendering 시작 전 tree 에 먼저 push.

또는 더 단순한 접근: paragraph render 시점에 controls 를 wrap 모드별 정렬 후 push.

## Stage 4 검증

### 단위 검증

```bash
./target/release/rhwp export-svg samples/test-image.hwp -p 0 -o /tmp/test/
# 기대: 라벨 텍스트가 그림 아래 위치, 글뒤로 image 위에 라벨 표시
```

### 회귀 sweep

- `samples/hwp3-sample16.hwp` / `hwp3-sample16-hwp5.hwp` (다중 wrap shapes)
- `samples/exam_kor.hwp` (단순 본문)
- `samples/aift.hwp` / `samples/biz_plan.hwp`
- `samples/3-09월_교육_통합_2022.hwp` (시험지)
- `cargo test --release --lib`
- `cargo clippy --release -- -D warnings`

## Stage 5 최종 보고서 + PR

- 최종 결과 보고서 (`mydocs/report/task_m100_1012_report.md`)
- PR 생성 (`closes #1012`, base = devel)
- WASM 빌드

## 단계 요약

| Stage | 작업 | 산출물 |
|-------|------|--------|
| 1 | Root cause 진단 (완료) | 결함 A/B 분리 확인 |
| 2 | 결함 A fix — line_seg.vpos 반영 | `paragraph_layout.rs` 수정 |
| 3 | 결함 B fix — wrap 모드별 z-order | `layout.rs` 또는 별도 push |
| 4 | 회귀 검증 + 시각 판정 | sweep + 시각 판정 |
| 5 | 보고서 + PR | report + PR url |
