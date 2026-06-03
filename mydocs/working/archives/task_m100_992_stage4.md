# Stage 4 보고서 — task992: 측정 경로 통일 시도 + 추가 진단

- 타스크: 로컬 task992 / 브랜치 `local/task992`
- 마일스톤: M100 (v1.0.0)
- 작성일: 2026-05-19
- 단계: Stage 4 — 구현계획서 v3 §2(측정 경로 통일)
- 결과: **v3 방식 시도 → 목표 미달로 되돌림. 추가 진단으로 진짜 결함 지점 확정.**

## 1. 시도한 수정 (구현계획서 v3 §2)

`HeightMeasurer`가 렌더러 `LayoutEngine`을 보유하고, 셀 콘텐츠 높이를 렌더러 함수 `calc_cell_remaining_content_height`로 측정하도록 통일했다(`measure_table_impl`의 `content_height`·`MeasuredCell.total_content_height`, `remaining_content_for_row`를 `total − content_offset`으로). `calc_cell_remaining_content_height`에 `recompose_for_cell_width`도 추가했다.

## 2. 결과 — 목표 미달

- SVG 출력 72쪽이 바뀌었으나(측정값이 이동) **오버플로 3건(123·144·176)은 그대로**. 페이지 수도 184 불변.
- 즉 페이지네이터의 측정 함수를 렌더러의 `calc_*` 측정 함수로 바꿔도 분할 표 오버플로가 해소되지 않는다 → 시도 코드 되돌림.

## 3. 추가 진단 — 진짜 결함 지점

렌더러 계측(`table_partial`의 `para_y`/`content_y_accum`)으로 확인:

- 페이지 144(pi=308) 분할 셀: 페이지네이터는 연속분을 ≈778px로 측정, 렌더러는 **≈1018px**를 그린다(content_offset 1642.5 기준 셀 전체 ≈2660px).
- 이 240px 차이는 **`calc_cell_remaining_content_height`로도 예측 불가**다. 그 함수는 `(total − content_offset)`의 *연속* 차감이지만, 렌더러의 실제 분할(`compute_cell_line_ranges`)은 **이산(discrete)** 이다 — 줄은 통째로, 중첩 표는 **원자적(atomic)**으로 한쪽 페이지에만 배치된다.
- pi=308은 중첩 표 11개를 가진다. content_offset이 어떤 중첩 표의 중간에 떨어지면, `compute_cell_line_ranges`는 그 표를 통째로 다음 페이지로 보낸다 → 연속분이 `total − offset`보다 표 하나 높이만큼(최대 ~150px) 커진다. 페이지 123·176(대형 다행 표)도 줄 단위 이산 스냅으로 같은 현상이 난다.

**결론**: 결함은 *높이 추정값*의 불일치가 아니라, **분할 위치(줄 범위) 결정의 불일치**다. 페이지네이터는 높이 기반 연속 측정으로 content_offset을 정하고, 렌더러는 `compute_cell_line_ranges`/`cell_line_prefix_counts`로 이산 줄 범위를 정한다 — 두 결정이 어긋난다. 구현계획서 v3는 *측정 경로* 통일을 노렸으나, 통일해야 할 것은 **분할 위치 결정 경로**였다.

## 4. 진짜 해결 방향

페이지네이터가 분할 표의 컷 위치를 정할 때, 렌더러와 동일한 `cell_line_prefix_counts`(= `compute_cell_line_ranges`의 기반)로 "주어진 가용 높이에 들어가는 prefix 줄 수"를 산출해야 한다. 그러면 페이지네이터의 컷 = 렌더러의 컷이 정의상 일치한다.

이는 `typeset.rs`의 분할 표 페이지네이션 로직(`typeset_block_table`/`find_break_row` 등, 현재 `MeasuredTable` 높이 기반)을 `cell_line_prefix_counts` 기반으로 재배선하는 작업으로, v3보다 큰 변경이며 구현계획서 재개정(v4)이 필요하다.

## 5. 검증

- 시도 코드 되돌림 후 `cargo build --release` 정상, `git diff` 무변경(Stage 3 커밋 `a4c7e823` 상태).

## 6. 다음 단계 — 작업지시자 결정 요청

- 안 A: 구현계획서 v4 — 분할 위치 결정을 `cell_line_prefix_counts`로 통일(`typeset.rs` 분할 표 로직 재배선). 정공법이나 범위가 크고 페이지네이션 회귀 점검 부담이 큼.
- 안 B: 방어 가드 — 분할 표 연속분 측정에 "남은 콘텐츠 중 최대 원자(중첩 표/줄) 높이"만큼 안전 여유를 더해 보수적으로 분할. 작고 국소적, 오버플로 0 보장에 충분.
- 안 C: Stage 3까지를 task992 결과로 매듭(페이지 171 해결) + 남은 3건은 분할 위치 통일 전용 후속 타스크.
