# #2320 1단계 완료보고 — 재현 고정 실패 테스트

- 계획서: `mydocs/plans/task_m100_2320.md`
- 브랜치: `local/task2320`

## 산출물

`tests/issue_2320_vpos_rewind_page_break.rs` (테스트 2건):

1. `issue_2320_last_column_rewind_splits_to_next_page` — **red**:
   - 문단 29 가 p2 단 1 에서 `PartialParagraph lines=0..1` 로 끊겨야 하나
     현행은 `FullParagraph pi=29 [vpos-rewind@line1]` 통째 배치
   - 문단 30 의 p2 부재 단언 (하단 잘림 소멸 확인용)
   - 문단 29 잔여(lines=1..)의 p3 계속 단언
   - 전체 페이지 수 7 불변 단언
2. `issue_2320_existing_column_zero_split_unchanged` — **green (가드)**:
   문단 21 의 기존 단 0→1 분할(lines=0..2 / 2..5) 불변 고정

## 확인

실패 메시지가 착수 정찰 진단과 일치 — 되감김 마커가 항목에 그대로 찍힘:
`FullParagraph pi=29 vpos=67050..4926 [vpos-rewind@line1]`

## 다음 단계

2단계: engine 게이트 확장(`current_column == 0` 한정 해제) + 마지막 단
되감김의 쪽 진행 분할 구현.
