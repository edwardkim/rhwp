# Task M100-1156 최종 보고서 — 2단 문서 차트 컨트롤 단 이동 + 자리차지 텍스트 + 워터마크 정합

- 이슈: [#1156](https://github.com/edwardkim/rhwp/issues/1156)
- 마일스톤: v1.0.0 (M100), enhancement
- 브랜치: `local/task1156`
- 작성일: 2026-05-29

## 1. 본질

2단(multicolumn) 문서에서 차트(OLE) 컨트롤이 한컴 에디터와 다르게 배치되는 결함. 작업지시자 추가 요구로 (a) 차트 단 이동 (b) 빈 공간 텍스트 back-fill (c) 자리차지 겹침 방지 + 진단 중 발견된 (d) 워터마크 효과 회귀까지 정합.

## 2. 재현 fixture

`samples/hwpx/143E433F503322BD33.hwpx` (+ HWP5 동일본):
- 2단(colCount=2) + 차트 OLE(80mm, MS Graph, wrap=TopAndBottom) + 표 + 배경 워터마크
- 정답지: `pdf-large/hwpx/143E433F503322BD33.pdf` (한컴 출력)

## 3. 차트 크기 확정 (선결)

한컴 + spec(SHAPE_COMPONENT 공통 크기) + HWPX `hp:sz`(22677 HU) = **80mm 3중 일치**. OLE curSz/orgSz(0/7200) 아님. `resolve_object_size`(common) 점유 정상. 차트 그림(MS Graph OLE2) 자체 렌더는 placeholder 80mm — scope 밖(승인).

## 4. 근본 원인 + 정정

### 4.1 차트 단 이동 (Stage 2) — `src/renderer/typeset.rs`

- 실제 pagination 엔진 = TypesetEngine(typeset.rs), engine.rs 는 fallback (`rendering.rs:1991`)
- `typeset_table_paragraph` Shape 분기가 **TAC 객체만** 단 이동/높이 처리 → 비-TAC 차트는 높이 0 push
- 정정: 비-TAC TopAndBottom vert=Para Picture/Shape(차트)에 `non_tac_pushdown_h` (common 높이+margin) + 단 잔여 부족 시 `advance_column_or_new_page`

### 4.2 자리차지 텍스트 겹침 (Stage 3) — `src/renderer/layout.rs`

- `layout_shape_item` 의 `if common.treat_as_char` 블록 밖이라 비-TAC 차트는 `result_y` 미진행 → 후속 텍스트가 차트 영역 겹침
- 정정: 비-TAC TopAndBottom vert=Para Shape 에 else 분기 추가 — `result_y = y_offset + 차트높이 + margin_bottom` (후속 텍스트를 차트 아래로)

### 4.3 워터마크 효과 회귀 (Stage 3, scope 확대) — svg.rs / web_canvas.rs / skia/renderer.rs

- PR #1019(#975 RealPic) 가 RealPic 톤 프리셋 경로 추가하며, effect=RealPic 이지만 톤 프리셋(brightness=-50,contrast=70)이 아닌 배경 워터마크(brightness=contrast=0)가 `is_real_picture_watermark_tone_preset`/`is_watermark_image` 둘 다 false → opacity 누락 (배경 워터마크 불투명 회귀)
- 정정: 페이지 배경 이미지는 본질적으로 워터마크 — RealPic 톤 프리셋 아닌 PageBackground 이미지도 워터마크 opacity 적용 (3 렌더러 경로 정합, 메모리 룰 `feedback_image_renderer_paths_separate`)
  - svg.rs `render_page_background_image`, web_canvas.rs PageBackground, skia/renderer.rs `PaintOp::PageBackground` (`save_layer_alpha`)

## 5. 검증

### 정량 (dump-pages)

| | 정정 전 | 정정 후 |
|--|---------|---------|
| 단1 diff | -396px | -15.5px |
| 차트 | 단0 (표 겹침) | 단1 상단 |
| 단1 텍스트 첫 줄 y | 154 (차트 113~415 겹침) | 486 (차트 아래) |

### 시각 (PNG export, 폰트 폴더 적용) — 작업지시자 통과

- 차트 단1 상단 배치 ✅ (정답지 일치)
- 자리차지 텍스트 차트 아래 ✅
- SVG/PNG 배경 워터마크 반투명 효과 복원 ✅

### 회귀 가드

- `tests/issue_1156_chart_column_flow.rs` — (1) 차트 단1 상단 + 텍스트 비겹침 (2) 배경 워터마크 opacity → 2 passed
- `cargo test --release --test svg_snapshot` → 8 passed (다단 골든 불변)
- `cargo test --release --tests` → 회귀 없음
- `cargo clippy --lib --release -- -D warnings` → 0 warnings
- `cargo fmt --all -- --check` → 정합

## 6. 변경 파일

- `src/renderer/typeset.rs` — 차트 단 이동
- `src/renderer/layout.rs` — 자리차지 텍스트 겹침 방지
- `src/renderer/svg.rs` / `web_canvas.rs` / `skia/renderer.rs` — 배경 워터마크 opacity (3 경로)
- `tests/issue_1156_chart_column_flow.rs` — 회귀 가드 (신규)
- fixture: `samples/hwpx/143E433F503322BD33.hwpx`, `pdf-large/hwpx/143E433F503322BD33.pdf`
- 문서: plans/working/tech/report

## 7. 메모리 룰 정합

- `feedback_image_renderer_paths_separate` — 워터마크 3 렌더러 경로 동시 정정
- `feedback_pdf_not_authoritative` — 한컴 PDF 정답지 시각 판정
- `feedback_hancom_compat_specific_over_general` — 비-TAC TopAndBottom + vert=Para case-specific 가드
- `feedback_visual_regression_grows` — dump-pages 정량 + PNG 시각 병행
- `feedback_push_full_test_required` — cargo test --tests + fmt

## 8. scope 밖 (후속)

- 차트 그림(MS Graph OLE2) 실제 그래프 렌더링 (현재 placeholder 80mm) — 별개 enhancement
