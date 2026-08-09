---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4372 메인터너 보정·통합 계획

## Commit 소유권과 순서

1. `c1d4da0878c94c898a4ebb903c4b9f6b46fc14b2` — contributor의 Docker CLI/GHCR 구현
2. `91782d3364b3b8929cac00a4d449e6785d93e6a1` — contributor의 Docker context 보정
3. `71aecd1273864ae42b5b19fa9382aa43c8f0ef77` — maintainer code/test/workflow 보정
4. 이 문서를 포함하는 별도 trailing review-doc commit

Contributor commits는 그대로 두고 maintainer commit만 선형으로 이었다. 검토 판정은
[PR #4372 검토 기록](pr_4372_review.md)에 있다.

## 단계

1. **완료:** contributor head와 기존 CI/Docker check를 접수 기준으로 고정했다.
2. **완료:** tag/source/Cargo 검증과 prerelease latest 보호를 maintainer commit 하나로 추가했다.
3. **완료:** focused contract tests, CI 배선, diff와 single-parent history를 검증했다.
4. **대기:** 명시적 push 승인 뒤 maintainer correction과 review-doc commit만 source branch에
   fast-forward push하고 원격 head가 예상 SHA인지 확인한다.
5. **대기:** code/workflow 변경이므로 Full CI fallback으로 최신 head의 CI, CodeQL,
   Docker CLI Image build와 required aggregate를 확인한다.
6. **대기:** 최신 mergeability와 check가 성공한 뒤 별도 merge 승인을 요청한다. 승인 전에는
   review 게시, image publish, merge를 수행하지 않는다.
7. **merge 후:** merge SHA와 GHCR workflow 결과를 확인하고 기록 archive 및 로컬 정리를 수행한다.

## Rollback

- push 전 rollback은 visibility branch/worktree 제거로 한정된다.
- push 뒤 merge 전에는 review-doc commit을 먼저 revert한 뒤
  `71aecd1273864ae42b5b19fa9382aa43c8f0ef77`을 revert한다.
- merge 후에도 역순 revert를 사용하며 contributor history를 rebase, amend 또는 force-push하지 않는다.

현재 작업에는 push나 merge 권한이 포함되지 않는다.
