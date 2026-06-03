# Stage 5 보고서 — task992: 분할 위치 결정 통일 시도 + 최종 진단

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 단계: Stage 5 — 구현계획서 v4 §2(분할 위치 결정 통일)
- 결과: **v4 방식 시도 → 목표 미달로 되돌림. 진짜 결함이 렌더러 내부에 있음을 확정.**

## 1. 시도한 수정 (구현계획서 v4 §2)

`TypesetEngine`이 렌더러 `LayoutEngine`을 보유하고, 분할 표 **연속분(`content_offset > 0`)** 측정을 헬퍼 `split_row_continuation_height`로 통일했다 — 헬퍼는 렌더러의 이산 줄 범위 함수 `compute_cell_line_ranges` + `calc_visible_content_height_from_ranges_with_offset`로 연속분 높이를 잰다.

## 2. 결과 — 목표 미달

- SVG 출력 10쪽이 바뀌었으나 **오버플로 3건(123·144·176) 그대로**. → 시도 코드 되돌림.
- 계측(`RHWP_DEBUG_SPLIT`): 페이지 144(pi=308) 연속분(content_offset=1642.5)을 헬퍼는 **≈546px**로 측정. 그러나 렌더러 `table_partial`은 같은 연속분을 **≈1018px** 그린다.

## 3. 최종 진단 — 결함은 렌더러 `table_partial` 내부 불일치

헬퍼는 렌더러의 *공식* 분할 함수 `compute_cell_line_ranges`/`calc_visible_content_height_from_ranges`를 그대로 쓴다. 그런데도 렌더러의 **실제 렌더(`table_partial`)와 어긋난다**.

원인: `table_partial`의 분할 셀 렌더가 **콘텐츠 종류별로 다른 가시성 메커니즘**을 쓴다.

- 텍스트 줄: `compute_cell_line_ranges`의 줄 범위로 가시성 판정.
- 중첩 표: 줄 범위가 아니라 `content_y_accum`(텍스트 누적 위치) + `calc_nested_table_height`로 별도 판정(`table_partial.rs`의 분할 행 중첩 표 분기).

두 메커니즘이 어긋나, `table_partial`은 `compute_cell_line_ranges`가 "스킵됨"으로 본 영역의 중첩 표를 실제로는 그린다 → 실제 렌더가 렌더러 자신의 추정 함수(`calc_visible_content_height_from_ranges`)보다 커진다. pi=308은 중첩 표 11개라 차이가 ~470px까지 누적된다.

즉 페이지네이터를 렌더러의 추정 함수에 아무리 정확히 맞춰도(v3·v4) 오버플로가 안 잡힌다 — **추정 함수 자체가 렌더러의 실제 렌더와 불일치**하기 때문이다.

페이지 123·176(pi=108/111, 중첩 표 없는 대형 다행 표)도 분할 표 연속분/행 렌더에서 같은 계열의 `table_partial` 측정·렌더 불일치다.

## 4. 진짜 해결 방향

`table_partial`의 분할 셀 렌더에서 **중첩 표 가시성을 `compute_cell_line_ranges`의 줄 범위로 통일**해야 한다(현재의 `content_y_accum` 기반 별도 판정 제거). 그러면 `table_partial` 실제 렌더 = `calc_visible_content_height_from_ranges` = 페이지네이터 측정이 정의상 일치해 오버플로 0이 된다.

이는 페이지네이터가 아니라 **렌더러(`table_partial.rs`)의 분할 셀 렌더 로직 정비**다. v4 계획(페이지네이터 측 통일)의 전제가 어긋났으므로, 렌더러 측 정비를 별도 범위로 다뤄야 한다.

## 5. 검증

- 시도 코드 되돌림 후 `cargo build --release` 정상, `git diff` 무변경(Stage 3 커밋 `a4c7e823` 상태).

## 6. task992 도달 상태

- **Stage 3 — 완료·커밋**: 페이지 171(페이지보다 큰 부동 표 데코레이션 오분류) 수정. 오버플로 4건 → 3건. `cargo test` 1301 passed.
- **페이지 123·144·176 — 미해결**: 렌더러 `table_partial` 분할 셀 렌더의 콘텐츠 종류별 가시성 메커니즘 불일치. 페이지네이터 측 수정(v3·v4)으로는 불가, 렌더러 측 정비 필요.
