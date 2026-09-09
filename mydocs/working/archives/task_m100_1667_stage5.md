# Task M100 #1667 Stage 5 보고서

## 단계 목표

#1872 1차 CI 관측에서 발견된 `Run lib tests` 중복 실행 여부를 확인하고, 기본 feature test step을
하나로 정리했다.

## 변경 파일

- `.github/workflows/ci.yml`
- `mydocs/plans/task_m100_1667_impl_v4.md`
- `mydocs/working/task_m100_1667_stage5.md`
- `mydocs/orders/20260704.md`

## 관측 내용

#1872 1차 PR run에서 다음이 확인됐다.

- `Build` step에서 `Compiling rhwp`가 발생했다.
- `Run integration tests`에서 다시 `Compiling rhwp`가 발생했다.
- `Run lib tests`에서는 `Fresh rhwp`가 출력됐다.
- `Run integration tests`와 `Run lib tests` 모두 `2081 tests`를 실행했다.

따라서 `Run lib tests`는 별도 compile을 유발하는 주 원인은 아니었지만, 같은 lib test harness를
한 번 더 실행하는 중복 step이었다.

## 로컬 target 확인

로컬에서 다음 명령으로 Cargo target 선택을 확인했다.

```bash
cargo test --profile release-test --tests --no-run --message-format=json
cargo test --profile release-test --lib --no-run --message-format=json
```

확인 결과:

| 항목 | 결과 |
|------|------|
| `tests/*.rs` 파일 수 | 181 |
| issue 계열 `tests/issue*.rs` 파일 수 | 150 |
| `--tests --no-run` executable artifact | 186 |
| `--tests --no-run` test artifact | 181 |
| `--tests --no-run`에 포함된 package artifact | `rhwp` |
| `--lib --no-run`에만 존재하는 고유 target | 없음 |

해석:

- 현재 저장소 구조에서 `cargo test --tests`는 `tests/*.rs` integration test와 `rhwp` lib test executable을 포함한다.
- 별도 `cargo test --lib` step은 `--tests`가 이미 실행한 lib tests를 다시 실행한다.

## 구현 내용

Build & Test에서 `Run lib tests` step을 제거했다.

변경 후 주요 순서:

1. `Build`
2. `Run default-feature tests`
3. `Check WASM target`
4. `Install native Skia runtime packages`
5. `Native Skia tests`
6. `Clippy`

## 변경하지 않은 것

- 기존 `Run integration tests` 명령인 `cargo test --tests`는 유지하되, 실제 역할에 맞춰
  step 이름을 `Run default-feature tests`로 바꿨다.
- `Native Skia tests` 명령은 유지했다.
- cache restore/save action, key, path는 변경하지 않았다.
- branch protection / required check는 변경하지 않았다.
- `tests/**`, `tests/golden_svg/**`, issue 계열 회귀 가드 파일은 변경하지 않았다.

## 기대 효과와 한계

기대 효과:

- 기본 feature lib tests는 `Run default-feature tests`에서 1회만 실행된다.
- `Run lib tests` step의 중복 실행 시간이 제거된다.
- `Run default-feature tests` 이후 `Fresh rhwp` 상태에서 같은 2081개 tests를 다시 돌리는 낭비가 사라진다.

한계:

- `Run default-feature tests`의 `Compiling rhwp` 1회는 test harness 산출물 생성 비용으로 남을 수 있다.
- `Native Skia tests`는 `native-skia skia` feature set이 달라 별도 compile이 정상 비용이다.
- 이번 PR은 cache key/path 또는 cache action 교체를 다루지 않는다.

## PR CI 관측 기준

후속 PR CI에서 다음을 확인한다.

- `Run lib tests` step 제거 확인
- `Run default-feature tests` test count 유지
- integration test executable 수와 issue 계열 회귀 가드 추적성 유지
- `CI / Build & Test` job 시간 변화
- cache exact hit / save skipped / save failure 없음 유지
- branch protection / required check 변경 없음

## 후속 이관

- Native Skia tests의 별도 compile 및 wall time 영향은 #1665 job 병렬화 검토에서 다룬다.
- target cache가 local crate 재사용에 주는 한계는 #1667 최종 measurement 문서에서 정리한다.
