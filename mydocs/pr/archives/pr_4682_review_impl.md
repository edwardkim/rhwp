---
kind: pr-review-implementation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-14
---

# PR #4682 self-review 보정 및 수용 이행 기록

## 목적

[PR #4682](https://github.com/edwardkim/rhwp/pull/4682)의 대형 CI 제어면 변경을 로컬 보정, 최신 devel
동기화, 새 head CI, maintainer 승인, main 등록 뒤 live audit의 분리된 단계로 이행한다. 각 단계가
완료되기 전 다음 단계의 성공을 선판정하지 않는다.

## Stage 1 — maintainer 요청 전 self-review 보정

- [x] 외부 fork를 누락할 수 있는 server-side `head_sha` filter를 source branch 조회와 base policy의
  repository·branch·SHA exact match로 대체한다.
- [x] 가능한 경우 현재 PR number와 base ref·SHA 연결을 추가 검증하고, fork의 빈 연결 배열은 exact
  head identity에서만 허용한다.
- [x] 최신 run을 timestamp-first로 고르고 run attempt는 tie-break로만 사용한다.
- [x] CI·CodeQL·Render Diff가 enforcement surface의 현재/이전 경로 변경을 감지하면 trailing
  fast-pass를 금지하고 현재 head full validation을 실행한다.
- [x] 단위·workflow 계약, actionlint, diff check를 통과한다.
- [x] 계획서·작업 기록·오늘할일·PR review에 finding과 남은 gate를 반영한다.

## Stage 2 — 최신 devel 통합

- [x] `upstream/devel@9b9cbf3c80b6`을 fetch하고 merge simulation에서 충돌이 없음을 확인한다.
- [x] self-review 보정 commit `4ba5e431d` 뒤 최신 devel을 `30bbcf9fe`로 실제 merge한다.
- [x] 최종 diff가 PR 목적 밖 제품·배포 변경을 포함하지 않는지 다시 확인한다.
- [x] 동기화 head에서 focused 검증과 `git diff --check`를 다시 실행한다.
- [x] 게시 self-review 뒤 전진한 `upstream/devel@55eb2860b7fa`도 merge head `d5b7ef831`에 반영한다.
- [x] maintainer review 시점의 최신 devel을 추가 반영한 원격 head `d31fa652f`를 보정 기준으로 고정한다.

## Stage 2.5 — 게시 self-review 후속 보정

- [x] `ci-impact-policy.test.cjs`를 Lint job에 배선하고 CJS 테스트 누락 가드를 추가한다.
- [x] 필요한 검사 누락은 실패시키면서 안전한 full 상위 집합은 허용한다.
- [x] 취소된 controller의 정책 계산·요약·status 게시를 차단한다.
- [x] trigger/live head 독립 비교와 게시 직전 live PR 재조회로 stale status를 차단한다.
- [x] 영향축을 소비하는 CI job과 감사 allowlist의 완전성 계약을 추가한다.
- [x] 활성화 시 기존 PR과 base 이동은 required 채택 전 live audit 항목으로 기록한다.
- [x] Node 51/51, focused Python 21/21, 전체 workflow 107/107, actionlint와 diff check를 통과한다.

## Stage 2.75 — maintainer review job 증거 보정

- [x] job API 조회 실패가 빈 job 목록으로 바뀌어 false failure를 게시하는 finding을 재현한다.
- [x] 기존 `actions/github-script`의 `retries: 3`을 유지하고, 재시도 소진 뒤 수집 완료 여부를
  `jobsCollected`로 명시한다.
- [x] 불완전한 job 증거는 `job-evidence-unavailable:<workflow>` pending, 정상 조회 뒤 실제 누락은
  `missing-job` failure로 분리한다.
- [x] 두 판정과 workflow 직렬화 경로를 Node·Python 회귀 테스트로 고정한다.
- [x] code candidate `e1cb68ff0`에서 Node 53/53, focused Python 9/9, 전체 workflow 108/108,
  actionlint와 diff check를 통과한다.

## Stage 3 — 원격 보정과 새 head CI

- [x] 작업지시자 승인 뒤 보정·동기화 commit을 같은 PR branch에 push한다.
- [x] PR 본문의 검증 수치와 동작 설명을 최종 구현에 맞춰 갱신한다.
- [ ] `e1cb68ff0`과 review 기록을 같은 PR branch에 push하고 최신 head를 재확인한다.
- [ ] maintainer finding 보정, 로컬 검증과 새 CI 대기 상태를 후속 코멘트로 게시한다.
- [ ] CI·CodeQL·Render Diff와 `Build & Test`의 같은 최신 head 결과를 분석한다.
- [ ] 실패가 있으면 Draft를 유지하고 원인을 보정한 뒤 새 head에서 다시 검증한다.

## Stage 4 — maintainer review

- [ ] 작업지시자에게 review request 문안을 먼저 제시한다.
- [ ] 승인 뒤 Draft를 해제하고 reviewer `edwardkim`을 지정한다.
- [ ] 다음 항목의 명시적 판단을 요청한다.
  - privileged controller가 PR head code·artifact를 실행하지 않는 신뢰 경계
  - 외부 fork와 stale/retry run identity 검증
  - CI·CodeQL·Render Diff job/step 진리표와 fail-closed 동작
  - 정상 `devel → main` 반영 뒤 `CI Impact Policy` required context 채택 여부
- [ ] maintainer 승인 전에는 collaborator 권한이 있어도 merge하지 않는다.

## Stage 5 — merge와 운영 후속

- [ ] 승인 뒤 최신 head·required checks·MERGEABLE/CLEAN을 재확인하고 `devel`에 merge한다.
- [ ] merge comment와 #3790 업데이트 comment에 merge commit, 검증, 아직 활성화되지 않은 이유를 남긴다.
- [ ] 정상 release로 workflow가 main에 포함될 때까지 controller 비활성 상태를 유지한다.
- [ ] main 등록 뒤 live PR에서 pending→success와 의도적 불일치 failure를 감사한다.
- [ ] repository admin의 required context 채택 또는 미채택 결정을 기록한다.
- [ ] 위 live audit 또는 미채택 결정 뒤에만 작업지시자 승인으로
  `tmp/issue-3790-stage26` prototype과 새 worktree 정리를 검토한다.
