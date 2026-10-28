# Task M100 #1667 v4 구현 계획서

## 개요

- 이슈: #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 작업 브랜치: `task-1667-dirty-rhwp-analysis`
- 대상 PR: #1872
- 구현 범위: Build & Test의 중복 `Run lib tests` step 제거

## 배경

#1872 1차 CI 관측에서 `Build` 직후 `Run integration tests`를 배치하자 다음 현상이 확인됐다.

- `Run integration tests`에서 `Compiling rhwp`가 1회 발생했다.
- 이후 `Run lib tests`에서는 `Fresh rhwp`가 출력됐다.
- 두 step 모두 동일하게 `2081 tests`를 실행했다.
- `Run lib tests`는 compile 비용은 거의 없었지만, 같은 lib test harness를 다시 실행하는 비용이 남았다.

따라서 현재 남은 개선 대상은 cache miss가 아니라 test target 중복 실행이다.

## Cargo target 확인

로컬에서 target 선택만 확인했다.

```bash
cargo test --profile release-test --tests --no-run --message-format=json
cargo test --profile release-test --lib --no-run --message-format=json
```

확인 결과:

| 명령 | executable artifact | 해석 |
|------|---------------------|------|
| `--tests --no-run` | `181 test`, `4 bin`, `1 cdylib+rlib` | `tests/*.rs` integration test와 `rhwp` lib test executable을 포함 |
| `--lib --no-run` | `rhwp` `cdylib+rlib` | `--tests`에 없는 고유 실행 대상 없음 |

`comm` 비교에서도 `--lib`에만 존재하는 고유 target은 없었다.
즉 `cargo test --tests`는 현재 저장소 구조에서 기본 feature의 lib test executable을 포함한다.

## 변경안

Build & Test 주요 순서를 다음처럼 정리한다.

1. `Build`
2. `Run default-feature tests`
3. `Check WASM target`
4. `Install native Skia runtime packages`
5. `Native Skia tests`
6. `Clippy`

제거 대상:

- `Run lib tests`

## 변경하지 않는 것

- 기존 `Run integration tests` 명령인 `cargo test --tests`는 유지하되, 실제 역할에 맞춰
  step 이름을 `Run default-feature tests`로 바꾼다.
- native-skia 전용 `cargo test --features native-skia skia --lib`는 유지한다.
- cache restore/save action, key, path는 변경하지 않는다.
- branch protection / required check는 변경하지 않는다.
- `tests/**`, `tests/golden_svg/**`, issue 계열 회귀 가드 파일은 변경하지 않는다.

## 기대 효과

| 항목 | 기대 |
|------|------|
| 기본 feature lib tests | `Run default-feature tests` 안에서 1회 실행 |
| `Run lib tests` step 시간 | 중복 step 제거로 0 |
| `Compiling rhwp` | Build & Test 기본 feature test harness 생성 1회는 남을 수 있음 |
| Native Skia tests | feature set이 달라 별도 compile 유지 |
| 회귀 가드 추적성 | `--tests`가 `tests/*.rs`와 lib test executable을 포함하므로 보존 |
| 실패 가시성 | default-feature test 실패는 `Run default-feature tests` step에 집중 |

## 리스크와 대응

| 리스크 | 대응 |
|--------|------|
| `--tests`가 향후 lib test를 포함하지 않는 형태로 바뀔 수 있음 | after 관측에서 test count와 Cargo executable artifact를 같이 기록 |
| step 이름에서 lib test 전용 위치가 사라짐 | `Run default-feature tests`로 이름을 바꿔 lib+integration 기본 feature test 담당 범위를 명시 |
| failure 위치가 넓어질 수 있음 | 현재도 같은 2081개 lib tests가 같은 harness에서 실행되므로 실질 가시성 저하는 작다고 판단 |

## 검증 기준

PR CI 완료 후 다음을 확인한다.

- `Run lib tests` step이 사라졌는지
- `Run default-feature tests`에서 lib tests `2081 tests`와 integration test executables가 유지되는지
- `CI / Build & Test` job 시간이 1차 #1872 run보다 줄었는지
- cache exact hit / save skipped / save failure 없음이 유지되는지
- branch protection / required check 변경이 없는지
- 회귀 가드 추적성이 유지되는지

## 후속 판단

- `Run lib tests` 제거 후에도 `Compiling rhwp` 1회는 정상 비용으로 본다.
- Native Skia tests의 별도 compile은 #1665 job 병렬화 검토 대상으로 이관한다.
- cache action 교체나 target cache path 변경은 이번 PR 범위에 포함하지 않는다.
