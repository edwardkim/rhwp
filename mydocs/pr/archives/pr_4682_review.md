---
kind: pr-review
status: needs-rework
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4682 self-review — 선택 실행 독립 정책 감사

## 결론

**보정 뒤 maintainer review 필요.** [PR #4682](https://github.com/edwardkim/rhwp/pull/4682)는
Stage 3~5에서 빨라진 CI·CodeQL·Render Diff가 필요한 job과 step을 실제로 실행했는지 default-branch
controller가 독립 검증하는 변경이다. 제품 기능이나 선택 실행 규칙을 새로 줄이는 PR이 아니라,
PR-controlled workflow가 필수 검사를 잘못 건너뛰면 `CI Impact Policy`를 실패시키는 안전망을 추가한다.

원격 head `f69856f4d`의 첫 전체 CI는 통과했다. 다만 maintainer 요청 전 self-review에서 외부 fork run
조회, 최신 run 선택, enforcement 변경 PR의 trailing fast-pass 정합 문제를 발견해 로컬에서 보정했다.
따라서 첫 CI는 최종 merge 근거로 재사용하지 않는다. 보정 commit, 최신 devel 동기화, 새 head의 전체
CI와 `edwardkim`의 명시적 승인 뒤에만 merge를 권고한다.

## 검토 경로

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
           rework_and_exceptions.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
                  collaborator_self_merge.md, intake_and_review.md,
                  local_validation.md, rework_and_exceptions.md
remote head at intake: f69856f4d5101eec4d9a454f7db91e3bb8a18a22
local correction head: 아직 commit하지 않음
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
pending, workflow 실패·증거 누락·중복·예상 밖 실행은 failure로 닫는다.

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

## 검증

- Node classifier+policy: 47/47 통과
- focused Python 계약: 20/20 통과
- 전체 workflow 계약: 106/106 통과
- `actionlint` 대상 CI·CodeQL·Render Diff·CI Impact Policy: 진단 없음
- `git diff --check`: 통과
- 원격 `f69856f4d`의 CI와 CodeQL: 성공. 로컬 보정 전 head이므로 최종 판정에는 재사용하지 않음
- 최신 `upstream/devel@9b9cbf3c80b6` merge simulation: 충돌 없음,
  tree `dd3946fac35487b859bbaab81d71f01184eaff2e`

renderer, layout, paint, sample과 제품 UI를 바꾸지 않으므로 시각·fixture sweep 대상은 아니다.

## 최종 권고

현재는 **needs-rework**다. 로컬 finding 보정 자체는 검증됐지만 아직 commit·push되지 않았고 branch가
최신 devel보다 11 commits 뒤에 있다. 다음 순서를 모두 만족한 뒤에만 수용 권고로 전환한다.

1. self-review 보정과 기록을 commit한다.
2. 최신 `upstream/devel`을 merge하고 충돌·최종 diff를 재확인한다.
3. 같은 head에서 focused 검증과 GitHub 전체 CI를 다시 통과시킨다.
4. Draft를 해제하고 `edwardkim`에게 controller의 privileged 경계와 main 등록 뒤 required status 채택
   판단을 포함한 review를 요청한다.
5. maintainer의 명시적 승인과 merge 직전 최신 head·checks·mergeability 재확인 뒤 merge한다.
