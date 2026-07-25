# PR #3262 collaborator 보정 실행 계획

## 적용 범위

- contributor 원 source head: `f8bd6017245002b99126a08a4b9f60e60549f9f0`
- 메인터너 보정: `6097885e5` — `export-structure` 다중 입력 사용법 오류와 회귀 test
- review 문서·오늘할일은 code/test 보정과 별도 commit으로 분리한다.

## 처리 기록과 다음 단계

1. GitHub head, `ls-remote`, `review/pr3262-maintainer` fetch SHA가 원 source SHA와 일치함을 확인했다.
2. 같은 루트 작업트리에서 보정을 추가하고 focused `cli_json_contract` 16 passed를 확인했다.
3. 이 문서와 오늘할일을 docs commit으로 분리하고 최신 full CI를 기다린다.
4. #3258 merge 및 필요 시 Update branch 뒤 CI·mergeable·사용자 merge 승인을 재확인한다. rollback은
   `6097885e5`만 revert하며 contributor 원 feature commit은 rewrite하지 않는다.
