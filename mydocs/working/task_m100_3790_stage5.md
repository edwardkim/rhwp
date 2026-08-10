# 작업 기록 — task_m100_3790 Stage 5A

- **이슈**: [#3790](https://github.com/edwardkim/rhwp/issues/3790)
- **브랜치**: `issue-3790-stage5a-codeql-safety`
- **worktree**: `tmp/issue-3790-stage5a-codeql`
- **최초 기준**: `upstream/devel` `e48fe86947fb` (#4310·#4317 merge 포함)
- **최신 동기화 기준**: `upstream/devel` `0664e6568e9b` (#4513 merge 포함)
- **상태**: Ready PR #4341 self-review F1–F6 보정·최신 devel 병합 완료, 새 full CI·CodeQL 대기

## 재개와 보존 경계

- 기존 `tmp/issue-3790-stage5-codeql`은 최신 devel 이전의 미완성 prototype과 보정 설계가 함께 있던
  rejected WIP다. 필요한 근거를 이 기록으로 옮기고 Stage 5A focused 검증을 통과한 뒤 작업지시자의
  명시적 승인을 받아 worktree와 로컬 브랜치 `codex/issue-3790-stage5-codeql`을 정리했다.
- `tmp/issue-3790-stage26`은 어느 원격에도 없는 Stage 2.6 controller prototype의 유일본이므로 이
  단계의 정리 대상이 아니다.
- #4310 merge 뒤 classifier의 `codeql_languages` 계약은 유지됐고 Native Skia 대상만 보강됐다.

## 설계 보정 근거

- Stage 4 canary PR #4078은 wall clock 575초 중 `Analyze (rust)`가 563초여서 CodeQL이 남은 critical
  path임을 확인했다.
- Actions의 `Analyze (...)` job 성공은 GitHub Advanced Security의 별도 policy check 성공을 보장하지
  않는다. PR #4310의 보정 전 candidate에서는 세 Analyze job이 성공했지만 app
  `github-advanced-security`의 `CodeQL` check가 high alert로 실패했고, 보정 candidate에서는 같은
  check가 성공했다.
- 따라서 workflow job만 재사용하는 정적 selector는 폐기한다. 기존 PR workflow run의 candidate SHA와
  현재 attempt 시작 시각을 기준으로 동일 SHA의 현재 보안 check를 식별해 missing·pending·failure를
  모두 닫는다. 재실행의 이전 attempt에서 생성된 check도 재사용하지 않는다. 다만 API 실측에서 이 check는
  첫 언어 분석 도착 때 종결되고 이후 JavaScript/TypeScript·Rust 분석으로 갱신되지 않았으므로, 단일
  check에서 뒤에 도착한 언어의 policy 결과까지 추론하지 않는다. 세 Analyze job 성공은 계속 별도 요구한다.
- Rust `cargo build` 뒤에도 CodeQL이 별도 autobuild와 extraction을 수행했다. Stage 5A는 blocking lane을
  바꾸지 않고 `build-mode: none`, `upload: never`인 별도 shadow를 추가해 prebuild 제거 효과와 SARIF
  동등성을 원격에서 측정한다.

기존 rejected WIP에서 재사용할 실측 근거도 이 문서로 옮겼다. #4310 Rust job의 cache 복원 뒤
`cargo build`는 약 52초였지만 analyze가 다시 `database trace-command --index-traceless-dbs`와 Rust
`autobuild.sh`를 실행했다. blocking 기준선의 추출 결과는 성공 1,097파일·오류 7파일이며, 원격 shadow의
coverage·진단 비교 기준으로 쓴다. 보정 전 GHAS check `93182688114`는 실패했고 보정 candidate의 check
`93186154548`은 성공했다. 폐기한 정적 selector의 로컬 테스트 통과 기록은 잘못된 보안 의미를 검증한
것이므로 새 구현 근거로 재사용하지 않는다.

## 구현 범위

- [x] `codeqlResult`에 candidate-bound GitHub Advanced Security `CodeQL` check 확인 추가
- [x] 기존 세 언어 blocking matrix와 Rust prebuild 기준선 보존
- [x] PR non-fast-pass 전용 Rust no-build shadow와 SARIF artifact 추가
- [x] Stage 5A workflow 계약 테스트와 CI test wiring 추가
- [x] focused 검증 통과
- [x] 보정 canary에서 기본 build mode의 no-prebuild 동등성 확인
- [x] blocking lane의 수동 cache·prebuild 제거와 측정용 shadow 정리

Stage 5B의 동적 언어 matrix, required status 변경, 원격 push·PR·canary는 이번 focused 구현 범위 밖이다.

## focused 검증

- TDD RED: 새 Stage 5A 테스트가 보안 check 조회와 shadow job 부재를 각각 검출했다.
- `python3 -m unittest scripts/tests/test_codeql_workflow.py` — 2026-08-09 기준 6/6 통과. 세 Analyze job이
  green이어도 GHAS `CodeQL` check가 `failure`면 fast-pass가 거부되고, 모두 성공하면 재사용되는 실행
  mock과 이전 workflow run attempt의 check를 거부하는 mock을 포함한다.
- Stage 5A·review-only fast-pass·wiring·CI impact·Render Diff·cache sweep Python 계약 테스트 —
  74/74 통과.
- `node --test scripts/tests/ci-impact-classifier.test.cjs` — 28/28 통과.
- `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml` — 통과.
- `git diff --check` — 통과.

변경은 workflow·정적 계약 테스트·문서뿐이며 Rust 제품 코드나 Cargo 계약을 바꾸지 않으므로 Cargo
검증은 적용하지 않는다. 원격 shadow의 1차 duration·SARIF 비교 결과는 아래에 기록한다.

## PR #4341 1차 원격 canary

- **candidate**: `f02aadce71e65b11ca29c6d365484abc0c01204b`
- **CodeQL run**: [31311707469](https://github.com/edwardkim/rhwp/actions/runs/31311707469)
- **결론**: workflow·세 Analyze job·GHAS `CodeQL`·shadow가 모두 성공했지만, no-build 활성화 근거는
  불충분하다.

### 시간

| 구간 | Blocking Rust | No-build shadow | 차이 |
| --- | ---: | ---: | ---: |
| 전체 job | 704초 | 658초 | -46초 (-6.5%) |
| checkout | 41초 | 34초 | -7초 |
| CodeQL init | 16초 | 29초 | +13초 |
| Rust toolchain | 2초 | 1초 | -1초 |
| cargo cache 복원 + 사전 build | 62초 | 0초 | -62초 |
| analyze | 576초 | 585초 | +9초 |

양쪽 analyze는 모두 내부 `autobuild.sh`를 실행했다. blocking은 사전 cache 복원 13초와 `cargo build`
49초를 추가로 썼고, no-build analyze 자체는 더 빠르지 않았다. 따라서 관측된 46초는 `build-mode: none`
효과라기보다 사전 build 제거 효과에 가깝다.

### 추출·SARIF

| 항목 | Blocking Rust | No-build shadow |
| --- | ---: | ---: |
| CodeQL CLI | 2.26.2 | 2.26.2 |
| 성공 추출 Rust 파일 | 1,097 | 1,097 |
| 오류 추출 Rust 파일 | 7 | 3 |
| raw diagnostic message | 2 | 2 |
| raw SARIF artifact | 없음 | 있음 |

shadow artifact `rust-no-build-sarif-31311707469-1`은 압축 95,824바이트, raw 1.4MiB다. 1,920개 artifact,
32개 fingerprinted result를 포함하며 `rust/hard-coded-cryptographic-value` 31건과
`rust/weak-cryptographic-algorithm` 1건이다. 성공 추출 수는 같지만 오류 수가 달라 database 동등성을
단정할 수 없다.

blocking Code Scanning analysis `1591823460`은 PR baseline 처리 뒤 `results_count=0`이고 API로 받은
SARIF도 result·artifact가 제거된 server-processed 형태다. 따라서 shadow의 32개 raw result와 blocking의
raw fingerprint를 직접 비교할 수 없다.

### annotation과 판정

shadow check의 annotation 3건은 PR file coverage 중단 안내, CLI 2.26.2 fallback, CodeQL Action feature
API 권한 부재다. 마지막 항목은 shadow에 `security-events` 권한이 없어서 blocking과 feature 입력이 달랐음을
뜻한다.

1차 canary만으로 `build-mode: none`을 활성화하지 않는다. 다음 측정에서는 다음을 모두 만족해야 한다.

1. blocking analyze도 raw Rust SARIF를 artifact로 보존한다.
2. shadow permissions를 blocking과 같게 선언하고 feature API 경고가 사라지는지 확인한다. fork token
   제한으로 경고가 계속되면 동등한 A/B가 아니므로 활성화하지 않는다.
3. 기본 build mode에서 cargo cache·사전 build만 제거한 shadow로 변수를 하나만 바꾼다.
4. 같은 SHA의 raw result·fingerprint, artifact URI, 성공·오류 추출 수와 duration을 비교한다.

이 보정 canary가 동등성을 확인하기 전에는 blocking Rust lane의 cache·prebuild를 제거하지 않고 Stage 5B
동적 언어 matrix로 넘어가지 않는다.

## PR #4341 보정 canary 구현

1차 측정의 비교 불능 요소를 다음처럼 제거했다.

- blocking matrix는 기본 build mode, Rust cache·수동 `cargo build`, Code Scanning upload를 유지한다.
  analyze의 CodeQL CLI SARIF를 `rust-blocking-results`에 출력하고 Rust matrix job에서만
  `rust-blocking-sarif-*` artifact로 7일 보존한다.
- shadow는 `build-mode: none`을 제거해 blocking과 같은 기본 build mode를 사용하고,
  `security-events: write`, `contents: read`를 동일하게 선언한다.
- shadow에서는 cache·수동 `cargo build`만 생략한다. `upload: never`와 별도 raw SARIF artifact를 유지해
  Code Scanning 결과를 오염시키지 않는다. check·artifact 이름은 첫 측정과 구별되도록
  `Rust no-prebuild shadow`, `rust-no-prebuild-sarif-*`로 바꿨다.

계약 테스트는 보정 전 blocking raw artifact와 no-prebuild shadow가 없어 2건 실패하는 RED를 확인했다.
구현 뒤 `python3 -m unittest scripts/tests/test_codeql_workflow.py` 6/6, 연관 Python workflow
계약 테스트 74/74, classifier Node 테스트 28/28이 통과했다. `actionlint`와 `git diff --check`도
통과했다. 같은 run의 두 raw SARIF, 추출 통계, annotation과 duration 비교 결과는 다음 절에 기록한다.

## PR #4341 보정 원격 canary 판정

- **candidate**: `484f6a3286dfd71b61809b95374a0fce31f8d8e9`
- **CodeQL run**: [31313096097](https://github.com/edwardkim/rhwp/actions/runs/31313096097)
- **결론**: 기본 build mode의 no-prebuild gate 통과. `build-mode: none`은 활성화하지 않는다.

### 시간

| 구간 | Blocking Rust | No-prebuild shadow | 차이 |
| --- | ---: | ---: | ---: |
| 전체 job | 701초 | 642초 | -59초 (-8.4%) |
| checkout | 36초 | 37초 | +1초 |
| CodeQL init | 15초 | 15초 | 0초 |
| Rust toolchain | 1초 | 1초 | 0초 |
| cargo cache 복원 + 수동 build | 60초 | 0초 | -60초 |
| analyze | 582초 | 579초 | -3초 |

두 analyze는 CodeQL CLI 2.26.2, 기본 build mode, `database trace-command --index-traceless-dbs`,
Rust `autobuild.sh`를 동일하게 사용했다. analyze 시간은 사실상 같고 전체 59초 절감은 blocking의 cache
복원 10초와 수동 `cargo build` 50초를 제거한 결과다.

### raw SARIF 동등성

| 항목 | Blocking Rust | No-prebuild shadow | 판정 |
| --- | ---: | ---: | --- |
| 전체 alert result | 32 | 32 | 전체 object 일치 |
| hard-coded cryptographic value | 31 | 31 | 일치 |
| weak cryptographic algorithm | 1 | 1 | 일치 |
| partial fingerprint | 32 | 32 | 전부 일치 |
| 성공 추출 파일 | 1,097 | 1,097 | 일치 |
| artifact URI | 1,104 | 1,100 | generated 4개 차이 |
| extraction warning | 19 | 15 | generated 4개 차이 |
| 추출 LOC | 504,259 | 504,237 | 22줄 차이 |
| unresolved macro | 63 | 63 | 일치 |
| raw diagnostic message | 2 | 2 | 일치 |

32개 result는 message·location·code flow·related location·fingerprint를 포함한 전체 SARIF object가
동일하다. 공통 artifact 1,100개도 비의미적 배열 index를 제외한 metadata가 모두 같다. blocking에만
있는 네 artifact는 다음 `target/` 생성 파일이다.

- `target/debug/build/serde-252a9bbeccb60cd9/out/private.rs`
- `target/debug/build/serde-8c63de14314a3a66/out/private.rs`
- `target/debug/build/serde_core-3b35127d46588d93/out/private.rs`
- `target/debug/build/serde_core-b2bf78d088e03c97/out/private.rs`

네 파일은 모두 `semantic analyzer unavailable (not included in files loaded from manifest)` warning을
남겼고 alert는 만들지 않았다. 즉 수동 prebuild가 추가한 차이는 repository source coverage가 아니라
실패한 generated dependency artifact 네 개다. 양쪽 check annotation도 0건이라 1차 측정의 feature API
권한 차이는 해소됐다.

### 활성화 경계

blocking `Analyze (rust)`에서 cargo cache restore/save와 수동 `cargo build`를 제거해도 alert·fingerprint와
유효한 source extraction coverage가 유지된다는 gate는 통과했다. 기본 build mode와 CodeQL 내부
`autobuild.sh`는 유지한다. 1차 canary의 `build-mode: none`은 이 판정에 포함하지 않는다.

PR #4341의 측정 구성은 그대로 merge하지 않고, blocking check identity와 정상 Code Scanning upload를
유지한 채 수동 cache·prebuild를 제거했다. 측정용 shadow와 두 raw SARIF artifact도 정리했다.

## PR #4341 최종 no-prebuild 전환

- 기존 세 언어 matrix, `Analyze (rust)` 이름, `security-events: write`, 기본 build mode, stable Rust
  toolchain과 정상 Code Scanning upload를 유지했다.
- Rust cargo cache restore/save와 수동 `cargo build`를 제거했다. CodeQL 내부 `autobuild.sh`는 기본
  analyze에서 계속 실행된다.
- `rust-no-prebuild-shadow`와 blocking·shadow raw SARIF output·artifact를 제거해 canary 비용과 임시
  산출물을 남기지 않는다.
- 최종 계약 테스트는 수동 cache·build와 측정 shadow가 남아 있어 2건 실패하는 RED를 확인한 뒤 6/6
  통과했다. 연관 Python workflow 계약 74/74, classifier Node 테스트 28/28, `actionlint`,
  `git diff --check`도 통과했다.

최종 PR CI에서 기존 세 Analyze job과 GHAS `CodeQL` check가 성공했고, 아래 실측으로 Stage 5A
코드·CI gate를 통과로 판정했다.

## PR #4341 최종 CI 판정

- **candidate**: `c2674bd336a26448d1673f7f70389cb8fc2a0ce8`
- **CodeQL run**: [31314188222](https://github.com/edwardkim/rhwp/actions/runs/31314188222)
- **CI run**: [31314188326](https://github.com/edwardkim/rhwp/actions/runs/31314188326)
- **PR 상태(2026-08-09 당시 참고값)**: `MERGEABLE / CLEAN`, 모든 check 성공, Draft, review 없음

### 최종 Rust 시간

| 구간 | 보정 blocking | 보정 shadow | 최종 blocking |
| --- | ---: | ---: | ---: |
| 전체 job | 701초 | 642초 | 542초 |
| checkout | 36초 | 37초 | 38초 |
| CodeQL init | 15초 | 15초 | 15초 |
| Rust toolchain | 1초 | 1초 | 1초 |
| cargo cache + 수동 build | 60초 | 0초 | 0초 |
| analyze | 582초 | 579초 | 480초 |

최종 job은 보정 blocking보다 159초(22.7%) 짧고, 같은 기본 build mode의 보정 shadow보다도 100초
짧았다. 그러나 analyze 자체가 shadow보다 99초 짧아 runner·CodeQL 실행 편차가 섞였다. 구현에 귀속할
보수적 효과는 같은 run A/B에서 수동 cache·build 60초를 제거해 확인한 59초(8.4%)다.

### 보안·운영 gate

- `Analyze (javascript-typescript)`, `Analyze (python)`, `Analyze (rust)`와 별도 GHAS `CodeQL` check가
  모두 성공했다.
- 최종 Rust는 CodeQL CLI 2.26.2, 기본 build mode,
  `database trace-command --index-traceless-dbs`, 내부 `rust/tools/autobuild.sh`를 사용했다.
- `Analyze (rust)`와 GHAS `CodeQL` annotation은 모두 0건이다.
- Rust Code Scanning analysis `1591906480`은 `12:57:08Z`에 25개 규칙·결과 0건으로 처리됐다.
- canary용 Actions artifact는 0개라 shadow·raw SARIF 정리가 확인됐다.

Stage 5A 코드·CI gate는 통과다. PR #4341 제목·본문도 최종 no-prebuild 동작과 canary 근거로 보정했다.
2026-08-09 당시에는 Draft 상태를 유지하고 이후 review 요청을 작업지시자 gate로 남겼다.

## PR #4341 self-review 보정

- **review**: [#4898256773](https://github.com/edwardkim/rhwp/pull/4341#pullrequestreview-4898256773)
- **최신 devel**: `0664e6568e9bc5a50ff6472db8f9eb5825d569c0`
- **병합 결과**: `.github/workflows/ci.yml`의 계약 테스트 목록 한 곳만 충돌했다. PR의 CodeQL 테스트와
  devel의 Docker·release installer·release package·setup 테스트를 모두 보존했다.
- **GHAS 범위**: candidate `c2674bd33`에서 단일 GHAS `CodeQL` check는 Python analysis와 같은
  `12:49:58Z`에 시작해 `12:50:00Z`에 끝났다. JavaScript/TypeScript analysis는 `12:50:28Z`, Rust
  analysis는 `12:57:08Z`에 생성됐으므로 이 단일 check에서 뒤의 두 언어 policy 결과를 추론하지 않는다.
  세 Analyze job 성공과 단일 GHAS check 성공은 각각 계속 요구한다.
- **코드 정리**: check-run에 없는 `created_at` fallback, `Date.parse(0)`, 도달 불가능한 identity mismatch
  분기를 제거했다. `started_at` 누락·이전 attempt는 현재 check 부재로 처리해 full CodeQL로 닫는다.
- **장기 계약 이름**: `scripts/tests/test_codeql_stage5a_workflow.py`를
  `scripts/tests/test_codeql_workflow.py`로 바꾸고 단계명이 merge 후 파일 계약에 남지 않게 했다.
- **focused 검증**: CodeQL 계약 7/7, 연관 Python workflow 계약 86/86, classifier 28/28,
  `actionlint .github/workflows/ci.yml .github/workflows/codeql.yml`, `git diff --check`가 통과했다.
- **남은 gate**: 보정 head의 full CI·CodeQL 통과와 실제 reviewer·작업지시자 승인이다.
