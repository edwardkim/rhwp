---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4376 메인터너 보정·통합 계획

## Commit 소유권과 순서

1. `cc94b1c0e6a5b3972212cbaf1fc8bca102b8be37` — contributor의 installer channel 구현
2. `12f3950644430d7692baec6587f8a1d149f8cae4` — contributor의 guide 링크 보정
3. `b1bb5a56a15aa06f84ee99eeecdfb208d5cc406b` — maintainer code/test/workflow 보정
4. 이 문서를 포함하는 별도 trailing review-doc commit

Contributor commits는 재작성하지 않았다. 상세 근거는 [PR #4376 검토 기록](pr_4376_review.md)에 있다.

## 단계

1. **완료:** contributor head와 기존 check를 접수 기준으로 고정하고 installer workflow 미실행을 분리 기록했다.
2. **완료:** immutable tag source와 portable checksum 보정을 maintainer commit 하나로 추가했다.
3. **완료:** focused unittest, workflow test 배선, shell syntax, diff와 commit parent를 검증했다.
4. **대기:** 작업지시자 승인 뒤 correction과 review-doc commit만 source branch에 fast-forward push한다.
5. **대기:** code/workflow 변경이므로 Full CI fallback을 사용한다. 최신 CI·CodeQL과 별도
   Release Installers run에서 deb/rpm/MSI, macOS installer 및 crates.io dry-run 결과를 확인한다.
   credentials가 없는 publish step의 명시적 skip과 build failure를 혼동하지 않는다.
6. **대기:** required checks, mergeability와 작업지시자의 별도 merge 승인을 모두 확인한다.
7. **merge 후:** merge SHA, release workflow 상태와 asset 정합을 확인하고 archive/cleanup을 수행한다.

## Rollback

- push 전에는 visibility branch/worktree만 정리한다.
- push 뒤 merge 전에는 review-doc commit을 먼저 revert하고,
  `b1bb5a56a15aa06f84ee99eeecdfb208d5cc406b`을 revert한다.
- merge 뒤에도 역순 revert를 사용한다. contributor commits나 release tag history를 rewrite하지 않는다.

현재 단계에는 push, publish, GitHub review/comment 또는 merge 승인이 없다.
