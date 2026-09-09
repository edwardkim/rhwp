---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4373 메인터너 보정·통합 계획

## Commit 소유권과 순서

1. `3cc9097730e7d1bc0c9fbd48e97c209bbe2d8a26` — contributor의 setup action 구현
2. `cad397a1894e82f69a0c17f1356b071143816df5` — contributor의 smoke EPIPE 보정
3. `c575d1a69e3940a93aaeb624bb7d2d9fac45f07a` — contributor의 macOS checksum 보정
4. `1e7c844472da1b101f6fbcc3808bb93c9b163c89` — maintainer code/test/workflow 보정
5. 이 문서를 포함하는 별도 trailing review-doc commit

Contributor commits는 재작성하지 않았다. 상세 판정은 [PR #4373 검토 기록](pr_4373_review.md)에 있다.

## 단계

1. **완료:** contributor head, 기존 3-OS self-test와 merge 상태를 접수 기준으로 기록했다.
2. **완료:** 고유 install 경로, Windows 경로 변환과 repeat self-test를 단일 maintainer commit으로 추가했다.
3. **완료:** focused unittest, CI 배선, diff check와 single-parent history를 검증했다.
4. **대기:** 작업지시자 push 승인 뒤 correction과 review-doc commit만 source에 fast-forward로 반영한다.
5. **대기:** Full CI fallback으로 최신 head의 CI·CodeQL과 Linux/Windows/macOS Action Self-test를
   모두 확인한다. 특히 Windows `cygpath` 경계와 같은 job 2회 호출이 필수 확인 대상이다.
6. **대기:** 최신 required checks와 mergeability 성공 뒤 별도의 merge 승인을 받는다.
7. **merge 후:** merge SHA와 최종 self-test 결과를 기록하고 archive/cleanup 절차를 수행한다.

## Rollback

- push 전에는 local branch/worktree만 제거한다.
- push 뒤 merge 전에는 review-doc commit과
  `1e7c844472da1b101f6fbcc3808bb93c9b163c89`을 역순 revert한다.
- merge 뒤에도 revert를 사용하며 contributor commit이나 source branch history를 rewrite하지 않는다.

현재 단계에는 push, GitHub review/comment, merge 승인이 없다.
