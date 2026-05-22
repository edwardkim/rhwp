# 최종 보고서 — Task #1073: 중첩 표(셀 내부 표) 페이지 분할

- 이슈: edwardkim/rhwp#1073
- 브랜치: `local/task1073` (stream/devel `be2a71c4` 기준)
- 수정: `table_layout.rs`, `height_measurer.rs`, `table_partial.rs` + 회귀 가드 `tests/issue_1073_*.rs`

## 증상
셀 안에 페이지보다 큰 중첩 표가 있는 표가 페이지 경계에서, 외부 행 분할만으로 페이지에 안 들어가
본문 하단을 758px 초과(`kps-ai.hwp` pi=674, 외부 3×6 래퍼표 셀[0]의 29행 중첩 표).

## 근본 원인
중첩 표가 4개 레이어에서 atom 으로 취급되어 `advance_row_cut` 진입 자체가 차단:
1. `is_row_splittable`(line_heights.len()>1) — 중첩 표 셀은 1 → false.
2. `MeasuredCell` — 중첩 표를 단일 높이로 측정.
3. `cell_units` — 중첩 표 atom 1개.
4. 부분 렌더 — 단일 셀 available_h 휴리스틱(연속 페이지 row0 재렌더).

## 한컴 정답지 (`pdf/kps-ai-2022.pdf` p62~63)
큰 표를 페이지 경계에서 **행 단위로 분할**(p62 제목~3.민간소프트웨어 / p63 시장침해~5.종합의견).

## 수정 (5점, 4 레이어)
- `CellUnit + nested_row`: 가시 텍스트 없는 단일 중첩 표(2행+) 문단을 per-중첩행 유닛 분해.
  Σ = `calc_nested_table_height`(드리프트 0).
- `MeasuredCell.nested_split_row_count` + `is_row_splittable` 확장.
- `row_cut_content_height`/`cell_line_ranges_from_cut` 유닛 기반 자동 정합.
- `layout_partial_table`: 컷 유닛 인덱스 → `NestedTableSplit{start_row,end_row}` 직접 구성
  (연속 페이지 start_row 반영).
- `layout_table_cells`: 연속 페이지(sr>0) 의 `r<sr` rowspan 라벨 셀 공란화
  (외부 advance_row_block_cut 정합).

## 검증
- kps-ai pi=674: 중첩행 컷(end_cut=[20]/start_cut=[20]), **758px overflow 해소**.
  page66=제목~3.민간소프트웨어(PDF p62), page67=시장침해~5.종합의견+footer(PDF p63).
  콘텐츠 완전, 누락/중복 없음, rowspan 라벨 연속 페이지 공란.
- 전수 sweep: 3057→3055 lines / 382815→382054px (**회귀 0**, kps-ai·hwpctl 개선).
- 회귀 가드 3 신규, 골든 8/8, cargo test lib 1324 + 통합 0 failed, clippy/fmt clean.

## 잔여 (known limitation)
- break row 가 한컴 대비 ~2 중첩행 늦음(측정 정밀도). 구조/콘텐츠 정합·overflow 0 달성.
- 범위 외(atom 폴백): 2단계+ 중첩, 텍스트 동거 문단 중첩 표.

## 후속
overflow 인벤토리 B군 처리 완료. C군(교육/실전 통합 누적 드리프트), D군(pr-149 Shape)은 별도.
