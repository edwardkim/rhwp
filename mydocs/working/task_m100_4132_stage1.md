# task_m100_4132 Stage 1 — 함수 게이트 native-skia test 파일 규약화

- **이슈**: [#4132](https://github.com/edwardkim/rhwp/issues/4132)
- **PR**: [#4310](https://github.com/edwardkim/rhwp/pull/4310)
- **브랜치**: `issue-4132-native-cli-exit`
- **분기 기준**: `upstream/devel` `f94fe5e4f`
- **계획서**: [수행](../plans/task_m100_4132.md) / [구현](../plans/task_m100_4132_impl.md)
- **상태**: Draft PR 게시, 원격 CI 대기
- **기록일**: 2026-08-09 KST

## 1. #4170 후처리와 선행 규약

PR #4170은 merge commit `7230a7ce4558d0596700df05e8396e6c28d278d3`으로 `devel`에 반영됐다.
merge 뒤 push CI와 CodeQL이 성공했고 Native Skia job은 6분 19초(379초)로 기존 368~382초 기준 안이었다.
이 근거를 #4040에 남기고 완료 종료했다.

#4170이 추가한 파일 게이트 자동 발견과 job·classifier 양방향 계약을 #4132의 구현 기반으로 사용했다.

## 2. RED 전수 검사에서 범위를 1건에서 2건으로 보정

함수 단위 native-skia `#[test]`를 전수 검사하자 세 건이 발견됐다.

| 파일 | 함수 | 착수 시 상태 |
| --- | --- | --- |
| `cli_exit_codes.rs` | `export_png_follows_the_same_contract` | 누락 |
| `issue_1144.rs` | `issue_1144_skia_png_export_entrypoint_does_not_freeze_filename_context` | 누락 |
| `issue_2225_missing_picture_placeholder.rs` | `issue_2225_export_png_defaults_to_print_equivalent_skia_profile` | 이미 Native job·classifier 배선 |

첫 RED는 기존 `issue_2225`만 허용하는 기대에 `cli_exit_codes`와 `issue_1144` 두 항목이 추가로 나타나
실패했다. #4132 코멘트로 범위를 보정하고 두 누락을 모두 선택지 B로 처리했다.

두 native 함수를 새 파일로 옮긴 뒤 두 번째 RED에서는 #4170의 파일 게이트 계약이
`cli_exit_codes_native`, `issue_1144_native`를 Native job 누락 목록으로 정확히 지목했다. workflow와
classifier를 배선한 뒤 같은 계약이 GREEN이 됐다.

## 3. 구현

### 3.1 파일 게이트 target 분리

- `tests/cli_exit_codes_native.rs`: `export-png`의 exit 2·1 계약 1건
- `tests/issue_1144_native.rs`: Skia PNG export 뒤 filename cache 무효화 계약 1건

두 파일은 crate 수준 `cfg(all(not(wasm32), feature = "native-skia"))`를 사용한다. default archive에서는
파일 전체가 cfg-out되고 Native Skia job에서만 실행된다.

### 3.2 helper 공유

- `tests/support/cli_exit_code_support.rs`: CLI 실행, 임시 경로, stdout/stderr 설명과 exit assertion
- `tests/support/issue_1144_support.rs`: filename footer document fixture와 PageLayerTree text 수집

원본 default target과 새 native target이 같은 helper를 컴파일하므로 행위 구현을 복제하지 않는다.
`CARGO_BIN_EXE_rhwp`는 #3289 계약대로 런타임 값을 먼저 사용하고 컴파일타임 경로를 fallback으로 쓴다.

### 3.3 workflow·classifier·재발 가드

- Native Skia job의 release-test·release 두 경로에 새 target 2개를 동일하게 추가했다.
- classifier native 소유 목록에 두 target과 두 `#[path]` support를 추가했다. 각 단독 변경은
  `classified:native-skia-rust`로 판정한다.
- 함수 게이트 발견기는 Rust 문자열·주석과 중첩 함수를 제외하고 최상위 outer cfg + test 조합을 찾는다.
  native-skia 비활성 상태에서 반드시 거짓인 cfg만 대상으로 하며, 이미 정상 배선된 `issue_2225` 한 건만
  명시적 예외로 고정한다.
- 새 두 파일은 #4170의 파일 게이트 자동 발견 known set에도 추가했다.
- file-gated target이 참조하는 `#[path]` support를 자동 발견해 classifier 소유를 강제한다. 이 가드가
  없으면 helper만 고치는 PR에서 Native job이 skip되는 새 공백이 생길 수 있다.

## 4. focused·전체 회귀 검증

| 검증 | 결과 |
| --- | --- |
| `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` | 25/25 통과 |
| workflow 계약 5개 파일 결합 실행 | 66/66 통과 |
| `node --test scripts/tests/ci-impact-classifier.test.cjs` | 28/28 통과 |
| default `cli_exit_codes` | 10/10 통과 |
| native `cli_exit_codes_native` | 1/1 통과 |
| default `issue_1144` | 4/4 통과 |
| native `issue_1144_native` | 1/1 통과 |
| `cargo fmt --check` | 통과 |
| `actionlint .github/workflows/ci.yml` | 통과 |
| `node --check scripts/ci-impact-classifier.cjs` | 통과 |
| `git diff --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 최신 devel rebase 후 전체 통과(exit 0, compile 2분 19초; lib 3,361 통과·13 ignored 포함) |
| `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` | 최신 devel rebase 후 통과(24.88초) |

첫 native build는 샌드박스에서 Skia 바이너리 다운로드 DNS가 차단돼 `curl` code 6으로 실패했다. 승인된
네트워크 재시도에서 공식 Skia 바이너리를 내려받은 뒤 native 두 target이 모두 통과했다. 제품·테스트 실패가
아니라 의존성 다운로드 경계였다.

focused 결과 공유 뒤 작업지시자 승인을 받아 장시간 release-test 전체와 clippy도 순차 실행했다.
`cargo-nextest`가 설치돼 있지 않아 계획서에 적은 `cargo test --profile release-test --tests` 경로를 사용했다.
접두사를 제거한 `issue-4132-native-cli-exit` 브랜치를 push하고 Draft PR #4310을 생성했다.
원격 PR CI는 아직 완료되지 않았다.

## 5. 다음 게이트

1. PR #4310 원격 CI에서 새 native target 두 건의 실제 실행을 확인한다.
2. Native Skia job 총 소요시간 증가폭을 확인하고 Draft를 review-ready로 전환한다.
3. #4132 merge 뒤 보존 중인 #3790 Stage 5 worktree를 최신 `devel`로 갱신한다.

## 6. PR #4310 self-review 보정

초기 self-review에서 재발 감시가 최상위 함수만 읽는 공백과 CodeQL의 비효율 정규식 경고를 확인했다.
outer attribute 전체를 중첩 반복 정규식으로 잡는 대신 함수·inline module body와 직전 attribute를 구조적으로
읽도록 바꿨다. 다음 세 우회와 support 경로 오탐을 회귀 입력으로 추가했다.

- inline module 안의 함수 단위 native-skia cfg
- native-skia cfg가 붙은 inline module 안의 test
- `cfg_attr(feature = "native-skia", test)`
- 주석 처리된 `#[path = "..."]`

`#[doc = "a]b"]` 사례는 문자열 마스킹 뒤 정상 탐지됨을 재확인해 초기 self-review의 해당 지적을 철회한다.
계획서의 실제 branch와 분기 기준도 `issue-4132-native-cli-exit` / `f94fe5e4f`로 정정했다.
