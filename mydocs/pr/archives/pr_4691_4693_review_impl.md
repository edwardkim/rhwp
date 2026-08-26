---
kind: implementation-review
status: completed
prs: [4691, 4693]
authors: [kevin9327]
base: devel
---

# PR #4691 · #4693 누적 체리픽 검토 이행 기록

## 목적과 범위

kevin9327의 열린 PR 두 건을 최신 `upstream/devel` `f7a98ce04` 기준에서 오래된 PR 번호
순으로 누적 검토한다. 이 branch는 충돌·회귀 검증을 위한 로컬 후보이며, contributor
branch를 재작성하거나 원격 `devel`에 직접 push하지 않는다.

## 적용 순서

| 순서 | 원 PR / 원 commit | 로컬 적용 commit | 범위 | 충돌 |
| --- | --- | --- | --- | --- |
| 1 | #4691 / `86594ea84` | `d13287690` | core-cli 기준 풀이 14건과 계약 보강 | 없음 |
| 2 | #4691 / `2e18701446` | `adf2d0c4b` | Lint 레인용 명령 선언 가드 정정 | 없음 |
| 3 | #4693 / `1d98b15258` | `fc0b47d01` | 트랙 L roadmap·자산 | 없음 |
| 4 | #4693 / `d2a8743f11` | `2f244e032` | MCP resource 실측 정정 | 없음 |

## 검증과 rollback 경계

검증은 `pr_4691_review.md`와 `pr_4693_review.md`에 PR별로 분리해 기록했다. #4691의
전 pack 기준 풀이 100/100 및 221/221 채점은 #4693 문서 변경과 무관하지만, 누적 tree에서
실행해 두 변경이 공존하는 후보의 회귀가 없음을 확인했다.

문제가 확인되면 이 검토 branch만 폐기하거나 해당 로컬 체리픽 commit을 되돌린다. 원 contributor
history와 원격 PR branch는 amend, rebase, force-push하지 않는다.

## 다음 게이트

1. 실제 merge 직전에 원 PR별 최신 head·mergeable 상태·필수 GitHub Actions를 재확인한다.
2. 작업지시자 승인 뒤에만 원격 review/comment 게시와 admin merge를 수행한다.
3. merge 완료 뒤 각 review 문서를 `mydocs/pr/archives/`로 이동하고, 오늘할일·merge SHA·이슈
   상태를 기록한 후 post-merge 절차를 따른다.
