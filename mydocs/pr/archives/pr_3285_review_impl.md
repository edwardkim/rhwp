# PR #3285 v2 통합 보정 계획

## 적용 범위

- contributor source head: `9132da68c642ab282619534a4c51034a30b729a4` (원 PR에는 추가 push하지 않음)
- v2 보정: `5f495e60a`(textbox address·Unicode offset), `a60ae3294`(회귀 test module 통합)

## 순서

1. v2에서 두 보정 commit과 review 기록을 함께 유지한다.
2. test와 docs는 통합 PR에 포함하고 오늘할일은 최종 PR 묶음에서만 추가한다.
3. latest full CI 뒤 사용자 merge 승인으로 처리한다. rollback은 두 v2 보정 commit만 역순 revert한다.
