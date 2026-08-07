---
kind: reference
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4037 검토 기록 — 에이전트 자동화 도구킷

## 메타

| 항목 | 값 |
| --- | --- |
| 원 PR | [#4037](https://github.com/edwardkim/rhwp/pull/4037) |
| 작성자 | `kevin9327` |
| base | `devel` |
| contributor 원 head | `48b41ca9c9303967d6333d2f161c9b798b6acba0` |
| 메인터너 보정 | `6f8ee6d84`, `6049d2919` |
| 검증한 code head | `6049d291915cf0eca403b61cdc2b3bcb7a3421dc` |
| 규모 | 12 파일, +1,615/-0; 대형 PR(1,000줄 초과) |
| 검토 branch | `review/kevin9327-4037-20260806` |
| 최신 devel | `d722c1161e03a0bf58beebc7b0d9c638f7a52b8b` (병합 시뮬레이션 기준) |
| 원격 검증 | [CI 31068823951](https://github.com/edwardkim/rhwp/actions/runs/31068823951), [CodeQL 31068823815](https://github.com/edwardkim/rhwp/actions/runs/31068823815) 성공 |
| 검증 뒤 PR 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |
| 기준선 정렬 merge | `df3aefd70` (`ca77f8ba9` + `d722c1161`), 오늘할일 한 파일의 기록을 함께 보존 |
| 기준선 병합 head 검증 | [CI 31069850597](https://github.com/edwardkim/rhwp/actions/runs/31069850597), [CodeQL 31069850507](https://github.com/edwardkim/rhwp/actions/runs/31069850507) 성공 |

원 head는 최신 `devel`의 조상이 아니었으나, 최신 `devel` 위 병합 시뮬레이션은 충돌 없이
구성됐다. 렌더러·레이아웃·fixture·기준 PDF 변경은 없으므로 시각 증적 경로는 적용하지 않았다.

code CI 완료 뒤의 첫 archive review commit `ca77f8ba9`는 최신 `devel`의 같은 날 오늘할일 추가와만
충돌했다. `df3aefd70`은 현재 `devel`을 정확한 두 번째 parent로 두고 양쪽 오늘할일 기록을 보존한
update-branch merge다. contributor 원 commit은 rewrite하지 않았다.

첫 fast-pass preflight는 `67aedc594`의 Build & Test가 아직 실행 중이고 update-branch merge
`df3aefd70`에는 독립 CI가 없어서 green candidate를 찾지 못했다. 따라서 기준선 병합 head 전체 CI를
정상 실행했고, Build & Test·Native Skia·Lint·package gate·slow/regular shard와 CodeQL 분석이 모두
성공했다.

## 발견 사항과 보정

| 우선순위 | 발견 사항 | 메인터너 보정 |
| --- | --- | --- |
| 높음 | `form_filling`과 `table_harvest`가 검증 실패 때 기존 `-o` 산출물도 삭제할 수 있었다. | 공통 출력 충돌 검사를 추가하고, 이번 호출이 새로 확보한 경로만 정리하도록 변경했다. |
| 높음 | `bulk_sweep`이 레코드 없이 `batch` 프로세스가 실패해도 exit 0을 보고했다. | 레코드 없는 비정상 batch 종료를 `batchFailures`로 기록하고 exit 1로 승격했다. |
| 중간 | `archive_search --report`가 최종 `exit`를 넣기 전에 파일을 저장했다. | 종료 판정 뒤에 보고서를 저장하고 batch 종료 코드·stderr를 함께 기록했다. |
| 중간 | 도구킷의 실제 rhwp 회귀 21건이 GitHub CI에서 실행되지 않았다. | 단순 보조 도구이므로 핵심 CI에는 추가하지 않고, 메인터너 보정 때 실제 `rhwp`로 27건을 로컬 실행했다. |
| 낮음 | PR 본문은 구현·테스트가 후속 PR이라고 설명하지만 실제 head에는 구현과 회귀가 포함돼 있었다. | 최종 PR 본문과 병합 후 comment에는 실제 구현·검증 범위를 명시한다. |

출력 충돌 계약은 모든 워크플로에 일관되게 적용했다. 기존 파일·보고서·계획된 CSV·NDJSON이
있으면 exit 2로 중단하며, 기존 `bulk_sweep` 폴더는 비충돌 파일에 한해 계속 사용할 수 있다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 최신 devel 병합 시뮬레이션 | 충돌 없음 |
| `CARGO_TARGET_DIR=target/review-kevin9327-4037-20260806 CARGO_INCREMENTAL=0 cargo build --profile release-test --bin rhwp` | 통과 |
| `RHWP_BIN=…/rhwp python3 tools/agent-toolkit/tests/test_workflows.py` | 27건 통과 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 27건 통과 |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` | 18건 통과 |
| `python3 -m py_compile …` 및 `git diff --check` | 통과 |
| GitHub CI `31068823951` | Build & Test, Native Skia, Lint, package gate와 3개 regular/slow shard 모두 성공 |
| GitHub CodeQL `31068823815` | JavaScript/TypeScript, Python, Rust 분석 모두 성공 |

## 최종 권고

**수용.** 메인터너 보정 commit을 포함한 code head의 로컬 검증과 GitHub CI·CodeQL이 모두 성공했다.
보정 범위는 기존 출력 보호, batch 실패 계약, 보고서 계약, 로컬 회귀 보강으로 한정했다.

이 commit은 전체 CI가 성공한 `67aedc594` 뒤의 단일 부모 review-only 기록이다. push 뒤에는
`67aedc594`을 candidate로 재사용한 fast-pass aggregate가 성공하고 최신 PR 상태가 `CLEAN`인지 확인한 뒤
작업지시자 승인에 따라 병합한다. 병합 comment에는 보정이 기존 산출물 삭제와 batch 실패 은닉을 막기 위해
필요했던 이유를 명시한다.

보정 실행과 rollback은 [PR #4037 메인터너 보정 실행 기록](pr_4037_review_impl.md)을 따른다.
