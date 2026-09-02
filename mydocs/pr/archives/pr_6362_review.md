---
kind: pr-review
status: archived
pr: 6362
issue: 6360
merged_at: 2026-08-29T09:48:53Z
---

# PR #6362 검토 - 개별 회귀 시간 기반 nextest suite 배분

## 결론 - 수용 및 병합 완료

[PR #6362](https://github.com/edwardkim/rhwp/pull/6362)는 일반 merge commit
[`86aa6128b8cc928c0cf79ed9bb8f27d0f74b7924`](https://github.com/edwardkim/rhwp/commit/86aa6128b8cc928c0cf79ed9bb8f27d0f74b7924)로
`devel`에 병합됐다. 구현 후보의 최종 head는
`24c872976ca0e18cd6870ab507cf089a13f3defe`다.

## 변경 판단

- `regression_suite_NNN`은 매 생성에서 포함 source가 달라지는 파생 target이므로, 과거 target 이름의
  실행시간을 그대로 B/C/D 배분 근거로 쓰지 않는다.
- JUnit 측정을 schema v2로 확장해 source-case와 개별 test case 시간을 수집하고, 현재 manifest의
  source 구성에 그 시간을 합산해 suite를 재배정한다.
- 기존 trusted v1 측정 artifact만 있는 post-merge 재사용은 v2 policy를 되돌리지 않고 유지한다.
- Render Diff merge-bridge/review-tail reuse 정책은 변경 범위에서 제외했다.

## 검증 기록

- 구현 PR 최신 head의 required CI는 성공했다. Archive A/B/C/D, Rust CodeQL, Native Skia를 포함한
  full CI가 통과했고, 정책상 생략되는 WASM Build와 Frontend unit gate만 skip이었다.
- 초기 head `679bc2ff232fbcba2ddb794ffd69eb31bce1ef07`에서는 동적 재배정 harness를 정적 기본
  manifest로 다시 검증해 generated harness drift가 발생했다. maintainer가 동적 manifest 자체를
  검증하도록 보정해 최종 head `24c872976ca0e18cd6870ab507cf089a13f3defe`에서 통과시켰다.
- 로컬 계약 검증:
  - `node --test scripts/tests/nextest-target-duration-policy.test.mjs` - 10 passed
  - `node --test scripts/tests/rust-test-suite-manifest.test.mjs` - 19 passed
  - CI 영향 및 trusted post-merge reuse Node 계약 테스트 - 78 passed
  - `python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'` - 166 passed
  - `git diff --check` - passed
- 병합 뒤 `devel` push는 CI 실행면 변경이므로 full lane이 정상 실행됐다.
  [CI run 33246360984](https://github.com/edwardkim/rhwp/actions/runs/33246360984)와
  [CodeQL run 33246361008](https://github.com/edwardkim/rhwp/actions/runs/33246361008)가 모두
  성공했다.

## 후속 상태

- [Issue #6360](https://github.com/edwardkim/rhwp/issues/6360)은 코드 PR 본문이 `Refs #6360`만
  사용했으므로 자동 close되지 않았다. 별도 close 지시 전까지 열린 상태를 유지한다.
- 이 문서는 Option B review-only 기록 PR에서 반영한다. 기록 PR은 코드·test·workflow를 포함하지 않는다.
