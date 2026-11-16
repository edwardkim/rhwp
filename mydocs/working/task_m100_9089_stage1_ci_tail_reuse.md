# Issue #9089 — 검증 완료한 테스트 목록 뒤 문서 tail 재사용

## 문제와 선행 근거

- [Issue #9089](https://github.com/edwardkim/rhwp/issues/9089).
- base: `e7ee272cfa33a960b0d0d924983d29e6509903e0`.
- 작업 브랜치: `task/9089-validated-review-tail-reuse`.
- 원 구현: [PR #6200](https://github.com/edwardkim/rhwp/pull/6200),
  `abeb29ca2fdbad2983d5562af0da75b57663449f`. current-base bridge와 실행 영역 변경의
  fail-closed 정책을 함께 도입했다. 파일 목록 완전성의 후속 보정도 유지한다.
- 실제 반례: [PR #9088](https://github.com/edwardkim/rhwp/pull/9088)의
  `0a8afd62bc955c8125e4238c9afc4117eeb4e683`에서 전체 검증을 마친 뒤
  `abbf3e21bf67c647ec633141e245cf389507ab4c`에 리뷰·오늘할일만 추가했다.
  Proptest·Adapter는 PR 전체에 남은 `scripts/rust-test-suite-manifest.mjs`의
  helper 소비자 한 줄을 실행 영역 변경으로 보고 재실행했다.
- `0a8afd62…` 자체는 보호 하한의 출처를 연결한 문서 커밋이며, 이 SHA에서 실제 full worker가
  성공했다. 마지막 제품 코드 변경은 `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`이다.
  기존 후보 탐색은 코드 이후 문서 head의 full 성공도 재사용할 수 있으나, 실행 영역의 PR 전체
  가드가 탐색 전에 종료해 이 근거를 사용하지 못했다.

## 구현 경계

1. workflow·runner·harness 정의와 테스트 목록 입력을 구분한다. 실행 정의가 변경된 PR의
   기존 전면 차단은 유지하며, 목록 변경을 문서로 분류하지 않는다.
2. 성공 후보 뒤의 단일 부모 문서 tail과 동일 PR source repository·branch·event·SHA를 확인한다.
   목록을 변경한 코드 자체는 full 실행 대상이다. 그 뒤 문서만 바뀐 경우에만 재사용한다.
3. 목록 변경이 있는 경우 workflow의 녹색 상태만으로 승인하지 않는다. 같은 run의 최신 attempt에서
   실제 worker가 성공했는지 확인한다. skip된 문서 후보는 앞선 실제 worker 성공을 찾아간다.
4. 후보 이후 실행 입력 변경, 실행 정의 변경, rename 우회, 실패·누락·진행 중·다른 source 후보,
   불완전한 파일 목록·worker 증거는 full로 처리한다. 기존 merge-tree bridge 검증은 유지한다.
5. 두 workflow는 같은 worker 증거 판정 코드를 사용하고, 실제 inline JavaScript를 실행하는
   기존 CI 계약 테스트 경로에 반례를 추가한다. 새로운 중복 CI 호출은 추가하지 않는다.

## 검증과 제출 경계

- #9088의 관련 API 자료 구조를 재현한 검사에서 수정 전 두 경로의 FAIL을 먼저 확인한다.
- 수정 후 정상 재사용과 위 거부 경계를 함께 실행한다. 기존 Python workflow/bridge 계약,
  Node classifier·policy·controller 계약과 workflow 구문을 확인한다.
- 제품 Rust·조판·WASM 및 worker 명령은 바꾸지 않는다. 따라서 제품 전체 빌드나 Visual Sweep은
  이 변경의 검증 범위가 아니다. CI 권한·event·안정적인 check 이름을 유지한다.
- 로그·임시 자료는 `output/issue9089/`에만 저장하고, 단계 결과를 이 문서에 보완해 코드와 커밋한다.
- workflow·판정 정책 변경이므로 제출 후 운영 기록은 옵션 2를 적용한다. 원 PR에 문서 trailing을
  추가해 전체 CI를 반복하는 방법으로 새 동작을 시험하지 않는다.

## 구현 결과

- 두 workflow의 `isAllowedReviewPath`에서 실행 정의·목록 경로와 `previous_filename`을 확인한다.
  목록을 문서로 간주하지 않으며 문서 경로로 rename하는 우회도 차단한다.
- workflow·runner·harness 정의는 기존 PR 전체 가드로 full 실행을 유지한다. 테스트 목록 입력의
  수정은 기존 역방향 계보 검사 뒤 실제 worker 성공을 추가 확인하는 경로로 보낸다.
- `candidateWorkerState`는 두 workflow에서 같은 본문을 사용한다. `filter=latest`로 job 전체를
  조회하고, 안정적인 worker 이름이 정확히 하나이며 run ID·attempt·head SHA가 일치하고
  completed/success인 경우만 수용한다. skipped는 앞선 후보를 계속 탐색하며 실패·누락·중복·
  API 오류는 full로 되돌린다. base bridge의 별도 트리 검사는 생략하지 않는다.
- 기존에 CI가 호출하는 `ci-impact-controller-contract.test.cjs`에 실제 YAML inline script를
  실행하는 검사를 추가했다. 별도 CI 호출·권한·event·worker 명령·check 이름은 변경하지 않았다.
- 운영 정본 `review_only_fast_pass.md`에 목록 입력과 실행 정의의 차이 및 실제 worker 증거 조건을
  반영했다. CI 정책 PR의 후속 기록은 옵션 2로 유지한다.

## 검증 결과

검증 환경은 `ubuntu-ted`, 시각은 2026-11-15 KST다. 아래 명령은 저장소 루트에서 실행했다.

| 검사 | 결과 | 로그 |
| --- | --- | --- |
| 실제 preflight를 실행하는 초기 재현 2건 | 수정 전 2 FAIL: 기존 실행 영역 가드가 두 경로를 차단 | `output/issue9089/worker-tail-before.log` |
| `node --test scripts/tests/ci-impact-classifier.test.cjs scripts/tests/ci-impact-controller-contract.test.cjs scripts/tests/ci-impact-policy.test.cjs scripts/tests/ci-impact-report.test.cjs scripts/tests/ci-workflow-evidence.test.cjs` | 206 PASS, 실패·skip 0 | `output/issue9089/node-ci-contracts.log` |
| `python3 -m unittest scripts.tests.test_adapter_diff_workflow scripts.tests.test_proptest_roundtrip_workflow scripts.tests.test_review_only_fast_pass_workflows scripts.tests.test_ci_impact_workflow scripts.tests.test_codeql_workflow scripts.tests.test_workflow_contract_wiring` | 152 PASS | `output/issue9089/python-workflow-contracts.log` |
| `actionlint .github/workflows/adapter-diff.yml .github/workflows/proptest-roundtrip.yml` | 종료 코드 0, 진단 0 | `output/issue9089/actionlint.log` |
| #9088 실제 REST 응답 재생 | 변경 195개·커밋 41개에서 수정 전 두 경로 full → 수정 후 두 경로 재사용 | `output/issue9089/pr9088-api-replay.log` |
| `node --test --test-name-pattern='#9089' scripts/tests/ci-impact-controller-contract.test.cjs` | 신규 19 PASS | `output/issue9089/worker-tail-after.log` |
| `python3 scripts/check_markdown_links.py mydocs/manual/pr_review/review_only_fast_pass.md mydocs/working/task_m100_9089_stage1_ci_tail_reuse.md` | 문서 2개, 내부 링크 오류 0 | `output/issue9089/markdown-links.log` |
| `python3 scripts/check_document_metadata.py`와 base의 동일 검사 결과 대조 | 전체 663개에서 기존 오류 20건 그대로, 신규 오류 0. 변경 문서는 오류 0 | `output/issue9089/document-metadata.log`, `output/issue9089/document-metadata-comparison.log` |
| `git diff --check` | 종료 코드 0 | 커밋 전 확인 |

합성 경계 검사에는 신규 목록 수정 이후 이전 성공 거부, 같은 SHA의 최신 실패 거부, 다른 source·
event·시각 거부, 실제 job의 skip·누락·중복·실패·attempt/SHA 불일치·API 오류, 목록 잘림·계보
불일치·실행 파일 rename 차단과 current-base bridge의 트리 검증 대기가 포함된다.

실제 API 재생은 완료된 원 PR을 변경하지 않는 로컬 검사다. 원 응답은
`output/issue9089/`에만 저장했다. F의 실행 전 상황을 재현하기 위해 성공한 E run만 공급했으며,
나중에 성공한 F run을 소급 사용하지 않았다. 검증된 worker는 다음과 같다.

| 경로 | E의 실제 성공 worker | 수정 전 | 수정 후 |
| --- | --- | --- | --- |
| Proptest | [run 43680269699 / job 132709227926](https://github.com/edwardkim/rhwp/actions/runs/43680269699/job/132709227926) | `proptest-execution-surface-change` | `fast_pass=true`, 후보 `0a8afd62…` |
| Adapter | [run 43680269627 / job 132709222230](https://github.com/edwardkim/rhwp/actions/runs/43680269627/job/132709222230) | `adapter-execution-surface-change` | `adapter_required=false`, 후보 `0a8afd62…` |

현재 상태는 로컬 구현·해당 범위 검증 완료다. 원격에 새 workflow를 적용한 실제 Actions 실행은
아직 하지 않았다. 제품 실행 코드·저장·조판 경로와 worker 명령이 같으므로 Cargo·WASM·Studio·
Visual Sweep 전체 재실행은 이 CI 정책 변경의 로컬 검증 대상이 아니다.

## 적용과 되돌리기

2026-11-15 KST에 `git fetch upstream devel`로 확인한 base는 위 SHA와 동일했다.
`jangster77` 계정의 열린 PR은 0건이었다. 조회 원본은 `output/issue9089/own-open-prs.json`,
fetch 로그는 `output/issue9089/base-fetch.log`에 있다. PR 생성 직전에는 다시 확인한다.

push·새 PR 생성 전에 base/head·변경 범위·검증·되돌리기를 보고하고 해당 원격 승인을 받는다.
새 PR의 실제 Full CI와 preflight 결과를 확인하기 전에는 원격 적용 완료나 Issue #9089 해결
완료로 보고하지 않는다. 되돌리기는 이 단계의 변경 커밋을 일반 revert하여 두 workflow의
목록 변경 전면 차단으로 복원한다. 권한·설정·보호 브랜치의 직접 변경은 필요하지 않다.
