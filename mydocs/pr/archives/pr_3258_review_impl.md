# PR #3258 collaborator 보정 실행 계획

## 적용 범위

- contributor source head: `94a55ccfeb0e5f32ea38a4763205561e5b59bd93`
- 보정 예정 commit: `d72f2f98f` — `info` 다중 입력을 사용법 오류로 처리하고 CLI contract 회귀를 추가
- review 문서·오늘할일 commit은 보정 code commit과 분리한다.

## 순서

1. 사용자 push 승인 뒤 #3258의 GitHub head SHA, `ls-remote`, `review/pr3258-maintainer` fetch SHA가 같은지 확인한다.
2. 해당 branch에서만 `d72f2f98f`를 cherry-pick하고 focused test, `git diff --check`를 다시 확인한다.
3. 이 review 문서와 오늘할일을 별도 docs commit으로 만든다. code/test가 있으므로 fast-pass가 아니라 최신 full CI를 기다린다.
4. CI·mergeable·작업지시자 merge 승인을 재확인한 뒤 #3262보다 먼저 merge한다. 실패 시 collaborator 보정 commit만
   되돌리면 원 feature head로 복귀한다.
