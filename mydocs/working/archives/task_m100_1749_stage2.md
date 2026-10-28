# Task #1749 Stage 2 완료보고서 — 페이지-마지막 증거 조건 + 테스트

## 수행 내용
- `src/renderer/typeset.rs`:
  - `saved_flow_marks_page_last(paragraphs, para_idx)` 신설 — 다음 문단의 첫 실줄
    (synthetic 제외)이 없거나(문서/구역 끝, fe6de3ef 합성 테스트 보호 케이스) 현재 줄보다
    작은 vpos 로 리셋(새 쪽)될 때만 참. 누적좌표 문서는 거짓.
  - `saved_single_line_bottom_fits` 조건에 `&& saved_flow_marks_page_last(...)` 추가 —
    저장 bounds 신뢰를 "저장 flow 가 이 줄을 페이지 마지막으로 인코딩한 경우"로 한정.
- 단위테스트 `test_saved_flow_marks_page_last` (문서 끝 신뢰/리셋 신뢰/누적 불신/빈 문단 스킵).
- 통합테스트 `tests/issue_1749_saved_bounds_cumulative.rs` (pi18 1쪽 미배치 + 2쪽 배치).

## 검증
- dump-pages: pi18 → 2쪽 시작, **1쪽 used 1011.8 → 981.4px** (본문 990.2px 내, overfill 해소),
  총 2쪽 불변.
- 신규 단위·통합테스트 통과.

## 상태
완료. Stage 3 (회귀 검증 + 최종보고) 진행.
