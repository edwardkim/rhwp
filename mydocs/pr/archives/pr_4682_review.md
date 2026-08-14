---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-14
---

# PR #4682 self-review — 선택 실행 독립 정책 감사

## 결론

**보정 뒤 maintainer review 필요.** [PR #4682](https://github.com/edwardkim/rhwp/pull/4682)는
Stage 3~5에서 빨라진 CI·CodeQL·Render Diff가 필요한 job과 step을 실제로 실행했는지 default-branch
controller가 독립 검증하는 변경이다. 제품 기능이나 선택 실행 규칙을 새로 줄이는 PR이 아니라,
PR-controlled workflow가 필수 검사를 잘못 건너뛰면 `CI Impact Policy`를 실패시키는 안전망을 추가한다.

원격 head `f69856f4d`의 첫 전체 CI는 통과했다. 다만 maintainer 요청 전 self-review에서 외부 fork run
조회, 최신 run 선택, enforcement 변경 PR의 trailing fast-pass 정합 문제를 발견해 commit
`4ba5e431d`로 보정했다. 최신 `upstream/devel@9b9cbf3c8`도 merge head `30bbcf9fe`에 반영하고 로컬
focused 검증을 다시 통과했다. 첫 CI는 최종 merge 근거로 재사용하지 않으며, 새 head의 전체 CI와
`edwardkim`의 명시적 승인 뒤에만 merge를 권고한다.

2026-08-13 게시 self-review 코멘트의 유효한 후속 지적도 로컬에서 보정했다. Node policy 테스트를 실제
CI에 배선하고, 필요한 검사 누락만 차단하면서 안전한 full 상위 집합은 허용했다. 취소·stale controller의
status 게시를 막고 CI 감사 lane 완전성 계약을 추가했다. 최신 devel 동기화 head의 새 CI가 남아 있다.

2026-08-13 maintainer review는 완료된 workflow의 job 목록 API 조회가 재시도 뒤에도 실패하면 빈 목록이
실제 누락으로 오인되어 `failure` status를 게시하는 운영 결함을 지적했다. 이 finding을 수용해 기존
`retries: 3`은 유지하고, 조회 완료 여부를 별도 증거로 전달해 API 증거가 불완전한 경우만 `pending`으로
남겼다. 정상 조회 뒤 필수 job이 실제로 빠진 경우는 계속 `failure`다. 보정 code candidate는
`e1cb68ff0`이며 새 원격 head CI와 maintainer 재확인이 남아 있다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           multi_pr_update_branch.md, rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, multi_pr_update_branch.md,
                  rework_and_exceptions.md
remote head at intake: f69856f4d5101eec4d9a454f7db91e3bb8a18a22
local correction commit: 4ba5e431dd13500e4321d4e1eb0082c98b6004bf
latest devel merge head: 30bbcf9fe1d0b5a55c4635d3f0bca56ba79b26b9
review remediation base merge: d5b7ef8318a1ff790b9d4e55f02c1f126c980427
maintainer review head: 3b421b12d49e9ceb9c3c4499660ab950288e996f
latest correction base head: d31fa652f535decce7fb51f8551c3913b0d1a106
job evidence correction commit: e1cb68ff0
```

1,000줄이 넘고 default-branch privileged controller와 향후 required status 채택 판단을 포함하므로,
구현·재검증·maintainer 승인 순서는 [별도 이행 기록](pr_4682_review_impl.md)에 고정한다. 작업지시자의
승인 전에는 Draft 해제나 reviewer request를 만들지 않는다.

## 메타데이터

| 항목 | 2026-08-12 self-review 접수 시점 |
| --- | --- |
| PR | [#4682](https://github.com/edwardkim/rhwp/pull/4682) |
| 관련 이슈 | [#3790](https://github.com/edwardkim/rhwp/issues/3790) |
| 작성자 | `postmelee` |
| reviewer | 미지정; 보정·새 head CI 뒤 `edwardkim` 요청 예정 |
| base / head | `devel` / `issue-3790-stage26-enforcement` |
| remote head | `f69856f4d5101eec4d9a454f7db91e3bb8a18a22` |
| 원격 규모 | 9 files, +1,857 / -14, 2 commits |
| 상태 | Open, Draft, MERGEABLE / CLEAN; 첫 head checks 성공 |

## 변경 범위와 신뢰 경계

controller는 `pull_request_target`과 세 workflow의 `workflow_run` 완료 이벤트에서 동작한다. live PR의
base SHA에서 classifier와 policy 두 파일만 credential 없이 sparse checkout하고, PR head·merge ref나
artifact는 실행하지 않는다. Actions API에서 읽은 PR file과 workflow run/job/step metadata만 증거로
사용한다.

CI의 Rust archive·worker, Native Skia, frontend unit/package, CodeQL 세 언어의 analysis/skip step,
Render Diff와 Canvas 결과를 classifier 진리표와 대조한다. 세 workflow가 모두 완료되고 기대
`success|skipped` 조합과 일치할 때만 exact head의 `CI Impact Policy`를 success로 바꾼다. 미완료는
pending, workflow run 실패와 정상 job 조회 뒤의 실제 누락·중복·예상 밖 실행은 failure로 닫는다.
workflow run 목록 또는 job 목록 API가 재시도 뒤에도 실패한 경우는 증거 불완전 상태이므로 pending을
유지하며, 빈 job 목록과 구분한다.

제품 Rust·TypeScript·renderer 코드와 기존 worker 선택 진리표는 바꾸지 않는다. 다만 enforcement
surface 자체를 바꾼 PR은 review-only trailing commit이 붙어도 과거 green candidate를 재사용하지 않고
CI·CodeQL·Render Diff의 현재 head 전체 검증을 실행하도록 preflight 안전 계약을 보강한다.

## Self-review finding과 보정

### 1. 외부 fork workflow run 누락

Actions API의 server-side `head_sha` filter는 fork run을 누락할 수 있고, fork run의
`pull_requests` 배열도 비어 있을 수 있다. 기존 구현은 이 filter에 의존해 외부 contributor PR의
`CI Impact Policy`가 영구 pending이 될 수 있었다.

source branch로 후보 run을 조회한 뒤 base policy에서 head repository·branch·SHA를 모두 정확히
검증하도록 바꿨다. `pull_requests` 연결 정보가 있으면 현재 PR number와 base ref·SHA도 일치시킨다.
연결 정보가 비어 있는 fork 응답은 exact head identity로만 허용한다.

### 2. 최신 run 선택 순서

`run_attempt`는 한 workflow run의 재시도 횟수이므로 서로 다른 run 사이의 최신성 기준이 아니다.
오래된 attempt 2가 더 최신인 attempt 1을 가릴 수 있던 정렬을 timestamp-first로 바꾸고, 동일 시각의
tie-break에서만 attempt와 id를 사용한다. 회귀 fixture는 잘못 연결된 PR과 오래된 rerun도 함께 거부한다.

### 3. Enforcement 변경과 trailing fast-pass의 교착

policy는 enforcement surface가 바뀐 PR의 fast-pass를 거부하지만, 기존 workflow preflight는 trailing
review-only commit에서 이전 green candidate를 재사용할 수 있었다. 그러면 policy가 실패한 채 세
workflow가 현재 head의 full 증거를 만들지 않는 교착이 생긴다.

CI·CodeQL·Render Diff가 workflow, local action, classifier, policy, merge verifier의 현재 경로와
`previous_filename`을 모두 검사하고, 하나라도 바뀐 PR은 fast-pass 대신 현재 head의 full validation을
실행하도록 세 계약을 일치시켰다.

### 4. CI에서 실행되지 않던 policy 단위 테스트

`ci-impact-policy.test.cjs`를 Lint job에 직접 배선하고 Python 배선 가드가 모든
`scripts/tests/*.test.cjs`도 확인하도록 넓혔다. 영향축을 소비하는 CI job id와 controller allowlist의
완전성도 Node 계약으로 고정했다.

### 5. 안전한 과다 실행 오탐

worker classifier가 일시 실패해 full로 열렸을 때 controller selective 판정보다 많은 검사가 성공해도
필수 검사는 누락되지 않는다. 예상 success는 계속 success만 요구하고, 예상 skip은 `skipped|success`만
허용한다. CodeQL 비선택 언어도 no-op과 실제 full analysis 두 짝만 허용해 모순된 step 조합은 실패한다.

### 6. 취소·stale status 게시

동일 head concurrency로 취소된 controller는 정책 계산·요약·status 게시를 실행하지 않는다. audit에는
workflow trigger SHA를 live PR SHA와 독립 입력으로 전달하고, 게시 직전 PR을 다시 조회해 closed PR이나
새 head에는 이전 상태를 쓰지 않는다.

### 7. 활성화 시 base 이동

main 등록 뒤 기존 열린 PR도 base policy와 workflow evidence base SHA를 표본 감사한다. 서로 다르면
오탐을 허용하지 않고 새 head 실행 또는 base 동기화로 증거를 갱신한다. 이 확인은 required context 채택
전 live audit 게이트다.

### 8. Job 증거 조회 실패의 false failure

[maintainer review](https://github.com/edwardkim/rhwp/pull/4682#issuecomment-5280530677)가 지적한 대로,
`listJobsForWorkflowRun`이 설정된 3회 재시도 뒤에도 실패하면 기존 controller는 예외를 경고한 뒤
`jobs: []`를 기록했다. 완료·성공한 workflow가 이 빈 목록과 함께 policy로 넘어가면
`missing-job:CI preflight` 같은 실제 누락 판정으로 바뀌어 exact head에 false failure가 게시됐다.

보정 commit `e1cb68ff0`은 `jobsCollected`를 API pagination이 온전히 끝난 뒤에만 `true`로 설정한다.
policy는 workflow identity와 run 결론을 먼저 검증한 뒤 `jobsCollected !== true`면
`job-evidence-unavailable:<workflow>` 사유의 pending을 반환한다. API가 정상 응답한 뒤 빈 목록이 온 경우는
기존 `missing-job` failure를 유지해 실제 검사 누락을 숨기지 않는다. 두 경로를 Node 회귀로 각각 고정하고,
workflow 정적 계약은 수집 완료 표지가 직렬화되는지 확인한다.

## 검증

- Node classifier+policy: 53/53 통과
- focused Python CI impact policy workflow 계약: 9/9 통과
- 전체 workflow 계약: 108/108 통과
- `actionlint` 대상 CI·CodeQL·Render Diff·CI Impact Policy: 진단 없음
- `git diff --check`: 통과
- 원격 `f69856f4d`의 CI와 CodeQL: 성공. 로컬 보정 전 head이므로 최종 판정에는 재사용하지 않음
- reviewer finding 재현: 완료·성공 CI와 빈 job 목록은 보정 전
  `failure / CI:missing-job:CI preflight`를 반환함
- 최신 review 기준 head: `d31fa652f`; job 증거 보정 code candidate: `e1cb68ff0`

renderer, layout, paint, sample과 제품 UI를 바꾸지 않으므로 시각·fixture sweep 대상은 아니다.

## 최종 권고

현재는 **pending-ci**다. maintainer finding 보정과 focused 검증은 완료됐다. 다음 순서를
모두 만족한 뒤에만 수용 권고로 전환한다.

1. `e1cb68ff0`과 이 review 기록을 같은 PR branch에 push한다.
2. 보정 내용·로컬 검증·새 head CI 대기 상태를 maintainer review의 후속 코멘트로 게시한다.
3. 같은 최신 head에서 GitHub 전체 CI를 통과시킨다.
4. maintainer의 재확인과 명시적 승인, merge 직전 최신 head·checks·mergeability 재확인 뒤 merge한다.
