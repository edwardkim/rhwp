# Task #1753 구현계획서 — 지연 표 후속 문단 선행 채움 (3단계)

수행계획서: `mydocs/plans/task_m100_1753.md`

## Stage 1 — 재현 샘플 동결 + 실패 재현
- `samples/task1753/deferred_takeplace_fill_ahead.hwpx` (2814765) 복사 + README
  (한글 PDF 시각 증거 요약 포함).
- dump-pages 실패 기록: pi52/53 이 11쪽 (한글 9쪽).

## Stage 2 — 구현 + 테스트
- `src/renderer/typeset.rs`:
  - `TypesetState.prefilled_paras: HashSet<usize>` 추가, 메인 루프 진입부 스킵.
  - `typeset_block_table` 에 `paragraphs`/`composed` 플럼빙 (호출부 2곳: typeset_table_paragraph,
    flush_deferred_table_controls).
  - 이월 분기(multirow_clean_defer 등 advance 직전)에서 가드 성립 시
    `prefill_following_paragraphs` 수행 (수행계획서 가드/후보 조건).
- 단위테스트: prefill 후보 판정 헬퍼 (같은 쪽 연속 vpos 신뢰 / 누적좌표 배제 / controls 배제).
- 통합테스트 `tests/issue_1753_deferred_table_fill_ahead.rs`: pi52/53 이 9쪽(전 fragment 쪽),
  표 fragment 는 10쪽부터.

## Stage 3 — 회귀 검증 + 최종보고
- lib + 통합 + 페이지 게이트(국제고속선기준 251 포함) + 한글 OLE 대조(재현/잔여 mismatch/
  MATCH 150) — 악화 0.
- rustfmt/clippy → 최종보고서 → squash → PR.
