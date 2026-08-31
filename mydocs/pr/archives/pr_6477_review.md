---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6477
issue: 6476
author: jangster77
---

# PR #6477 review - 동일 merge tree post-merge 검증 재사용

## 라우팅과 metadata

- PR: [#6477](https://github.com/edwardkim/rhwp/pull/6477), base: `devel`.
- 작성자 self-review: `jangster77` collaborator PR이므로 reviewer request는 등록하지 않았다.
- code candidate: `5b8ee8b510e00e16528a367dae828bd1a4525f35`
  (`ci: 동일 merge tree의 post-merge 검증을 재사용한다`), 5 files, `+265/-21`.
- 문서 작성 시점 참고 상태: `mergeable=MERGEABLE`, `mergeStateStatus=BLOCKED`.
  최신 head의 GitHub Actions 완료 및 merge 직전 재확인이 필요하다.
- 관련 이슈: [#6476](https://github.com/edwardkim/rhwp/issues/6476). PR 본문의 `Closes #6476`는
  merge 뒤 자동 종료 여부를 확인한다.
- base route: `collaborator_self_merge.md`.
  modifiers: `intake_and_review.md`, `local_validation.md`.
  loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `github_operations.md`.

## 변경 범위와 판단

이전 trusted post-merge verifier는 PR head가 최종 merge base를 조상으로 포함하지 않으면, 최종 merge tree가
이미 PR merge ref와 완전히 같아도 재사용을 거부했다. 이 PR은 PR event에서 실제 merge ref의 두 parent와
tree SHA를 artifact로 보존하고, devel push에서 최종 merge commit과 그 증적을 다시 대조한다.

다음 조건을 모두 만족할 때만 성공한 PR CI를 재사용한다.

1. artifact가 가리키는 merge ref parent가 final merge의 base parent와 PR head다.
2. artifact tree와 final merge tree가 정확히 일치한다.
3. 최종 성공 PR workflow run 및 필요한 Full-lane 증거가 존재한다.

그 경우 기존 duration artifact를 내려받아 `Refresh nextest target duration data`만 실행한다. 증적 누락,
parent 또는 tree 불일치, CI enforcement 변경, stale squash merge와 같은 경계는 새 예외로 열지 않고 기존
Full lane으로 fail-closed 처리한다. Rust 제품 소스, renderer, HWP/HWPX/PDF fixture는 변경하지 않아
visual sweep은 대상이 아니다.

## 로컬 검증

- `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs`:
  15 passed, 0 failed.
- `python3 -m unittest scripts.tests.test_trusted_postmerge_ci_reuse_workflow`:
  6 passed, 0 failed.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`:
  171 passed, 0 failed.
- `actionlint .github/workflows/trusted-postmerge-ci-reuse.yml`: 통과.
- `git diff --check`: 통과.
- CI workflow와 JavaScript/Python 계약만 변경했으며 Rust source/test/baseline helper는 변경하지 않아
  Cargo lint/test와 visual sweep은 실행하지 않았다.
- 최초 PR CI에서 `actions/upload-artifact` ref가 39자리여서 action download 전에 reusable workflow가
  실패했다. runner가 제시한 40자리 SHA로 보정하고, workflow의 모든 `uses:` ref 길이를 확인하는 계약을
  추가했다. 보정 head의 관련 workflow contract와 전체 workflow contract를 다시 실행해 통과했다.

## 위험과 CI 관찰

- artifact name 또는 manifest가 예상과 다르거나 API 조회가 불완전하면 verifier는
  `pr-merge-tree-evidence-unavailable`로 재사용을 거부하고 Full lane을 실행한다.
- artifact를 정상적으로 찾더라도 final merge tree의 parent와 tree가 모두 같아야 하므로, 최신 base에서
  code tree가 달라진 경우에는 PR 결과를 재사용하지 않는다.
- merge 후에는 동일 merge tree의 stale-base PR에서 CI/CodeQL/Adapter/Proptest의 heavy worker가 재사용되고,
  duration refresh만 수행되는지를 실제 devel push run으로 확인해야 한다.

## 최종 판단

**수용 후보, CI 대기.** 로컬 policy contract는 모두 통과했고 재사용 경로는 exact merge-tree 증적으로
좁게 제한된다. 최종 수용과 merge는 최신 PR head의 required checks 및 mergeability를 확인한 뒤에만 진행한다.
