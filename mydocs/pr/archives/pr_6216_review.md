# PR #6216 검토 기록

## 대상

- PR: [#6216](https://github.com/edwardkim/rhwp/pull/6216)
- 제목: `perf(ci): #6213 green PR 검증을 merge 뒤 재사용한다`
- 기준 브랜치: `devel`
- head: `task_m100_6213_trusted_postmerge_reuse`
- 관련 이슈: [#6213](https://github.com/edwardkim/rhwp/issues/6213)

## 검토 범위

- `devel` merge commit이 정확히 하나의 green PR 검증 결과를 재사용할 수 있는지 판정하는 공통 workflow를 추가했다.
- CI, CodeQL, adapter diff, proptest-roundtrip에 동일한 fail-closed 판정을 연결했다.
- nextest B/C JUnit 측정에서 testcase 기준 target 시간을 수집하고, 빈 값과 서로 다른 provenance의 정책 반영을 거부한다.

## 안전성 판단

- 재사용은 `devel`의 2-parent merge, 정확히 일치하는 merged PR, source head와 merge tree, base 계보, 최신 successful PR workflow run, enforcement 경로 무변경을 모두 만족할 때만 허용한다.
- direct push, stale base, merge-queue 형태, pending/failed/missing source run, workflow 또는 CI 계약 변경은 모두 전체 CI로 폴백한다.
- CI duration 정책은 B/C artifact가 같은 `run_id`, `ref`, `sha`를 가지며 target map이 비어 있지 않을 때만 승격한다.
- 첫 배포 PR은 기준 `devel`에 verifier helper가 아직 없으므로 의도적으로 전체 CI를 실행한다. helper를 PR head에서 실행하지 않아 self-approval 경로를 만들지 않는다.

## 로컬 검증

- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `node --test scripts/tests/nextest-target-duration-policy.test.mjs scripts/tests/verify-trusted-postmerge-ci-reuse.test.mjs` (9 passed)
- `python3 -m unittest scripts/tests/test_nextest_archive_workflow.py scripts/tests/test_ci_impact_workflow.py scripts/tests/test_codeql_workflow.py scripts/tests/test_adapter_diff_workflow.py scripts/tests/test_proptest_roundtrip_workflow.py scripts/tests/test_trusted_postmerge_ci_reuse_workflow.py` (83 passed)
- `node scripts/rust-test-suite-manifest.mjs --check`
- `git diff --check`

## 시각 증적

이 PR은 GitHub Actions workflow, CI 정책 스크립트와 계약 테스트만 변경한다. 문서 렌더링 또는 Studio 화면 동작을 변경하지 않으므로 visual sweep 대상이 아니다.

## 결론

로컬 계약 검증은 통과했다. 최신 head의 필수 CI가 성공하고, 첫 배포가 기대대로 전체 CI fallback으로 완료되는 것을 확인한 뒤 merge한다.
