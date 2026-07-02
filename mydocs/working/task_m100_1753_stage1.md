# Task #1753 Stage 1 완료보고서 — 재현 샘플 동결 + 실패 재현

## 수행 내용
- `samples/task1753/deferred_takeplace_fill_ahead.hwpx` 동결 (2814765, 118KB, 21쪽) + README.
- 실패 재현 (upstream/devel + PR #1744/#1746/#1751/#1752 적용 베이스):
  ```
  페이지 10: PartialTable pi=51 rows 0..34 (cont=false)
  페이지 11: PartialTable pi=51 rows 34..57 → FullParagraph pi=52, pi=53
  ```
  한글(PDF 시각 + 저장 LINE_SEG vpos=72581/74121): pi52/53 은 **9쪽 하단**.

## 확인 사항
- 이월 지점: `typeset_block_table` 의 multirow_clean_defer 분기(advance_column_or_new_page).
- 기존 host 텍스트 pre-emit 메커니즘(place_table_with_text 의 PartialParagraph)은 fit 경로
  전용 — split 경로는 layout 의 "분할 표 첫 부분" 렌더에 의존(제목 줄 10쪽, 범위 밖 후속).

## 상태
완료. Stage 2 (prefill 구현 + 테스트) 진행.
