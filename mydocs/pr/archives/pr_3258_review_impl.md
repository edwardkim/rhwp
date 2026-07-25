# PR #3258 collaborator 보정 실행 계획

## 적용 범위

- contributor 원 source head: `94a55ccfeb0e5f32ea38a4763205561e5b59bd93`
- 메인터너 보정: `1b9cf58fd` — `info` 다중 입력을 사용법 오류로 처리하고 CLI contract 회귀를 추가
- review 문서·오늘할일은 code/test 보정과 별도 commit으로 분리한다.

## 처리 기록과 다음 단계

1. GitHub head, `ls-remote`, `review/pr3258-maintainer` fetch SHA가 원 source SHA와 일치함을 확인했다.
2. 같은 루트 작업트리의 `review/pr3258-maintainer`에서 보정을 추가하고 focused contract test를 통과했다.
3. 이 문서와 오늘할일을 docs commit으로 분리한다. code/test가 있으므로 최신 head full CI를 기다린다.
4. CI·mergeable·작업지시자 merge 승인을 재확인한 뒤 #3262보다 먼저 merge한다. rollback은 `1b9cf58fd`만
   revert하며 contributor 원 feature commit은 rewrite하지 않는다.
