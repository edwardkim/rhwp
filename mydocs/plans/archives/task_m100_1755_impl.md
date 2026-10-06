# Task #1755 구현계획서 — host 제목 줄 pre-emit + layout 억제 (3단계)

수행계획서: `mydocs/plans/task_m100_1755.md`. 재현 샘플은 #1753 것 재사용
(`samples/task1753/deferred_takeplace_fill_ahead.hwpx`) — 신규 동결 불필요.

## Stage 1 — typeset pre-emit
- `TypesetState.pre_emitted_host_paras: HashSet<usize>` 추가.
- `prefill_before_deferred_table` 진입 가드 통과 직후: host fmt 계산 →
  `PartialParagraph{para_idx, 0..line_count}` push + `line_advances_sum` 소비 + 기록.
- `PaginationResult.pre_emitted_host_paras` 신설 + typeset 결과 복사
  (Paginator/기타 구성처는 빈 집합 기본).

## Stage 2 — layout 억제 배선
- layout 에 `set_pre_emitted_host_paras` 세터(hidden_empty_paras 패턴).
- `render_deferred_rowbreak_host_text_after` 렌더 시 pre-emit 문단이면 스킵.
- 호출부(document_core/queries/rendering.rs) 배선.

## Stage 3 — 검증 + 보고
- 통합테스트 확장(issue_1753 테스트에 pi51 9쪽 PartialParagraph 단언 추가 또는 신규 테스트).
- SVG 시각(9쪽 제목 렌더, 11쪽 미렌더), 한글 OLE 대조(2814765 완전 MATCH), lib/통합/게이트/
  코퍼스 배치, fmt/clippy → 최종보고서 → squash → PR.
