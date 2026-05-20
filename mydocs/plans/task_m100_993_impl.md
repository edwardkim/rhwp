# 구현계획서 — task993: 분할 표 페이지네이션 모델 교체 (줄 범위 기반)

- 타스크: 로컬 task993 / 브랜치 `local/task993` (`local/task992` 위)
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행: 수행계획서 `task_m100_993.md`

## 1. 설계 개요

### 1-1. 컷 디스크립터 — 콘텐츠 유닛 인덱스

분할 표의 분할 상태를 연속 px `content_offset`이 아니라 **셀별 "소비한 콘텐츠 유닛 수"** 로 표현한다.

- **콘텐츠 유닛(content unit)**: 한 셀 안의, 문서 순서대로 나열한 *합성 줄(composed line)* 또는 *중첩 표 atom* 1개.
- **컷(cut)**: 셀마다 "지금까지 소비한 유닛 수". 행 단위 컷 = `Vec<usize>`(셀 인덱스 → 소비 유닛 수). 한 셀 내부는 문단 경계를 유닛 평탄화로 흡수한다.

px가 사라지므로 vpos 리셋에 의한 비단조성이 원천 제거된다 — 컷은 항상 유닛을 ≥1개 더 소비하는 방향으로만 전진한다(무한 루프 불가). vpos 리셋은 px 오프셋으로 "건너뛰는" 대상이 아니라, 전진 walk의 **강제 정지점**으로만 작동한다.

### 1-2. 단일 권위 함수

```
fn advance_row_cut(table, row, start_cut, avail_height, styles)
    -> RowCutResult { end_cut, consumed_height, hit_hard_break, fully_consumed }
```

- `start_cut`: 이전 페이지까지 소비한 행 컷(`Vec<usize>`, 셀별).
- `avail_height`: 이 페이지에서 행에 줄 수 있는 콘텐츠 높이.
- 동작: 각 셀에서 `start_cut`부터 유닛을 누적하다 `avail_height` 도달 또는 **vpos 리셋(hard break)** 에서 정지. 셀들이 공통 높이에서 잘리도록 행 차원에서 정합. 줄은 통째로·중첩 표는 atomic.
- 페이지네이터(분할 판정)와 렌더러(가시 범위)가 **모두 이 함수**를 호출 → 컷이 정의상 일치.

`compute_cell_line_ranges` / `cell_line_prefix_counts` / `calc_visible_content_height_from_ranges`의 px·vpos 3중 누적 로직을 이 함수의 단일 유닛 누적으로 대체·흡수한다.

### 1-3. PartialTable 스키마

`PageItem::PartialTable`의 `split_start_content_offset: f64` / `split_end_content_limit: f64`(px)를 **`start_cut` / `end_cut`(행 컷)** 으로 교체. `start_row`/`end_row`/`is_continuation`은 유지.

## 2. 단계 구성

### Stage 1 — 설계 확정 + 조사

- 컷 디스크립터 구체 타입 확정(`Vec<usize>` vs per-(cell,para) 등), 유닛 평탄화 규칙 확정.
- `PartialTable` 생산자(`typeset.rs typeset_block_table`)·소비자(`layout.rs`, `table_partial.rs layout_partial_table`) 전 호출부 목록화.
- `compute_cell_line_ranges`·`cell_line_prefix_counts`·`calc_visible_content_height_from_ranges(_with_offset)`·`content_y_accum`·`find_break_row`·`effective_row_height`·`remaining_content_for_row` 의존 관계도 작성.
- vpos 리셋 검출 규칙(`[Task #697]`)·중첩 표 atomic·머리행 반복·rowspan 블록과의 상호작용 정리.
- 단계보고서에 설계 확정안 기록 → 승인.

### Stage 2 — 단일 컷 함수 구현

- `advance_row_cut`(및 보조: 유닛 평탄화, 유닛 높이, vpos 리셋 검출)를 `table_layout.rs`에 구현. `LayoutEngine` 메서드.
- 컷 디스크립터 타입 정의.
- 단위 테스트: 일반 텍스트 셀, 중첩 표 셀, vpos 리셋 셀, rowspan, 머리행 반복 — 컷 전진·정지·완료 판정.
- `cargo test`(해당 모듈) + `cargo clippy`.

### Stage 3 — 렌더러 정합

- `layout_partial_table`이 `start_cut`/`end_cut`(행 컷)으로 가시 범위를 렌더. `content_y_accum` 기반 중첩 표 재판정 제거.
- `calc_visible_content_height_from_ranges*`를 `advance_row_cut` 기준으로 재배선하거나 컷 기반 높이 계산으로 대체.
- 분할 행 높이(`layout_partial_table` 2b단계) 컷 기준으로 정정.
- 골든 SVG 영향 점검(이 단계는 PartialTable 생산이 아직 px라면 어댑터 경유 — Stage 4와 함께 정합).

### Stage 4 — 페이지네이터 교체

- `typeset_block_table` 분할 루프를 `content_offset`(px) → 행 컷으로 교체. 분할 판정에 `advance_row_cut` 사용.
- `PageItem::PartialTable` 스키마를 `start_cut`/`end_cut`으로 변경 — `typeset.rs`·`layout.rs`·관련 소비부 동시 갱신.
- `find_break_row`/`effective_row_height`/`remaining_content_for_row`의 분할 행 px 경로 정리(행 경계 분할은 유지).
- 분할 표 export가 무한 루프 없이 종료됨을 확인.

### Stage 5 — 전체 검증 + 최종 보고서

- 비공개 샘플 `export-svg` 후 전 페이지 `<text>` y > viewBox 높이 스캔 → **오버플로 0**.
- 분할 표 샘플 export 정상 종료(무한 루프 0).
- `dump-pages`로 분할 표 연속분 측정 = 렌더 일치 확인.
- `cargo test --release` 전체 통과 + `cargo clippy --release` 무경고.
- 골든 SVG 회귀 — 이동 페이지를 한컴 2022 PDF와 대조해 정정/회귀 판정. 회귀면 교정, 정정이면 골든 갱신.
- 공개 분할 표 샘플 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(페이지네이터·렌더러) — 릴리즈 시 Docker 재빌드 명시.
- `report/task_m100_993_report.md` 작성.

## 3. 영향 범위 / 리스크

- 수정 파일: `src/renderer/typeset.rs`, `src/renderer/layout.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/layout/table_layout.rs`, `src/renderer/render_tree.rs`(PartialTable 정의 위치 따라).
- `PartialTable` 스키마 변경은 생산·소비부를 동시에 바꿔야 하며 부분 적용 시 빌드 불가 — Stage 3·4를 묶어 일관 상태 유지(중간 어댑터 허용).
- 분할 표 문서의 골든 SVG가 광범위 이동 → Stage 5에서 페이지별 PDF 대조 필수.
- 정확한 분할로 페이지 수가 늘 수 있다(오버플로 해소의 정상 귀결, 수행계획서 §6).
- 비-분할 표·인라인(TAC) 표 경로는 건드리지 않는다 — 변경을 `PartialTable`(블록 분할 표) 경로로 한정.
- 제외: HWP3, 파서, 폰트 메트릭 한컴 불일치, 페이지 수 누적 드리프트.

## 4. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. 회귀 테스트는 공개 골든 SVG·공개 샘플 우선, 불가 시 비커밋 처리 후 보고서에 명시. `orders/` 파일은 생성·수정하지 않는다.
