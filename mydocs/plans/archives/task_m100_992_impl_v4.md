# 구현계획서 v4 — task992: 페이지 영역 밖 콘텐츠 제거

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 선행: 수행계획서 `task_m100_992.md`, 구현계획서 v1~v3, 단계보고서 stage1~4.

## 0. v4 개정 사유

구현계획서 v3(측정 *높이* 경로 통일)는 페이지네이터의 셀 높이 측정 함수를 렌더러 `calc_cell_remaining_content_height`로 통일했으나, 오버플로 3건(123·144·176)이 해소되지 않았다(Stage 4 보고서).

추가 진단으로 결함 지점을 재확정했다: 결함은 *높이 추정값*이 아니라 **분할 위치(줄 범위) 결정**의 불일치다.

- 페이지네이터: 분할 표 연속분을 `total − content_offset`의 **연속(continuous)** 차감으로 측정.
- 렌더러: `compute_cell_line_ranges`/`cell_line_prefix_counts`로 **이산(discrete)** 분할 — 줄은 통째로, 중첩 표는 **원자적(atomic)** 으로 한쪽 페이지에만 배치.

`content_offset`(px 예산)에 맞춰 렌더러가 prefix를 이산 스킵하면 스킵량 ≤ `content_offset`이 되어, 렌더러의 연속분 = `total − 스킵량` ≥ `total − content_offset`(페이지네이터 추정값)이 된다. 즉 페이지네이터가 항상 과소 측정 → 연속분을 추가 분할 안 함 → 렌더러가 본문 밖까지 그림.

## 1. 목표

페이지네이터가 분할 표 **연속분(continuation)**의 높이를, 렌더러와 **동일한 이산 줄 범위 함수**로 측정한다. 페이지네이터의 "이 연속분이 페이지에 들어가는가" 판정이 렌더러의 실제 렌더와 정의상 일치 → 페이지 경계 초과 시 반드시 추가 분할 → 오버플로 0.

## 2. 설계

렌더러 `LayoutEngine`에 이미 구현된 함수:

- `compute_cell_line_ranges(cell, composed_paras, content_offset, content_limit, styles)` — content_offset 이후, content_limit 이내의 가시 줄 범위. `cell_line_prefix_counts` 기반 이산 스냅.
- `calc_visible_content_height_from_ranges_with_offset(composed, paras, line_ranges, styles, content_offset)` — 그 줄 범위의 실제 렌더 높이(중첩 표 포함).

이 두 함수가 렌더러 `table_partial`의 실제 분할 렌더와 같은 기준이다.

### 변경

1. `TypesetEngine`이 `LayoutEngine`(dpi) 보유.
2. 헬퍼 추가 — `split_row_continuation_height(table, row, content_offset, styles) -> f64`: 행의 각 셀 문단을 compose(+`recompose_for_cell_width`)하고 `compute_cell_line_ranges(cell, .., content_offset, 0.0)` → `calc_visible_content_height_from_ranges_with_offset` 로 연속분 실제 높이를 구해 셀별 최댓값을 반환.
3. `typeset_block_table`의 분할 루프에서, **연속분(`content_offset > 0`)** 측정을 헬퍼로 교체:
   - 빈 행 스킵 판정(`remaining_content_for_row(cursor_row, content_offset) <= 0`),
   - `effective_first_row_h`(= `effective_row_height(cursor_row, content_offset)`),
   - 연속분 인트라-로우 재분할 시 `remaining_content_for_row(r, content_offset)`.
   첫 fragment(`content_offset == 0`)는 렌더러가 `content_limit` 예산 내로 이산 스냅하므로 그 페이지를 넘지 않는다 — 종전 `MeasuredTable` 경로 유지(회귀 최소화).
4. 패딩은 기존 `max_padding_for_row` 유지(헬퍼는 콘텐츠 높이만).

### 정합 근거

연속분 측정이 렌더러 `compute_cell_line_ranges` 결과의 실제 높이와 같아지므로, 페이지네이터가 "연속분이 안 들어감"을 정확히 판정해 추가 분할(`split_end_limit` 설정)한다. 추가 분할된 fragment는 `content_limit`로 렌더러가 이산 스냅 → 페이지 내. 남은 연속분은 다시 헬퍼로 정확히 측정 → 재귀적으로 오버플로 0.

## 3. 단계 구성

### Stage 1~4 — 완료

조사 / 페이지 143 측정 시도·진단 / 페이지 171 데코레이션 수정(`a4c7e823`) / v3 측정 통일 시도·진단.

### Stage 5 — 분할 위치 결정 통일 + 전체 검증 + 최종 보고서

- §2 변경 적용(`typeset.rs` 중심, `LayoutEngine` 함수 가시성 조정 필요 시 `pub(crate)`).
- 비공개 샘플 184쪽 `export-svg` → 전 페이지 `<text>` y > viewBox 높이 스캔 → **오버플로 0**(현재 3건).
- `dump-pages`로 pi=308·108·111 연속분 측정이 렌더와 일치하는지 확인.
- `cargo test --release` 전체 + `cargo clippy --release` 무경고. 골든 SVG 회귀 시 각 변경 페이지를 한컴 2022 PDF와 대조해 정정/회귀 판정.
- 공개 분할 표 샘플 교차 회귀 + 페이지 수 회귀 확인.
- WASM 영향(페이지네이터·렌더러) — 릴리즈 시 Docker 재빌드 명시.
- 단계보고서 + `report/task_m100_992_report.md`.

## 4. 리스크

- `typeset_block_table` 분할 루프는 페이지네이터의 복잡 영역이다. 변경을 **연속분(`content_offset > 0`) 경로로 한정**해 첫 fragment 경로 회귀를 피한다.
- 연속분 측정이 정확해지면 분할이 더 보수적이 되어 분할 표 문서의 페이지 수가 늘 수 있다 → 골든 SVG·페이지 수 회귀를 PDF 권위 자료로 판정.
- 헬퍼가 셀 문단을 compose 하므로 측정 비용 증가 — 분할 표 연속분에 한정되어 영향은 작다.
- 제외(수행계획서 §4): HWP3, 파서, 폰트 메트릭 자체의 한컴 불일치, 페이지 수 누적 드리프트.

## 5. 비공개 문서 처리

재현용 HWPX/PDF는 커밋하지 않는다. 회귀 테스트는 공개 골든 SVG 우선. `orders/` 파일은 생성·수정하지 않는다.
