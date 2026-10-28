# PR #6253 검토 기록

## 대상

- PR: [#6253](https://github.com/edwardkim/rhwp/pull/6253)
- 제목: \`perf(ci): nextest archive를 A/B/C/D로 균형 분할\`
- 기준 브랜치: \`devel\`
- 최종 head: \`a7afced031d5abd251c10141094671273a234a39\`
- merge commit: \`bb9e76c738ee04019cbbd5130edecbb09a42d343\`
- 관련 이슈: [#6251](https://github.com/edwardkim/rhwp/issues/6251)

## 변경과 판단

- Archive A는 두 hash shard를 단일 \`hash:1/1\` worker로 통합했다.
- 41개 integration target은 duration 정책의 longest-processing-time 배정으로 B/C/D에
  14개, 13개, 14개씩 분배했다.
- 각 worker의 추정 시간은 B 947.948초, C 947.937초, D 948.171초로 정렬됐다.
- 실제 CI worker는 A 3,889건, B 1,427건, C 1,427건, D 1,531건을 실행했고
  모두 성공했다. B/C/D의 실제 경과 시간 편차는 새 측정 정책의 초기 기준이므로,
  성공한 \`devel\` 실행의 JUnit 측정이 다음 정책 갱신에 반영될 때 보정 대상이다.

## 로컬 검증

- \`node --test scripts/tests/nextest-target-duration-policy.test.mjs\` (7 passed)
- \`node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-policy.test.cjs\` (65 passed)
- \`python3 -m unittest scripts/tests/test_nextest_archive_workflow.py scripts/tests/test_ci_impact_workflow.py scripts/tests/test_ci_impact_policy_workflow.py scripts/tests/test_workflow_contract_wiring.py\` (59 passed)
- \`python3 -m unittest scripts/tests/test_adapter_diff_workflow.py scripts/tests/test_proptest_roundtrip_workflow.py scripts/tests/test_nextest_archive_workflow.py scripts/tests/test_workflow_contract_wiring.py\` (37 passed)
- \`node scripts/rust-test-suite-manifest.mjs --prepare && cargo fmt --all -- --check\`
- \`git diff --check\`

## CI 보정과 결과

- 첫 head는 CI 영향 정책 감사에 Archive D builder/worker와 새 REST job 표시명이 빠져
  fail-closed로 실패했다. 정책 allowlist와 alias를 동기화했다.
- 다음 head는 Proptest roundtrip의 A 2-shard 고정 계약이 실패해 A/B/C/D 네 worker
  계약으로 갱신했다.
- 마지막으로 Adapter Diff에도 같은 고정 계약이 남아 있어 동일하게 갱신했다.
- 최종 [CI run 33082308666](https://github.com/edwardkim/rhwp/actions/runs/33082308666)은
  Lint, Native Skia, CodeQL, B/C/D archive build와 A/B/C/D worker, Build & Test를
  모두 성공으로 완료했다. PR 실행이므로 duration data refresh는 의도대로 skipped였다.

## 시각 증적

이 PR은 CI workflow, 분배 정책 스크립트, 계약 테스트만 변경한다. 문서 렌더링 및 Studio UI의
출력이 바뀌지 않으므로 visual sweep 대상이 아니다.

## 결론

최종 head의 필수 CI와 분배 coverage를 확인한 뒤 squash merge했다. merge 후 확정 기록은 이
문서와 오늘할일을 담은 별도 docs-only PR로 보존한다.
