# 작업 기록 — task_m100_3790 Stage 2.6 trusted enforcement

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage26-enforcement`
- **worktree**: `tmp/issue-3790-stage26-enforcement`
- **분기 기준**: `upstream/devel` `88012c7e09a6`
- **최신 확인 기준**: `upstream/devel` `55eb2860b7fa`
- **원형**: `tmp/issue-3790-stage26` / `060998dc863a` (읽기 전용 보존)
- **상태**: Draft PR #4682 리뷰 2차 보정·최신 devel 동기화·focused 검증 완료, 새 head CI 대기

## 재개 근거

Stage 3~5가 main에 포함됐고 Stage 5B canary의 same-head selective/full gate도 끝났다. PR #4573의
candidate `7e5216f709bd`에서 전체 job elapsed는 4,820초에서 249초로 94.8%, 실제 병렬 완료시간은
1,068초에서 158초로 85.2%, CodeQL job elapsed는 889초에서 164초로 81.6% 감소했다. 이 수치는
[#3790 canary 완료 코멘트](https://github.com/edwardkim/rhwp/issues/3790#issuecomment-5265357466)에
원격 보존돼 있다.

현재 `main@496333b27d21`은 Stage 5B merge commit `c64b5c70a700`을 포함하지만
`.github/workflows/ci-impact-policy.yml`은 아직 없다. 따라서 지금은 controller를 devel에 통합할 수
있는 시점이지만, 실제 `pull_request_target`·`workflow_run` 등록은 다음 정상 main 릴리즈 뒤다.

## prototype 감사 결과

기존 `060998dc8`은 base classifier만 실행하고 PR head artifact를 읽지 않는 신뢰 경계, exact head
commit status, stale run 무시와 fail-closed 기본값을 재사용할 가치가 있다. 다음 옛 가정은 폐기한다.

- 모든 code PR에서 Rust·Native Skia job이 success여야 한다는 Stage 2.6 이전 full-lane 진리표
- CI workflow 하나가 끝나면 곧바로 policy success를 쓰는 단일 workflow 감사
- CodeQL의 선택 언어와 no-op lane, Render Diff trigger/Canvas 상태를 독립 검증하지 않는 구조
- trailing review-only fast-pass를 전체 PR file classification만으로 판별하는 구조
- reusable workflow job 이름을 한 형태로 가정한 구조. #4573 jobs API 실측에서 skipped call은
  `build-test-archive-a` 같은 caller id, 실행된 call은
  `build-test-archive-a / Build test archive (1)` 형태다. 새 policy는 둘을 한 논리 lane의 alias로 묶되
  둘이 동시에 나타나면 duplicate로 실패시킨다.
- CodeQL job-level matrix 전체가 fast-pass로 skip되면 세 언어 check 대신
  `Analyze (${{ matrix.language }})` literal 한 건이 jobs API에 나타난다. 새 policy는 이 단일 skipped
  template 또는 세 expanded skipped job 중 한 표현만 허용한다.

## 보정된 신뢰 경계

- privileged controller workflow는 default branch 정의로만 실행한다.
- 실행 가능한 저장소 파일은 live PR의 base SHA에서 sparse checkout한 classifier·policy 두 파일뿐이다.
- PR head·merge ref를 checkout하지 않고 artifact도 다운로드하지 않는다.
- API로 읽는 PR files, workflow runs, jobs와 steps는 실행하지 않는 증거 데이터로만 취급한다.
- 각 workflow 완료 이벤트에서 CI·CodeQL·Render Diff의 같은 head repository·branch·SHA 최신 run을
  다시 모은다. `pull_requests` 연결 정보가 있으면 현재 PR과 base ref·SHA도 함께 검증한다. 순서와
  무관하게 미완료는 pending, 실패는 failure다. 필요한 검사를 생략하면 실패시키되, worker 쪽
  fail-open으로 필요보다 많은 검사가 성공한 안전한 상위 집합은 허용한다.
- fast-pass는 workflow·local action·classifier·merge verifier가 base와 동일한 PR에서만 허용한다.
  이 surface가 바뀐 PR은 classifier full 진리표를 실제 job/step으로 증명해야 한다.

## 활성화·종료 게이트

1. Node policy 단위 테스트와 Python workflow 계약, 기존 classifier/CI/CodeQL/Render Diff 계약,
   actionlint와 diff check를 통과한다.
2. devel PR의 full CI를 통과한다. 이때 새 controller는 main에 없어 live status를 발행하지 않는다.
3. 다음 정상 release로 main에 포함된 뒤 실제 PR에서 pending→success와 의도적 불일치 failure 표본을
   확인한다.
4. repository admin이 `Require branches to be up to date before merging`과 GitHub Actions expected source를
   실제 `devel` 보호 규칙에서 검증한 뒤에만 `CI Impact Policy`를 required context로 채택한다. 둘 중
   하나라도 확인되지 않으면 미채택 상태를 유지한다.
5. 위 live audit 또는 미채택 결정 뒤에만 사용자 승인으로 원형 branch/worktree를 정리한다.

## 구현 결과

- `pull_request_target` publish와 세 `workflow_run` audit를 단일 privileged job으로 합쳐 base policy
  checkout·classifier 실행을 한 번만 유지했다.
- exact head SHA별 concurrency를 직렬화하고 새 완료 이벤트가 CI·CodeQL·Render Diff 전체 증거를 다시
  모으게 했다. 일부 미완료는 pending, 필요한 검사의 실패·누락은 failure, 기대 진리표 또는 안전한 full
  상위 집합만 success다.
- CI의 Rust 8개 논리 lane, Native Skia와 frontend unit/package를 classifier 축에 맞춰 감사한다.
  reusable workflow는 skipped caller id와 실행된 `caller / called job` 이름을 alias로 묶고 중복을
  거부한다.
- CodeQL 선택 언어는 실제 analysis step success를 요구한다. 비선택 언어는 no-op 또는 fail-open full
  analysis success 두 형태만 허용하고 모순된 step 조합을 거부한다. 전체 matrix fast-pass의 literal
  job과 expanded job 표현을 각각 허용하되 혼합은 거부한다.
- Render Diff trigger 계약과 Canvas `success|skipped`를 감사한다. 세 workflow의 paths filter 상수는
  실제 YAML 목록과 단위 테스트에서 동일해야 한다.

## maintainer 요청 전 self-review 보정

Draft PR #4682의 첫 전체 CI가 통과한 뒤 최신 devel과 실제 Actions API 동작을 다시 대조해 다음 세
문제를 발견하고 로컬에서 보정했다.

1. 외부 fork run은 Actions API의 server-side `head_sha` filter에서 누락될 수 있다. controller가
   source branch로 run 후보를 조회하고, policy가 head repository·branch·SHA를 로컬에서 모두
   일치시키도록 바꿨다. `pull_requests`가 비어 있는 fork 응답은 이 exact identity로 허용하되,
   연결 정보가 있으면 현재 PR number와 base ref·SHA까지 일치해야 한다.
2. `run_attempt`는 서로 다른 run 사이의 최신성 기준이 아니다. 최신 workflow run은
   `run_started_at|created_at`으로 먼저 고르고 같은 run의 재시도만 attempt·id로 판별한다. 오래된
   attempt 2가 더 최신인 attempt 1을 가리는 회귀 fixture를 추가했다.
3. enforcement surface를 바꾼 PR에 review-only trailing commit이 붙으면 policy는 fast-pass를
   거부하지만 기존 세 workflow는 과거 green candidate를 재사용할 수 있었다. CI·CodeQL·Render Diff가
   workflow·local action·classifier·policy·merge verifier의 현재/이전 경로 변경을 감지하면
   fast-pass 대신 현재 head의 full validation을 실행하도록 계약을 정렬했다.

최신 `upstream/devel@9b9cbf3c80b6`은 PR의 분기 기준보다 11 commits 앞서 있지만
`git merge-tree --write-tree HEAD upstream/devel`은 충돌 없이 tree
`dd3946fac35487b859bbaab81d71f01184eaff2e`를 만들었다. 보정 commit `4ba5e431d` 뒤 최신 devel을
`30bbcf9fe`로 실제 병합했고 같은 로컬 head에서 focused 검증을 다시 통과했다. 이 head를 push한 뒤
전체 CI를 다시 통과시켜야 하며, 첫 CI 성공은 최종 merge 근거로 재사용하지 않는다.

## 게시 self-review 코멘트 보정

[PR 코멘트](https://github.com/edwardkim/rhwp/pull/4682#issuecomment-5267715311)는 이전 head
`f69856f4d`를 기준으로 7개 항목을 제기했다. 현재 head에서 이미 해결된 review 문서 누락을 제외하고 다음을
보정했다.

1. `ci-impact-policy.test.cjs`를 Lint job에 실제 배선하고, `scripts/tests/*.test.cjs`가 빠지면
   `test_workflow_contract_wiring.py`가 실패하게 했다.
2. controller가 요구한 검사를 worker가 생략하면 계속 실패시키되, worker classifier·checkout 오류가
   full로 열려 더 많은 CI·CodeQL·Render Diff가 성공한 경우는 안전한 상위 집합으로 허용했다.
3. 취소된 동일-head controller가 fallback failure를 게시하지 못하도록 정책 계산·요약·status 게시에
   `!cancelled()`를 적용했다.
4. trigger head SHA와 live PR head SHA를 독립 입력으로 비교하고, status 게시 직전 PR을 다시 조회해
   closed/stale head에는 아무 상태도 쓰지 않게 했다.
5. `needs.preflight.outputs`의 영향축을 소비하는 CI job id 전체와 controller 감사 allowlist가 정확히
   일치하는 정적 계약을 추가했다.

활성화 시에는 main 등록 전에 열려 있던 PR도 base policy와 workflow evidence의 base SHA가 일치하는지
표본으로 확인한다. devel 이동 때문에 둘이 다르면 임의 허용하지 않고 새 head 실행 또는 base 동기화로
증거를 갱신한 뒤 required context 채택 여부를 판단한다.

## 후속 review — file-list 완전성과 base 이동

head `3fca10931` 기준 후속 review는 두 P1 경계를 확인했다.

1. 외부 fork에서 PR file 목록이 3,000개 경계로 잘리거나 조회에 실패하면 기존 policy는 관측된 부분만으로
   enforcement 변경을 판단해 `full`로 진행했다. `controller-mediated-fork`과 `forceFullReason`의 조합을
   `blocked`로 바꾸고 `collection-error`, `pull_request-file-list-boundary`, `file-list-empty`를 회귀로
   고정했다. same-repository/collaborator는 보수적 full 실행을 유지한다.
2. head가 고정된 채 `devel`만 이동하면 기존 성공 status가 같은 head에 남는다. 작성 시점 PR은 base
   snapshot `d871bb8ce`에서 현재 `devel@a5a92ca3`보다 34 commits 뒤였지만 `CLEAN`이었고, required context는
   `Build & Test` 하나였다. 따라서 strict up-to-date가 현재 실제 설정에 적용되지 않았으며,
   `CI Impact Policy` required 활성화는 금지 상태다.

base-push마다 모든 열린 PR에 status를 게시하는 fan-out은 별도 운영 승인 없이 도입하지 않는다.
`CI Impact Policy`만 test-merge SHA에 게시하는 안도 기존 head SHA의 `Build & Test`와 required-check 기준
commit을 갈라놓을 수 있어 채택하지 않는다. 대신 required 활성화의 선행조건을 다음처럼 고정한다.

- repository admin이 `required_status_checks.strict=true`와 GitHub Actions app expected source를 실제
  protection API 또는 Settings에서 확인한다.
- live 표본에서 `base B1 / head H success → 동일 H / base B2`가 `BEHIND` 또는 merge 차단이 되고,
  base 반영으로 새 head가 생성된 뒤에만 새 policy success가 생기는지 확인한다.
- policy status contract v3는 평가한 40자리 base SHA를 `b=<sha>`로 포함한다. 같은 head의 B1/B2 설명이
  달라지는 단위 회귀로 감사 generation을 식별한다.

현재 설정이 선행조건을 충족하지 않으므로 main 등록 뒤에도 live audit과 admin 설정 검증 전에는
non-required 관측 상태로만 운영한다.

최신 `upstream/devel@55eb2860b7fa`는 merge simulation tree
`f70884072c51f4248b4361f8aa5b3c56ee0f57dc`로 충돌이 없었고, 실제 merge head `d5b7ef831`에 반영했다.

## 실제 API 대조

PR #4573의 같은 head 표본을 GitHub jobs API로 다시 읽어 fixture와 대조했다.

- selective: CI `31582691020`, CodeQL `31582690810`, Render Diff `31582690807`
- manual full: CI `31583545091`, CodeQL `31583557284`, Render Diff `31583566849`
- trailing review-only fast-pass: CI `31586324768`, CodeQL `31586324514`, Render Diff `31586324623`

selective에서는 skipped reusable call이 caller id, full에서는 `caller / called job` 이름으로 나타났다.
CodeQL fast-pass는 `Analyze (${{ matrix.language }})` literal 한 건으로 나타났고, Render Diff는 전체 PR
diff가 Studio 경로를 포함하므로 classifier를 다시 실행해 Canvas만 skipped했다. 새 policy fixture가 세
형태를 모두 반영한다.

## focused 검증

```bash
node --test \
  scripts/tests/ci-impact-classifier.test.cjs \
  scripts/tests/ci-impact-policy.test.cjs
python3 -m unittest discover -s scripts/tests -p 'test_*workflow.py'
actionlint \
  .github/workflows/ci.yml \
  .github/workflows/codeql.yml \
  .github/workflows/render-diff.yml \
  .github/workflows/ci-impact-policy.yml
git diff --check
```

- Node classifier+policy: 51/51 통과
- Python workflow 계약: 107/107 통과
- actionlint: 진단 없음
- whitespace: 통과

제품 Rust·TypeScript·renderer 코드는 바꾸지 않는다. 이번 gate는 controller의 순수 Node 정책,
workflow 정적 계약과 실제 Actions metadata 대조이므로 Cargo·Studio·WASM·시각 검증은 범위에서 제외한다.

## Maintainer review job 증거 보정 — 2026-08-14

[review comment](https://github.com/edwardkim/rhwp/pull/4682#issuecomment-5280530677)의 경로를
원격 head `d31fa652f`에서 재현했다. `listJobsForWorkflowRun`이 설정된 3회 재시도 뒤에도 실패하면
controller가 `jobs: []`를 기록했고, 완료·성공한 CI run과 이 빈 목록을 policy에 넣은 결과는 다음과 같았다.

```json
{
  "publish": "true",
  "conclusion": "failure",
  "reason": "CI:missing-job:CI preflight"
}
```

code candidate `e1cb68ff0`은 API pagination이 끝난 경우에만 `jobsCollected: true`를 기록한다. workflow
identity와 run 결론을 검증한 뒤 이 표지가 없으면 `job-evidence-unavailable:<workflow>` pending으로
남기며, 정상 조회 뒤 실제 job이 없으면 기존 `missing-job` failure를 유지한다. 따라서 transient API
증거 부족은 false red로 바뀌지 않고, 실제 필수 검사 누락도 숨기지 않는다.

이 후속 보정은 Node classifier+policy 55/55, focused Python workflow 9/9, 전체 workflow 108/108을
통과했다. CI·CodeQL·Render Diff·CI Impact Policy 네 workflow의 actionlint와 `git diff --check`도
통과했다. code/test 보정은 `8ff214740`으로 분리 커밋했다.
renderer·제품 코드·fixture 변경은 없어 Cargo·Studio·WASM·시각 검증은 추가하지 않았다. 이 code와 review
기록을 PR branch에 push한 뒤 최신 head GitHub Actions와 maintainer 재확인을 새 merge 조건으로 둔다.
