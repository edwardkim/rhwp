# PR #6594 검토 기록 — canonical devel promotion test-policy 기준 정정

- PR: [#6594](https://github.com/edwardkim/rhwp/pull/6594)
- 이슈: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- 선행 release PR: [#6592](https://github.com/edwardkim/rhwp/pull/6592)
- 작성자·검토자: `edwardkim` collaborator self-review
- base: `devel@bd8cdd0a1ab476ef401e51e70b2436ff012c33ae`
- 검토 대상 head: `65cb562c11804ca6f0b054179c3abb86720bca64`
- 핵심 workflow commit: `fa194b820eb6bfca91d819399ec6d3de5a524788`
- 규모: 5 files, +186 / -5, 2 commits
- 검토일: 2026-09-02 KST

## 1. 리뷰 라우팅

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
  `pr_review/collaborator_self_merge.md`, `pr_review/intake_and_review.md`,
  `pr_review/local_validation.md`, `pr_review/review_only_fast_pass.md`,
  `github_operations.md`
- 작성자 본인의 self-review이므로 reviewer를 지정하지 않았다.

## 2. 문제와 변경 범위

release PR #6592의 최초 Lint는 제품 source나 Clippy 실패가 아니라, test-policy가 도입되기 전
`main@496333b27`을 증분 기준으로 사용해 `devel`에서 이미 개별 검증·병합된 integration source를 모두
신규 위반으로 오판하면서 실패했다. `main`에는 unit-tier 정책도 없으므로 manifest 첫 오류만 우회해도
후속 unit-tier 검사가 같은 구조로 실패한다.

이 PR은 일반 PR의 실제 base 비교를 유지하면서, 아래 조건의 교집합을 만족하는 canonical promotion에서만
manifest와 unit-tier의 증분 기준을 exact head로 바꾼다.

1. base branch는 `main`, head branch는 `devel`이다.
2. head repository와 현재 repository가 같다.
3. exact base SHA가 exact head SHA의 조상이다.
4. checkout한 synthetic merge tree가 exact head tree와 같다.

SHA 공백, ancestry 위반과 tree 불일치는 fail-closed 한다. 예외를 통과해도 현재 manifest 전체 정합과
unit-tier 전체 inventory 검사는 그대로 실행한다. 제품 Rust source, renderer, WASM API, fixture·baseline,
sample, 배포 manifest와 required-check 이름·권한은 바꾸지 않는다.

PR 본문은 #6584와 #6592를 참조한다. 실제 release promotion, tag·GitHub Release와 공식 채널 정산이
남았으므로 #6594 병합만으로 #6584를 닫지 않는다.

## 3. 코드·운영 계약 self-review

### 3.1 예외 경계와 fail-closed 동작

- 일반 PR은 `github.event.pull_request.base.sha`를 manifest와 unit-tier 양쪽의 공통 증분 기준으로 유지한다.
- fork의 동명 `devel`은 repository 일치 조건에서 제외되고, same-repository 일반 feature branch도 branch
  조건에서 제외된다.
- canonical branch 조합이라도 base/head SHA가 비거나 ancestry가 끊기면 즉시 실패한다.
- GitHub가 checkout한 PR synthetic merge tree와 event의 exact head tree가 다르면 즉시 실패한다.
- 예외가 선택돼도 `--prepare` 뒤 manifest 전체 검사, unit-tier 전체 검사와 target resolution은 생략되지
  않는다.

### 3.2 GitHub Actions 영향

GitHub 운영 등급은 job command와 release PR 라우팅을 바꾸는 O3 실행·보안 계약으로 판정했다. workflow의
trigger, permissions, action pin, job·required context 이름은 바뀌지 않았다. 2026-09-02 live branch
protection 조회에서 `devel`과 `main`의 required context는 모두 기존 `Build & Test`였다.

PR head가 바꾸는 workflow를 스스로 신뢰해 review-only 후행 실행을 생략해서는 안 된다. 이 review 기록을
push한 뒤에는 기본 branch의 trusted controller가 exact Full candidate와 single-parent `mydocs/` tail을
증명한 경우에만 재사용하며, status가 없거나 불완전하면 Full CI fallback 결과를 기다린다.

### 3.3 렌더 영향과 시각 검증

renderer/layout/typeset/paint, WASM API, Studio 화면, HWP/HWPX/PDF fixture와 시각 기준 자료를 변경하지
않는다. 따라서 이 PR에는 visual sweep이 필요하지 않다.

## 4. 검증

### 4.1 로컬·격리 검증

- workflow YAML parse와 manifest step Bash syntax가 통과했다.
- `rust-test-suite-manifest.test.mjs`: 21/21 통과.
- `rust-unit-test-tiers.test.mjs`: 12/12 통과.
- `test_ci_impact_workflow.py`: 33/33 통과.
- exact promotion 격리 재현에서 synthetic merge tree와 head tree가 일치했고, integration 1,110 sources,
  48/48 targets, nextest 최소 6,559 cases와 unit-tier 4,221 tests / 299 modules가 통과했다.
- fork의 동명 `devel`과 일반 feature branch는 실제 base를 유지했고, tree mismatch 반례는 실패했다.
- `git diff --check upstream/devel...65cb562c11804ca6f0b054179c3abb86720bca64`가 통과했다.

상세 원인·명령·계측값은 [Stage R6 CI gate 결과](../../working/task_m100_6584_stage_r6_ci_gate.md)에
고정했다.

### 4.2 exact PR head GitHub 검증

검토 대상 head `65cb562c11804ca6f0b054179c3abb86720bca64`의 PR-triggered check는 성공 26,
정책상 생략 3, 실패·취소·대기 0으로 종료됐다.

- CI [run #33576546357](https://github.com/edwardkim/rhwp/actions/runs/33576546357): Lint, Native Skia,
  frontend package, archive A/B/C/D build·shard와 `Build & Test` 성공.
- CodeQL [run #33576546394](https://github.com/edwardkim/rhwp/actions/runs/33576546394): JavaScript/TypeScript,
  Python, Rust 분석과 GHAS `CodeQL` check 성공.
- Proptest [run #33576546464](https://github.com/edwardkim/rhwp/actions/runs/33576546464): 성공.
- Adapter inter-diff [run #33576546484](https://github.com/edwardkim/rhwp/actions/runs/33576546484): 성공.

이 exact head에서 GitHub Full CI가 완료됐고 self-review 중 code·test·fixture·workflow 보정을 추가하지
않았으므로 동일한 광범위 Rust 회귀를 로컬에서 다시 실행하지 않았다. 대신 변경 범위의 focused 계약과
live required context를 독립 재확인했다.

## 5. 발견 사항과 잔여 위험

- 차단 발견 사항은 없다.
- #6594 자체는 `devel` 대상이므로 canonical `devel -> main` 조건의 실제 GitHub event를 직접 실행하지
  않는다. 이 경로의 최종 live E2E는 #6594 병합으로 head가 갱신된 #6592의 Full Lint·CodeQL·Pages에서
  확인해야 한다.
- #6592의 새 exact head가 실패하면 main 병합, tag와 Release를 진행하지 않고 #6594의 가정과 로그를 다시
  조사한다.
- #6584는 필수 배포 채널과 기록 정산 뒤, #5949는 실제 Linux AArch64 release asset 확인 뒤, #6243은
  post-release Render Diff canary 성공 뒤에만 닫는다.
- 이 review·오늘할일 후행 변경은 `mydocs/`만 포함한다. push 뒤 최신 head의 trusted reuse 또는 Full CI
  결과와 mergeability를 다시 확인해야 한다.

## 6. 최종 판정

- 판정: 승인
- 검증 대상: head `65cb562c11804ca6f0b054179c3abb86720bca64`.
- merge 전 조건: 이 review 기록을 포함한 최신 trailing head의 GitHub Actions 성공, 최신 `devel` 대사,
  `MERGEABLE / CLEAN` 재확인과 메인테이너의 별도 merge 승인.
- merge 후 조건: 갱신된 release PR #6592의 exact head에서 Full CI·CodeQL·Pages와 별도 self-review를
  처음부터 다시 통과한다.
- 이 기록 자체는 GitHub review comment, 원격 push 또는 merge를 수행하지 않는다.
