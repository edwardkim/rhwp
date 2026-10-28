---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
pr: 6488
issue: 6279
author: jangster77
---

# PR #6488 review - review/PDF tail의 Full 후보 재사용

## 라우팅과 metadata

- PR: [#6488](https://github.com/edwardkim/rhwp/pull/6488), base: `devel`.
- 작성자 self-review: `jangster77` collaborator PR이므로 reviewer request는 등록하지 않았다.
- code candidate: `53d9fea2ed4203a4b87c6cf59fab8f010fc144de`
  (`fix(ci): review/PDF tail의 Full 후보를 재사용한다 (#6279)`), 4 files, `+203/-34`.
- 이 review record는 code candidate 뒤에 붙는 trailing documentation commit이다. 문서 작성 시점의
  code candidate는 `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`이고, merge 직전에는 이 review
  record를 포함한 최신 head의 required check와 mergeability를 다시 확인한다.
- 관련 이슈: [#6279](https://github.com/edwardkim/rhwp/issues/6279). PR 본문의 `Closes #6279`는
  merge 뒤 자동 종료 여부를 확인한다.
- base route: `collaborator_self_merge.md`.
  modifiers: `intake_and_review.md`, `local_validation.md`, `post_merge.md`.
  loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `post_merge.md`.

## 변경 범위와 판단

`#6485`의 최종 head에는 code candidate 뒤에 허용된 review/PDF 증적 tail이 추가됐다. 기존 verifier는
최종 code candidate 하나만 조회해, 그 후보가 fast-pass였을 때 이전의 성공 Full run `40fbca176eee3874f5acf5687c91d4c9ec5a6d07`을
발견하지 못하고 `no-current-pr-workflow-candidate`로 Full lane을 다시 실행했다.

이 PR은 선형 review/PDF tail의 각 commit을 최신 순으로 Full 후보로 탐색한다. 후보는 해당 workflow run의
immutable merge-tree artifact가 후보 head를 정확히 가리킬 때만 채택한다. 최종 head의 stale-base
merge-tree 증적, 후보 run의 Full-lane 성공, duration artifact, CodeQL Analyze worker를 모두 다시
확인한다. 비선형 tail, artifact/API 누락, enforcement 변경, 증적 불일치는 계속 Full lane으로
fail-closed 한다.

CI controller와 JavaScript/Python 계약만 변경했다. Rust 제품 소스·test·baseline helper, renderer,
HWP/HWPX/PDF fixture와 visual output은 변경하지 않았으므로 visual sweep은 대상이 아니다.

## 로컬 검증

- `node --test scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs`:
  18 passed, 0 failed.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`:
  171 passed, 0 failed.
- `actionlint .github/workflows/trusted-postmerge-ci-reuse.yml`: 통과.
- `git diff --check`: 통과.
- Rust source/test/baseline helper와 renderer가 범위 밖이므로 Cargo lint/test와 visual sweep은 실행하지 않았다.

## CI와 잔여 위험

- code candidate `53d9fea2...`의 CI, CodeQL, Adapter inter-diff, Proptest와 workflow contract는 모두
  성공했다. 최초 `Frontend package gates`는 Chrome websocket startup timeout으로 실패했으나, 소스 변경
  없이 failed job 재실행 후 성공했다. 이는 이 PR의 controller 변경과 무관한 CI browser startup 변동으로
  기록하며, 최신 trailing head의 결과를 다시 확인한다.
- review/PDF tail의 허용 범위나 merge-tree 증적이 불완전하면 이 PR도 재사용하지 않고 Full lane으로
  되돌아간다. merge 뒤 실제 devel push가 `Refresh nextest target duration data`만 실행하는지는 post-merge
  확인 항목이다.

## 최종 판정

**승인.** code candidate의 로컬 계약 검증과 최신 GitHub Actions가 통과했다. merge 전 조건은 이 review
record를 포함한 최신 PR head의 required checks 성공, `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`,
그리고 작업지시자의 merge 승인이다.
