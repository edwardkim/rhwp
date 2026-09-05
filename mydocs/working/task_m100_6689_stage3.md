# #6689 Stage 3 — 실제 실행 경로·배포 안전 경계

- Issue: #6689
- Plan: `mydocs/plans/task_m100_6689.md`
- Branch: `task_m100_6689`
- Baseline: `upstream/main@e8800c8def63449808a4092798442652ed460552`
- Implementation candidate: `e483e940a794f8abf50b1ac1e3366fa684da6277`
- Status: active — 원격 exact-head 실행 증적 대기
- Date: 2026-09-05 KST

## 1. 로컬 구현 완료 범위

Stage 2에서 승인된 수정 계획에 따라 자동 diff와 운영 정책을 분리했다.

1. `scripts/workflow_promotion_policy.json`에 baseline 8개 workflow의 실행 mode, 민감 표면, 필수 성공·skip
   job, artifact, verdict, 허용 event·actor를 명시했다.
2. `apply_execution_policy()`는 policy SHA-256을 inventory에 결합하고 모든 executable entry에 정책이
   없으면 `missing-workflow-policy`로 실패하도록 했다.
3. verifier는 run URL의 repository·run ID, 실행 mode, required skipped job, 일반 artifact와 구조화된
   verdict artifact의 hash·파일·판정을 검증한다.
4. Pages는 전역 권한을 `contents: read`로 축소하고 `pages: write`·`id-token: write`를 `Deploy` job에만
   두었다. `Deploy`는 `main` push에서만 실행되므로 수동 dispatch는 Build와 Pages artifact만 만든다.
5. Gym 수동 dispatch의 기본 mode를 `contracts`로 두고, `mode=full`을 명시할 때만 전건 benchmark가
   실행된다. checkout 두 곳과 Rust toolchain은 full commit SHA로 고정했다.
6. Oracle advisory는 skip·build failure·compare failure·pack failure·completed를 구조화한
   `verdict.json`을 항상 생성하고 별도 `oracle-public-advisory-verdict` artifact로 올린다. promotion
   증적으로 인정되는 판정은 `completed`뿐이다.

로컬 구현 commit:

```text
e483e940a794f8abf50b1ac1e3366fa684da6277
ci: add workflow promotion execution modes (#6689)
```

## 2. 실행 mode와 증적 계약

| workflow | mode | 필수 성공 job | 필수 skip·artifact |
| --- | --- | --- | --- |
| Adapter inter-diff | direct | preflight, adapter inter-diff | 없음 |
| CI | direct | CI preflight, Build & Test | 없음 |
| CodeQL | direct | preflight, 3-language Analyze | 없음 |
| Deploy Pages | verify-only | Build | Deploy=skipped, `github-pages` |
| Gym | contracts-only | Gym benchmark contracts | Full Gym benchmark validation=skipped |
| Oracle advisory | direct | oracle-public-compare-advisory | verdict artifact=`completed` |
| Proptest roundtrip | direct | Proptest preflight, prop roundtrip | 없음 |
| Render Diff | direct | Render Diff preflight, Canvas visual diff | `render-diff-artifacts` |

일반 pre-main 증적 event는 `workflow_dispatch`, actor는 `edwardkim`으로 제한한다. Oracle은 등록되지 않은
workflow identity를 bootstrap하기 위해 workflow 파일 자체가 바뀐 신뢰 branch의 `push`도 허용한다. mode
문자열만 믿지 않고 Pages와 Gym의 금지된 job이 실제로 `skipped`인지 함께 검증한다.

## 3. 로컬 검증

```text
python3 -m unittest \
  scripts/tests/test_workflow_promotion_preflight.py \
  scripts/tests/test_gym_benchmark_validation.py \
  scripts/tests/test_oracle_public_advisory_workflow.py
Ran 35 tests
OK
```

- 수정 workflow 3개의 YAML parse: 통과
- execution policy JSON parse: 통과
- Python syntax compile: 통과
- `git diff --check`: 통과
- 기존 Gym 계약 한 건은 “모든 수동 실행이 full”인 과거 조건을 “명시적 `mode=full`만 full”로
  현행화했다.

## 4. exact local candidate inventory

`upstream/main → e483e940a`에 policy를 결합해 두 번 생성한 canonical JSON은 byte-identical이다.

