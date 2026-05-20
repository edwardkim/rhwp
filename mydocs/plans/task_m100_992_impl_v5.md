# 구현계획서 v5 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행: 수행계획서, 구현계획서 v1~v4, 단계보고서 stage1~5.

## 0. v5 개정 사유

v3·v4(페이지네이터 측 측정 통일)가 실패한 이유를 Stage 5에서 확정했다: 결함은 페이지네이터가 아니라 **렌더러 `table_partial`의 분할 셀 렌더 내부 불일치**다.

`table_partial`은 분할 셀에서 콘텐츠 종류별로 다른 가시성 메커니즘을 쓴다:
- 텍스트 줄 — `compute_cell_line_ranges`의 줄 범위(`start_line < end_line`)로 판정.
- 중첩 표 — `compute_cell_line_ranges`가 "스킵"(`start >= end`)으로 판정해도, `table_partial.rs`(약 690~708행)가 `content_y_accum`(텍스트 누적 위치) 기반으로 **재판정**해, `content_y_accum + nested_h > split_start_content_offset`이면 fall-through하여 그 중첩 표를 그린다.

`content_y_accum`은 중첩 표 호스트 문단을 작은 앵커 줄 높이로만 전진시켜(중첩 표 높이 미반영) `compute_cell_line_ranges`의 누적과 어긋난다. 결과적으로 `table_partial`은 줄 범위상 이전 페이지 소속인 중첩 표를 연속분 페이지에 **중복 렌더**한다 → 실제 렌더가 렌더러 자신의 줄 범위 함수보다 커져 본문 밖으로 넘침(pi=308: 중첩 표 11개, ~470px 초과).

## 1. 목표

`table_partial`의 분할 셀 렌더에서 **중첩 표 가시성을 `compute_cell_line_ranges`의 줄 범위로 통일**한다(텍스트 줄과 동일 기준). 그러면 `table_partial` 실제 렌더 = `compute_cell_line_ranges` = `calc_visible_content_height_from_ranges`가 정의상 일치하고, 페이지네이터가 같은 함수로 분할 표 연속분을 측정하면(v4 헬퍼) 분할 판정이 렌더와 일치 → 오버플로 0.

## 2. 설계

### 2-a. 렌더러 정정 (`table_partial.rs`)

분할 셀 루프에서 `start_line >= end_line`(= `compute_cell_line_ranges`가 스킵 판정)인 **중첩 표 포함 문단**은, 현재의 `content_y_accum` 재판정을 제거하고 **무조건 스킵**한다(`content_y_accum`을 중첩 표 높이만큼 전진 후 `continue`).

근거: `compute_cell_line_ranges`는 중첩 표를 atomic으로 처리해, 연속분 페이지에서 보여야 하는(경계를 걸치거나 offset 이후인) 중첩 표는 `start < end`(visible)로 표시한다. `start >= end`는 "이전 페이지에서 이미 렌더됨"이 확정된 경우뿐이므로, 스킵이 정답이다. fall-through 렌더는 중복이다.

### 2-b. 페이지네이터 정합 (`typeset.rs`)

v4 §2의 헬퍼 `split_row_continuation_height`(`compute_cell_line_ranges` + `calc_visible_content_height_from_ranges`)를 재적용해, 분할 표 연속분(`content_offset > 0`)의 측정을 렌더러 줄 범위와 일치시킨다. 2-a로 렌더러 실제 렌더가 줄 범위와 일치하므로, 헬퍼 측정 = 렌더 실제가 성립한다.

### 정합 사슬

`compute_cell_line_ranges`(줄 범위) → `table_partial` 렌더(2-a로 줄 범위 준수) = `calc_visible_content_height_from_ranges`(줄 범위 높이) = 페이지네이터 헬퍼(2-b) → 분할 판정이 렌더와 정의상 일치 → 오버플로 0.

## 3. 단계 구성

### Stage 1~5 — 완료

조사 / 페이지 143 시도·진단 / 페이지 171 데코레이션 수정(`a4c7e823`) / v3 시도·진단 / v4 시도·진단.

### Stage 6 — 렌더러·페이지네이터 줄 범위 통일 + 전체 검증 + 최종 보고서

- §2-a `table_partial.rs` 정정 + §2-b `typeset.rs` 헬퍼 재적용.
- 비공개 샘플 184쪽 `export-svg` → 전 페이지 `<text>` y > viewBox 높이 스캔 → **오버플로 0**(현재 3건).
- **콘텐츠 보존 검증**: pi=308 분할 표가 페이지 142·143·144에 걸쳐 중첩 표·문단을 중복·누락 없이 한 번씩 렌더하는지 대조.
- `cargo test --release` 전체 + `cargo clippy --release` 무경고. 골든 SVG 회귀 시 변경 페이지를 한컴 2022 PDF와 대조해 정정/회귀 판정.
- 공개 분할 표 샘플 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(렌더러·페이지네이터) — 릴리즈 시 Docker 재빌드 명시.
- 단계보고서 + `report/task_m100_992_report.md`.

## 4. 리스크

- `table_partial.rs`는 렌더러의 최복잡 파일이며 분할 셀 중첩 표 렌더는 intra-nested 분할·`content_y_accum` 위치 계산과 얽혀 있다. 변경을 **`start_line >= end_line` 스킵 분기로 한정**해 가시 중첩 표 렌더 경로는 건드리지 않는다.
- `content_y_accum`은 가시 중첩 표 위치 계산에 계속 쓰이므로 제거하지 않는다.
- 페이지 수가 늘 수 있다(중복 렌더 제거 + 정확한 분할). 골든·페이지 수 회귀를 PDF로 판정.
- 제외(수행계획서 §4): HWP3, 파서, 폰트 메트릭 한컴 불일치, 페이지 수 누적 드리프트.

## 5. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. 회귀 테스트는 공개 골든 SVG 우선. `orders/` 파일은 생성·수정하지 않는다.
