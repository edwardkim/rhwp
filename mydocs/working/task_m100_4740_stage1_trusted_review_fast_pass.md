---
kind: working
status: completed
canonical: mydocs/plans/task_m100_4740.md
last_verified: 2026-08-15
---

# Task M100 #4740 Stage 1 — trusted review-only fast-pass

## 기준선과 재현

- 기준선: `upstream/devel@62a3ba677`
- 계획 커밋: `43554af03`
- 관찰 PR: #4819
- Full code candidate: `bd01f583424054f6550afa0460a114c531ff662c`
- review-only current head: `b8b8a37b615060485f552401e1eb23e2a136f714`

candidate의 CI run `31880305532`와 CodeQL run `31880305422`는 성공했지만, current head에서는 PR 전체
변경 목록의 workflow 파일 때문에 preflight가 `ci-execution-surface-change`로 종료하고 CI
`31881202046`과 CodeQL `31881201970`을 Full로 다시 실행했다. PR head가 자신이 수정한 workflow로 검증을
생략하지 못하게 만든 기존 fail-closed guard가 원인이며, 단순 path 예외로 완화할 수 없는 보안 경계다.

## 구현

기본 브랜치의 `CI Impact Policy Controller`가 다음 증빙을 live base의 코드로만 판정하도록 확장했다.

- same-repository PR의 commit 수와 전 페이지 file 목록을 수집하고, newest-to-oldest로 single-parent
  review-only tail과 직전 code candidate를 고른다.
- current-base merge bridge는 commit object만 fetch하며 PR head를 checkout하거나 실행하지 않는다.
  자동 merge tree 일치 또는 trusted base 검사기의 `mydocs/` 한정 remerge diff만 허용한다.
- candidate의 exact repository·branch·PR·SHA에 결합된 CI·CodeQL·필요한 Render Diff run과 job/step을
  감사한다. 세 workflow의 preflight가 fast-pass였던 candidate는 Full 증빙으로 인정하지 않는다.
- CodeQL은 세 Analyze job 외에 같은 candidate와 run 시작 이후의 GHAS `CodeQL` check까지 확인한다.
- 증빙이 완결되면 policy status v5에 `rfp=1`을 넣고 exact current head에 success로 발행한다. current-head
  no-op aggregate가 끝나는 동안 audit event가 pending이어도 이 authorization은 유지하며, 실제 audit
  failure는 failure status로 덮어쓴다.

CI·CodeQL·Render Diff consumer는 PR 전체에 execution-surface 변경이 있을 때만 최대 30초 동안 status를
기다린다. context·creator뿐 아니라 v5, `rfp=1`, current base SHA, target run의 controller name·path와
`pull_request_target` event를 함께 확인한 뒤 기존 candidate 탐색을 계속한다. status가 이미 도착했으나
재사용 불가이면 즉시 Full 경로로 전환한다.

## Fail-closed 경계

다음은 모두 `trusted_review_fast_pass=false` 또는 기존 Full 실행이다.

- 외부 fork의 execution-surface 변경
- PR commit 수 불일치, 빈/300-file 경계 commit, file/API/classifier 수집 오류
- trailing source·test·workflow 변경, 일반 merge, 복수 base merge
- candidate run 누락·pending·failure·identity 불일치 또는 candidate 자체 fast-pass
- GHAS CodeQL check 누락·실패·candidate/run 시간 불일치
- 검증되지 않은 current-base merge tree, stale base/status/controller run

일반 code PR과 PR 전체가 review-only인 기존 경로는 policy status를 소비하지 않고 이전 계약을 유지한다.

## 검증 결과

- `node scripts/tests/ci-impact-classifier.test.cjs`: 31/31 PASS
- `node scripts/tests/ci-impact-policy.test.cjs`: 31/31 PASS
- CI·CodeQL·Render Diff·review-only·policy controller·workflow wiring Python 계약: 70/70 PASS
- PyYAML 변경 workflow 4개 parse: PASS
- 변경 manual 3개 Markdown 상대 링크: PASS
- `git diff --check`: PASS
- `actionlint`: 로컬 실행 파일이 없어 미실행. YAML 구조와 repository workflow mirror test로 보완했으며,
  실제 Actions 결과는 PR에서 확인해야 한다.

## 배포 경계

`pull_request_target`과 `workflow_run`은 default branch에 있는 workflow만 등록한다. 이 변경을 `devel`에
병합해도 controller는 아직 live가 아니며, 정상 release로 `main`에 반영된 뒤부터 후속 workflow PR이
`rfp=1`을 사용할 수 있다. #4819의 이미 실행 중인 current head에는 소급 적용되지 않는다.