| 항목 | 값 |
| --- | --- |
| policy SHA-256 | `3147abf441da77d3d1362bdf26d99e31511a09d05c532f9bfaf4939796ff1d83` |
| inventory SHA-256 | `0115253a66e7aa50a4e271188e566feaa7f914239f522425c972d7e97d848a14` |
| executable workflow | 8개 |
| missing policy | 0건 |
| action pin policy violation | 0건 |

Pages는 이번 수정으로 `permissions`·`deployment`, Oracle은 `action-ref`·`artifact`까지 실제 변경 축으로
검출됐다. 이는 Stage 2에서 기존 민감 표면과 변경 축을 분리한 효과다.

## 5. 원격 정합과 남은 Stage 3 게이트

2026-09-05 확인 결과 `upstream/devel`은 기준선
`1c49df3d33a323d459c8e90517f4a0f5bd3c790b` 그대로이고 task branch는 4 commit ahead, 0 behind다. 열린
PR 중 `.github/workflows/**` 또는 `.github/actions/**`를 변경하는 PR은 없다.

남은 순서는 다음과 같다.

1. 문서 commit까지 포함한 새 exact candidate를 원격 task branch에 push한다.
2. 별도 승인 뒤 8개 workflow를 candidate ref에서 dispatch한다. 서로 독립인 run은 병렬 실행하되
   repository runner 비용·concurrency를 관찰한다.
3. run path, event, actor, head SHA, workflow content hash, 전체 job pagination과 artifact를 snapshot으로
   수집한다.
4. Pages `Deploy`와 Gym full job의 skip, Oracle verdict=`completed`, 나머지 필수 job의 success를 offline
   verifier로 판정한다.
5. 하나라도 누락·실패하면 Stage 3를 닫지 않고 원인과 재실행 범위를 분리한다.

push와 workflow dispatch는 이 문서의 로컬 결과 승인과 별도로 각각 메인테이너 승인을 받는다.

## 6. 원격 카나리 결과와 Oracle 경로 정정

원격 candidate `a1a199e96697aab43508d5d06c625d1ffec95f23`에서 먼저 세 개의 카나리를 순차
실행했다.

| workflow | run | 결과 |
| --- | --- | --- |
| Deploy Pages | `33940406533` | Build=success, Deploy=skipped, `github-pages` artifact 생성 |
| Gym | `33940407609` | contracts=success, full benchmark=skipped |
| Oracle advisory | run 없음 | workflow dispatch endpoint HTTP 404 |

Oracle 파일은 기본 브랜치 `main`에 존재하고 YAML·actionlint 검사를 통과했지만 Actions API가 반환한
27개 workflow identity에는 포함되지 않았다. 나머지 다섯 workflow는 카나리 실패 즉시 중단하여 실행하지
않았다.

이 발견에 따라 Oracle에만 다음 bootstrap을 추가한다.

- branch: `devel`, `task_m100_*`
- path: `.github/workflows/oracle-public-advisory.yml` 단일 경로
- event: `push`
- permissions: `contents: read`
- dispatch input이 없는 push fallback: `top_n=10`, `limit=0`

일반 소스 push와 PR에서는 새 runner 비용이 생기지 않는다. 정정 commit은 candidate SHA를 바꾸므로 위 두
성공 run은 mode·권한 경계의 관찰 근거일 뿐 최종 exact-head 증적으로 재사용하지 않는다. 새 candidate를
push한 뒤 Oracle push run을 카나리로 확인하고, 성공할 때만 여덟 workflow의 새 exact-head 증적 수집을
재개한다.

정정 구현의 로컬 검증 결과는 다음과 같다.

- Stage 3 focused Python 계약: 36건 통과
- Oracle workflow YAML parse: 통과
- actionlint 1.7.12: 기존 candidate에도 있던 `SC2016` 정보 진단 한 건을 제외하면 통과
- execution policy JSON parse·Python compile·`git diff --check`: 통과
- 정책 event 전수 확인: Oracle만 `push,workflow_dispatch`, 나머지 7개는 `workflow_dispatch`
- `test_workflow_contract_wiring.py`: Stage 4에서 promotion 계약 테스트를 CI Lint job에 배선하기 전까지
  의도된 RED 2건 유지; 누락 항목은 `test_workflow_promotion_preflight.py` 한 파일뿐이다.
