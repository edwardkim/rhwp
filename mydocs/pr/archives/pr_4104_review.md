---
kind: pr_review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4104 검토 - Native Skia와 기본 테스트 shard 의존성 분리

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| PR / 이슈 | [#4104](https://github.com/edwardkim/rhwp/pull/4104) / [#4103](https://github.com/edwardkim/rhwp/issues/4103) |
| code head | `a86ae08b58ac3dcfdcffac9a0195bab6035a5998` |
| base | `0102a7ae2be8c06ada3636326c82858de890cef9` |
| 브랜치 | `task/4103-native-skia-shard-parallel` |
| 대상 경로 | `.github/workflows/ci.yml`, `scripts/tests/test_ci_impact_workflow.py` |
| 시각 검증 | 비대상. CI scheduling 및 workflow 계약만 변경한다. |

기존 네 기본 테스트 shard는 `native-skia-tests`를 `needs`와 실행 조건에 함께 넣어, 각자가
소비하는 test archive가 준비돼도 Native Skia가 성공할 때까지 시작하지 못했다. Native Skia는
shard에 artifact나 실행 결과를 전달하지 않는다.

## 변경과 안전 계약

각 shard의 선행 job은 `preflight`와 자신이 받는 archive로만 축소했다. `native_skia_required`와
`native-skia-tests` 결과 분기도 shard 조건에서 제거했다. 반면 `native-skia-tests`는 기존 독립 검증
job으로 남고, `Build & Test` final aggregate의 `needs` 및 Native Skia 결과 검증은 유지한다. 따라서
Native Skia 실패·취소는 여전히 PR 성공으로 집계되지 않는다.

회귀 테스트는 네 shard가 정확히 자신의 archive만 기다리고 Native Skia 문자열을 포함하지 않는지,
Native Skia job이 archive·shard를 기다리지 않는지, final aggregate가 Native Skia를 계속 필요로 하는지를
고정한다.

## 검증

| 검증 | 결과 |
| --- | --- |
| workflow 계약 | `python3 scripts/tests/test_ci_impact_workflow.py` 18 passed |
| review-only fast-pass 계약 | `python3 scripts/tests/test_review_only_fast_pass_workflows.py` 4 passed |
| workflow 정적 검사 | `actionlint .github/workflows/ci.yml` 통과 |
| 공백 검사 | `git diff --check` 통과 |
| GitHub CI | [CI 31092097921](https://github.com/edwardkim/rhwp/actions/runs/31092097921?pr=4104) 전체 성공 |
| CodeQL | [CodeQL 31092097642](https://github.com/edwardkim/rhwp/actions/runs/31092097642?pr=4104) 전체 성공 |

원격 실행에서 Lint는 10:13:07에 끝났고, Native Skia와 archive A/B/slow는 10:13:10~12에 함께
시작했다. Native Skia는 10:18:16에, archive B는 10:18:56에, slow는 10:19:06에 완료됐다. archive
B/slow 완료 직후 shard 3·slow·2가 10:19:06~09에 시작했으며, archive A 완료(10:21:52) 뒤 shard 1도
10:21:55에 시작했다. 이번 실행에서는 Native Skia가 archive보다 먼저 끝나 시간상 역전 사례는 없었지만,
workflow graph 의존성과 정적 회귀 계약에서 shard의 Native Skia 선행 관계가 제거됐음을 확인했다.

## 수용 판단

**수용.** CI·CodeQL과 final `Build & Test` aggregate가 성공했고, Native Skia 결과의 최종 실패 집계는
보존됐다. 이 review-only 기록을 push한 뒤 fast-pass aggregate와 최신 mergeability를 확인하고 병합한다.
