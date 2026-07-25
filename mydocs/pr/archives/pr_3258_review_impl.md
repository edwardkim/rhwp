# PR #3258 v2 통합 보정 계획

## 적용 범위

- contributor source head: `94a55ccfeb0e5f32ea38a4763205561e5b59bd93` (원 PR에는 추가 push하지 않음)
- v2 보정 commit: `d72f2f98f` — `info` 다중 입력을 사용법 오류로 처리하고 CLI contract 회귀를 추가
- review 문서는 v2 통합 증적으로 유지하며 오늘할일은 통합 PR 최종 묶음에서만 추가한다.

## 순서

1. v2에서 원 feature와 `d72f2f98f`를 함께 검증한다.
2. code/test가 있으므로 통합 PR의 최신 full CI를 기다린다.
3. #3262보다 먼저 통합한다. 실패 시 v2의 보정 commit만 revert하며 contributor 원 PR은 rewrite하지 않는다.
