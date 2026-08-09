---
kind: review-implementation
status: completed-local-awaiting-final-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4102 구현·검토 기록

## commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `b38d715f7` | current-base merge bridge 검증, workflow 회귀 테스트, 운영 문서 보정 |
| 2 | 이 commit | PR review·오늘할일 trailing review-only 기록 |

## 완료 단계

1. [#4101](https://github.com/edwardkim/rhwp/issues/4101)에 #4073의 full CI 재실행 원인과 fail-closed 요구사항을 기록했다.
2. CI와 CodeQL의 inline preflight에 pending bridge 상태를 추가했다.
3. current base를 정확히 한 부모로 둔 merge만 bridge 후보로 보관하고, PR head를 실행하지 않는 `git merge-tree`
   검증이 true일 때만 final `fast_pass=true`를 출력하도록 했다.
4. bridge 검증 실패는 preflight 자체를 실패시키지 않고 final `fast_pass=false`로 내려 full lane을 실행하게 했다.
5. workflow 계약·CI 영향 분류·`actionlint`·#4073 실제 tree 재현과 code head 전체 CI·CodeQL을 완료했다.

## 남은 순서

1. 이 review-only commit push 뒤 preflight와 최종 Build & Test aggregate가 성공하고 heavy worker가 skip됐는지 확인한다.
2. 최신 PR head와 `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`을 확인한다.
3. 작업지시자 승인 뒤 merge하고, #4101 close 상태·후속 comment·`devel` 동기화·작업 branch 정리를 수행한다.

새로운 source merge를 만드는 것은 이 구현의 검증 절차가 아니다. future PR에서 이미 발생한 current-base merge가
자동 tree 조건을 만족할 때만 이 호환 경로가 선택된다.
