# PR #3262 v2 통합 보정 계획

## 적용 범위

- contributor source head: `f8bd6017245002b99126a08a4b9f60e60549f9f0` (원 PR에는 추가 push하지 않음)
- 선행: v2에서 #3258 feature·보정 반영
- v2 보정 commit: `df3797ce4` — `export-structure` 다중 입력 사용법 오류·회귀 test

## 순서

1. v2에서 feature·보정 code/test와 review 기록을 함께 유지한다.
2. full CI 성공 뒤 통합 PR을 사용자 승인으로 merge한다. 이후 #3276, #3282, #3280, #3285, #3288을 같은 PR에 포함한다.
3. rollback은 v2 보정 commit만 revert하며 contributor 원 feature commit은 rewrite하지 않는다.
