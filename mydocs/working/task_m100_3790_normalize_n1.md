# #3790 N1 — 실행 경계·재사용 계약 조사

- Issue: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- 수행계획: [정상화 계획](../plans/task_m100_3790_normalize.md)
- 일자: 2026-09-14 KST
- 브랜치: `task_m100_3790_normalize`
- 승인된 계획 커밋: `937e90f00`
- 조사 제품/CI tree: `922946438fd89346a022df82e76648f88472f0cc`와 동일. 제품·CI·정식 테스트 변경 없음.
- 원격 재조회: devel `922946438fd89346a022df82e76648f88472f0cc`, main
  `cac9b4f7cc743535cd7c00fe4f286abd67e7145b`. 원격 변경 작업 없음.

## 1. 확인 결과

### 1.1 버전 계약 회귀

`statusDescription()`은 `POLICY_VERSION=6`을 발행하지만 CI·CodeQL·Render Diff의
`hasTrustedReviewReuse()`는 v5를 요구한다. `test_ci_impact_policy_workflow.py`도 v5 문자열을
고정하므로 기존 테스트는 통과한다. 영향 범위는 CI 실행 정책 변경 PR의 후행 review-only 재사용이다.

로컬 진단 [n1-contract-probe.cjs](../../output/3790/n1-contract-probe.cjs)는 현재 파일에서 실제
세 소비 함수를 추출하고, 실제 발행 함수의 출력에 정상 GitHub API 모의 응답을 연결했다.
실행: `node output/3790/n1-contract-probe.cjs`.

| 소비자 | 실제 v6 출력 수용 | 버전만 v5로 바꾼 대조군 수용 |
| --- | --- | --- |
| CI | false | true |
| CodeQL | false | true |
| Render Diff | false | true |

이는 네트워크 없는 계약 재현이며 실제 GitHub에서 절약된 시간 또는 live fast-pass 성공은 아니다.
진단 스크립트는 임시 output에 두고 PR에는 넣지 않는다. N2에서 정식 회귀 테스트로 구현한다.

### 1.2 PR 대상 변경 경계 누락

현재 `Resolve live pull request identity`의 PR 초기 이벤트 경로는 open/head SHA만 확인한다.
`Publish`도 open/head SHA만 다시 확인하며 live base ref/SHA를 검사하지 않는다.
이벤트 발생 시 devel PR이었으나 처리 전에 main으로 retarget된 경우를 실제 inline 코드로 재현했다.

| 지점 | 모의 입력 | 현재 결과 | 요구 결과 |
| --- | --- | --- | --- |
| resolve | 이벤트 base=devel, live base=main, 동일 head, 다른 base SHA | active=true, base_ref=main | active=false, trusted checkout·감사 금지 |
| publish 직전 | devel 기준 결과 계산 후 live base=main, 동일 head | createCommitStatus 호출 1회, success | 게시 0회, 비대상 사유 |

실제 main PR에 잘못 게시된 사고를 확인했다는 의미는 아니다. race 입력에 대한 코드 결함 재현이다.
게시 직전 live devel base SHA도 원래 계산 기준과 같아야 한다. 다른 기준의 결과로 최신 상태를
덮어쓰거나 스스로 무한 재실행하지 않는다.

### 1.3 일반 main 이벤트와 정보 없는 이벤트

현재 `pull_request_target.branches=[devel]`은 이미 main 대상 PR을 제외한다. 그러나
`workflow_run`의 job 조건은 `event=pull_request`만 확인하므로 main PR CI 완료도 runner에 진입할
수 있다. 이후 resolve의 `base: devel` 목록 검색에서 보통 제외된다. 즉 **불필요한 진입**과
**잘못된 정책 게시**는 서로 다른 문제이며 기존 코드를 모두 오동작으로 판정하지 않는다.

[실제 run 34800361317](https://github.com/edwardkim/rhwp/actions/runs/34800361317)의 API 응답은
event=pull_request, source branch=`dependabot/npm_and_yarn/rhwp-chrome/devel/vite-8.3.0`,
연결 PR #7125의 base=devel이었다. source branch를 devel로 제한하면 이 정상 경로가 빠진다.
REST run metadata 확인이며 webhook payload 원본을 저장한 것은 아니다.

GitHub 공식 [trigger 필터 문서](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)에 따르면
PR branch 필터는 대상 브랜치, workflow_run branch 필터는 선행 workflow가 실행된 브랜치에 적용된다.
따라서 `workflow_run.branches: [devel]`로 해결하지 않는다.

초기 job 조건에서 연결 PR 정보로 main 대상임이 명확하면 job을 skip한다. 정보가 없거나 불완전하면
한정된 live PR 식별을 수행하고, 정확한 open devel PR이 아니면 checkout·수집·감사·게시 전에 종료한다.
run 생성 자체, skipped job, 신원 확인만 수행한 runner, 실제 감사 실행을 구분해서 보고한다.
GitHub 이벤트 구독을 유지하는 설계에서 모든 main 관련 workflow 기록 자체의 부재는 보장하지 않는다.

### 1.4 기존 수정과 운영 배포

main의 Controller YAML은 아직 `cancel-in-progress: true`다. devel에는 #6819의 이벤트 제한과
#7069의 제한된 증거 재조회 배선이 있다. 이번에는 그 구현을 다시 만들지 않는다.
동일 head의 감사끼리 취소하지 않는 계약을 유지하고 retarget/stale 결과의 게시를 차단한다.
현재 concurrency group은 head SHA 기준이므로 새로운 head의 실행이 옛 head 실행을 반드시
취소한다는 보장은 하지 않는다. latest identity 검사로 옛 결과 게시를 막는다.

[GitHub workflow_run 문서](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run)의
default-branch 등록 경계 때문에 devel 병합과 main YAML 활성화를 구분해야 한다.
기존 main 직접 변경이나 릴리즈 생성은 이번 승인 범위가 아니다.

## 2. baseline 검증

다음은 **수정 전 기준선**이며 수정 후 성공 증거가 아니다.

```bash
node --test scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs \
  scripts/tests/ci-workflow-evidence.test.cjs \
  scripts/tests/ci-impact-report.test.cjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  scripts.tests.test_ci_impact_policy_workflow scripts.tests.test_ci_impact_workflow \
  scripts.tests.test_codeql_workflow scripts.tests.test_render_diff_workflow \
  scripts.tests.test_review_only_fast_pass_workflows
```

- Node: **136 PASS / 0 FAIL**, 201.5ms.
- Python: **99 PASS**, 9.789초.
- 별도 실제 inline 코드 모의 실행: 버전 불일치 3개 소비자 및 retarget 누락 2지점 재현.
- PyYAML 6.0.1 사용 가능. `actionlint`는 PATH에서 발견되지 않아 미실행.
- Rust/WASM/Studio/전체 GitHub CI는 변경 범위 밖이므로 실행하지 않았다.

## 3. 다음 단계

[구현계획](../plans/task_m100_3790_normalize_impl.md) 승인을 요청한다.
N2에서 발행/소비 연결·resolve/publish 실행형 테스트를 먼저 추가해 수정 전 실패를 고정한다.
이후 최소 코드 변경 및 기존 수렴/보고/선택 실행의 보호 테스트를 수행한다.
