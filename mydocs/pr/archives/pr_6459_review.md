---
kind: pr-review
status: complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6459
issue: 6457
author: jangster77
---

# PR #6459 review - 문서-only 병합 후 worker 재사용

## 라우팅과 metadata

- PR: [#6459](https://github.com/edwardkim/rhwp/pull/6459), base: `devel`.
- 작성자 self-review: `jangster77` collaborator PR이므로 reviewer request는 등록하지 않았다.
- code candidate: `7ec7f99b399891f0484688282aba0b5cf0cc3f72`
  (`ci: 문서 전용 병합 후 worker 재사용`), 6 files, `+199/-0`.
- merge commit: `46fc02706e60da8fc959fc9e8adec2c2eb2b1672`, 2026-08-30 08:44:32 UTC.
- 관련 이슈: [#6457](https://github.com/edwardkim/rhwp/issues/6457). PR 본문은 이슈를 자동 종료하지 않는
  `Refs #6457`로 유지했다.

## 변경 범위와 판단

`trusted-postmerge-ci-reuse`가 동일 저장소의 직접 review-only PR만 다음 증거를 모두 만족할 때
post-merge worker 재사용 대상으로 판정하도록 보정했다.

1. 유일한 `devel` merge mapping과 merge tree=head tree가 일치한다.
2. PR 전체 파일과 linear history가 review-only 허용 범위다.
3. 정확한 PR workflow run에서 preflight는 성공하고 해당 heavy worker는 실제 `skipped`였다.

workflow, CI policy, source, test, fork, tree/history 불일치 또는 worker-skip 증거가 빠진 경우에는
Full 실행으로 닫는다. #6459 자체는 CI enforcement surface를 변경하므로 이 재사용 예외의 대상이 아니다.
renderer, sample, PDF, fixture는 변경하지 않아 visual sweep은 필요하지 않았다.

## 로컬 검증

- `git diff --check`: 통과.
- `actionlint .github/workflows/trusted-postmerge-ci-reuse.yml`: 통과.
- `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse-squash.test.mjs`:
  87 passed, 0 failed.
- `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'`: 169 passed.
- Rust source 변경이 없으므로 Cargo lint/test와 visual sweep은 실행하지 않았다.

## GitHub Actions와 병합 후 관측

- PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33301593399),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33301593385),
  [Adapter](https://github.com/edwardkim/rhwp/actions/runs/33301593381),
  [Proptest](https://github.com/edwardkim/rhwp/actions/runs/33301593390)가 모두 성공했다.
- merge commit의 [Adapter post-merge run](https://github.com/edwardkim/rhwp/actions/runs/33302332413)은
  trusted controller와 preflight를 성공한 뒤 worker를 2026-08-30 08:44:56--08:47:18 UTC에 Full 실행해
  성공했다.
- 같은 commit의 [Proptest post-merge run](https://github.com/edwardkim/rhwp/actions/runs/33302332392)도
  controller와 preflight 성공 뒤 worker를 08:44:56--08:47:42 UTC에 Full 실행해 성공했다.
- [devel CI](https://github.com/edwardkim/rhwp/actions/runs/33302332391)는 08:56:15 UTC,
  [devel CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33302332389)은 09:02:27 UTC에 모두
  성공했다.

위 Full 실행은 구현 PR이 CI enforcement surface를 변경했기 때문에 의도된 fail-closed 결과다. 이 문서와
오늘할일만 담는 후속 PR을 병합해 `direct-review-only-pr-fast-pass-reused`와 두 worker `skipped`를 별도로
검증한다.

## 최종 판단

**수용.** 직접 review-only PR의 PR 단계 skip 증거를 정확한 run/job 상태로 재검증하고, 증거가 없거나
경계가 달라지면 Full 실행을 유지한다. 이 문서-only follow-up의 PR 및 post-merge 결과가 모두 기대대로
skip일 때까지 [#6457](https://github.com/edwardkim/rhwp/issues/6457)은 열린 상태로 둔다.
