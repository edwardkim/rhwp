# PR #3262 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `f8bd6017245002b99126a08a4b9f60e60549f9f0`
- 선행: #3258 merge 및 `devel` 반영
- 보정 예정 commit: `df3797ce4` — `export-structure` 다중 입력 사용법 오류·회귀 test

## 순서

1. #3258 merge 뒤 Update branch가 필요하면 사용자 승인 뒤 실행하고 최신 source head를 다시 고정한다.
2. `review/pr3262-maintainer`에서 보정 code/test와 review·오늘할일 docs를 별도 commit으로 추가한다.
3. full CI 성공 뒤 사용자 승인으로 merge한다. 이후 #3276, #3282, #3280, #3285, #3288을 순서대로 처리한다.
4. rollback은 collaborator 보정 commit만 revert하며 contributor 원 feature commit은 rewrite하지 않는다.
