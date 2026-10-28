# Task #1749 구현계획서 — saved bounds 신뢰에 페이지-마지막 증거 조건 (3단계)

수행계획서: `mydocs/plans/task_m100_1749.md`

## Stage 1 — 재현 샘플 동결 + 실패 재현
- `samples/task1749/saved_bounds_cumulative_vpos.hwpx` 로 재현 파일(36371084) 복사 + README
  (opengov 결재문서 계열, PII 방침 A 선례 — samples/hwpx/opengov README 참조).
- dump-pages 실패 기록(1쪽 used 1011.8px, pi18 1쪽 배치) + FIT 트레이스 재확인.

## Stage 2 — 수정 + 테스트
- `src/renderer/typeset.rs` `saved_single_line_bottom_fits` 계산에 조건 추가:
  다음 문단의 첫 실줄(synthetic 제외)이 없거나(vpos 리셋 포함: next_vpos < curr_vpos)
  일 때만 bounds 신뢰. 헬퍼 함수로 분리해 단위테스트 2개(리셋형 신뢰 / 누적형 불신).
- 통합테스트 `tests/issue_1749_saved_bounds_cumulative.rs`: pi18 이 1쪽에 없고 2쪽에 있음.
- 재현 파일 dump-pages 로 1쪽 used ≤ body 확인.

## Stage 3 — 회귀 검증 + 최종보고
- `cargo test --lib`(fe6de3ef 합성 테스트 포함) + 통합(1417/546/1486/diag1042/1750).
- 페이지 게이트: byeolpyo1/4 · 승강기 · task1700 · task1745 · task1750 + 국제고속선기준(#1725).
- 한글 OLE 대조: 재현 + mismatch 잔여 배치 + MATCH 표본 150건 — 악화 0.
- rustfmt/clippy → 최종보고서 → squash → PR.
