# 작업 기록 — task_m100_3790 Stage 2.6 trusted enforcement

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage26-enforcement`
- **worktree**: `tmp/issue-3790-stage26-enforcement`
- **분기 기준**: `upstream/devel` `88012c7e09a6`
- **최신 확인 기준**: `upstream/devel` `9b9cbf3c80b6`
- **원형**: `tmp/issue-3790-stage26` / `060998dc863a` (읽기 전용 보존)
- **상태**: Draft PR #4682 self-review 보정·최신 devel 동기화·focused 검증 완료, push·새 CI 대기

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
  무관하게 미완료는 pending, 실패는 failure, 전체 진리표 일치만 success다.
- fast-pass는 workflow·local action·classifier·merge verifier가 base와 동일한 PR에서만 허용한다.
  이 surface가 바뀐 PR은 classifier full 진리표를 실제 job/step으로 증명해야 한다.

## 활성화·종료 게이트

1. Node policy 단위 테스트와 Python workflow 계약, 기존 classifier/CI/CodeQL/Render Diff 계약,
   actionlint와 diff check를 통과한다.
2. devel PR의 full CI를 통과한다. 이때 새 controller는 main에 없어 live status를 발행하지 않는다.
3. 다음 정상 release로 main에 포함된 뒤 실제 PR에서 pending→success와 의도적 불일치 failure 표본을
   확인한다.
4. repository admin이 `CI Impact Policy`를 devel required context로 채택하거나 미채택 결정을 남긴다.
5. 위 live audit 또는 미채택 결정 뒤에만 사용자 승인으로 원형 branch/worktree를 정리한다.

## 구현 결과

- `pull_request_target` publish와 세 `workflow_run` audit를 단일 privileged job으로 합쳐 base policy
  checkout·classifier 실행을 한 번만 유지했다.
- exact head SHA별 concurrency를 직렬화하고 새 완료 이벤트가 CI·CodeQL·Render Diff 전체 증거를 다시
  모으게 했다. 일부 미완료는 pending, 실패·진리표 불일치는 failure, 전체 일치만 success다.
- CI의 Rust 8개 논리 lane, Native Skia와 frontend unit/package를 classifier 축에 맞춰 감사한다.
  reusable workflow는 skipped caller id와 실행된 `caller / called job` 이름을 alias로 묶고 중복을
  거부한다.
- CodeQL 선택 언어는 실제 analysis step success, 비선택 언어는 skip step success와 analysis step
  skipped를 요구한다. 전체 matrix fast-pass의 literal job과 expanded job 표현을 각각 허용하되 혼합은
  거부한다.
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

- Node classifier+policy: 47/47 통과
- Python workflow 계약: 106/106 통과
- actionlint: 진단 없음
- whitespace: 통과

제품 Rust·TypeScript·renderer 코드는 바꾸지 않는다. 이번 gate는 controller의 순수 Node 정책,
workflow 정적 계약과 실제 Actions metadata 대조이므로 Cargo·Studio·WASM·시각 검증은 범위에서 제외한다.
