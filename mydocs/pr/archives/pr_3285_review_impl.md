# PR #3285 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `9132da68c642ab282619534a4c51034a30b729a4`
- 보정 예정: `5f495e60a`(textbox address·Unicode offset), `a60ae3294`(회귀 test module 통합)

## 순서

1. 최신 source SHA를 GitHub/API, `ls-remote`, local fetch에서 일치시킨 뒤 `review/pr3285-maintainer`로 전환한다.
2. 두 보정 commit을 적용하고 test와 docs/오늘할일 commit을 분리한다.
3. latest full CI 뒤 사용자 merge 승인으로 처리한다. rollback은 두 collaborator commit만 역순 revert한다.
