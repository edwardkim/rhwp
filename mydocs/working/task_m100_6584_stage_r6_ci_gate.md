# Task M100 #6584 Stage R6 결과 — canonical promotion Rust test-policy gate 정정

- Issue: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- release PR: [#6592](https://github.com/edwardkim/rhwp/pull/6592)
- 최초 실패 head: `bd8cdd0a1ab476ef401e51e70b2436ff012c33ae`
- correction code head: `fa194b820eb6bfca91d819399ec6d3de5a524788`
- 최초 CI: [run #33575299570](https://github.com/edwardkim/rhwp/actions/runs/33575299570)
- 작성일: 2026-09-02 KST

## 1. 실패 원인

PR #6592의 최초 Lint는 `Validate Rust test suite manifest`에서 종료됐다. Clippy·WASM compiler나 제품
source 오류가 아니라, 현행 Rust test-policy 도입 전 `main@496333b27`을 증분 비교 기준으로 사용한 것이
원인이다.

- `main`에는 `tests/suites/suite-policy.json`과 `tests/suites/unit-test-tier-policy.json`이 없다.
- `main..devel`에는 최상위 integration source 33개, `tests/cases/` source 545개와 그 밖의 source 2개가
  누적됐다.
- 이 source들은 `devel`에서 개별 PR마다 검증·병합됐지만 release promotion의 단일 diff에서는 모두
  “PR base에 없는 신규 source”로 보인다.
- manifest 단계의 첫 실패가 없더라도 unit-tier 단계는 base 정책 부재로 이어서 실패한다.

따라서 source를 이동하거나 예외 목록을 늘리는 것은 이미 검증된 이력을 훼손하는 우회이며 정답이 아니다.

## 2. 정정 정책과 보호 불변식

일반 PR의 실제 base 비교를 기본값으로 유지한다. 다음 조건을 모두 만족하는 canonical promotion에서만
manifest와 unit-tier의 **증분 비교 기준**을 exact head SHA로 전환한다.

1. base branch `main`, head branch `devel`이다.
2. head repository가 현재 repository와 같다. fork의 동명 branch는 해당하지 않는다.
3. exact base SHA가 exact head SHA의 조상이다.
4. checkout한 synthetic merge tree가 exact head tree와 같다.

SHA 공백, ancestry 위반과 tree 불일치는 fail-closed 한다. 위 조건을 통과해도
`rust-test-suite-manifest.mjs --check`와 `rust-unit-test-tiers.mjs --check`의 현재 전체 정합 검사는 그대로
실행한다. 바뀌는 것은 이미 `devel`에서 통과한 누적 변경의 증분 기준뿐이다.

## 3. 구현

- `.github/workflows/ci.yml`이 PR base/head branch·SHA·repository를 명시적으로 받는다.
- canonical promotion이면 `git merge-base --is-ancestor`와 두 tree SHA를 검사한 뒤 head SHA를
  `--base-ref`로 전달한다.
- 일반 PR은 기존과 같이 PR base SHA를 manifest와 unit-tier 검사에 함께 전달한다.
- workflow 계약 테스트는 기본 base 보존, same-repository branch 조건, ancestry와 tree guard,
  head 기준 전환을 고정한다.

## 4. 검증

### 4.1 focused 계약

- workflow YAML parse와 `Validate Rust test suite manifest` Bash syntax: 통과
- `rust-test-suite-manifest.test.mjs`: 21/21
- `rust-unit-test-tiers.test.mjs`: 12/12
- `test_ci_impact_workflow`: 33/33
- 변경 문서 2개 내부 링크와 `git diff --check`: 통과

### 4.2 exact promotion 재현

`upstream/main@496333b27`과 correction head의 merge-tree가 correction head tree
`aaf3627796c7a07c479f00a606aa5def2f1312e4`와 같음을 먼저 확인했다. 격리 worktree에서 CI의 manifest
step을 동일 환경값으로 실행한 결과는 다음과 같다.

- integration: 1,110 sources, 4,777 static test attributes
- generated/exception targets: 28 suites + 20 exceptions = 48/48
- nextest minimum cases: 6,559
- unit-tier: 4,221 tests / 299 modules / cfg support items 28
- `issue_1035_alignment` target resolution: 성공

### 4.3 fail-closed 반례

- fork repository의 `devel`: promotion 예외를 타지 않고 실제 `main` base 유지
- 같은 repository의 일반 feature branch: 실제 `main` base 유지
- same-repository `devel`이라도 checkout tree와 지정 head tree 불일치: 즉시 실패

## 5. 영향과 다음 게이트

- 제품 Rust source, renderer, WASM API, fixture·baseline과 배포 manifest는 바꾸지 않는다.
- 시각 결과에 영향이 없어 별도 visual sweep은 필요하지 않다.
- correction PR을 `devel`에 정상 병합한 뒤 PR #6592가 새 exact head로 갱신돼야 한다.
- 갱신된 release PR의 Full CI·CodeQL·Pages와 self-review가 성공하기 전에는 `main` 병합, tag와 Release를
  진행하지 않는다.
