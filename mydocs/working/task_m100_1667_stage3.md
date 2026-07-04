# Task M100 #1667 Stage 3 보고서

## 단계 목표

Build & Test exact cache hit 이후에도 남는 `Dirty rhwp` / `Compiling rhwp`를 분석해, target cache 실효성과
후속 개선 후보를 분리한다.

이번 단계는 관측/분석 단계이며 `.github/workflows/ci.yml` 코드는 변경하지 않았다.

## 분석 대상

최근 successful full `CI / Build & Test` run 중 fast-pass가 아닌 표본을 선정했다.

### `devel` push

| run | head | Build & Test | restore | save |
|-----|------|--------------|---------|------|
| https://github.com/edwardkim/rhwp/actions/runs/28659404284 | `7391325b` | 14m24s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |
| https://github.com/edwardkim/rhwp/actions/runs/28658053891 | `c8c13b17` | 13m50s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |
| https://github.com/edwardkim/rhwp/actions/runs/28656864046 | `772dc2c7` | 13m35s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |

### PR

| run | ref | head | Build & Test | restore | save |
|-----|-----|------|--------------|---------|------|
| https://github.com/edwardkim/rhwp/actions/runs/28660640062 | `refs/pull/1867/merge` | `80a52dd9` | 11m38s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |
| https://github.com/edwardkim/rhwp/actions/runs/28656905057 | `refs/pull/1862/merge` | `ce9c8f80` | 11m19s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |
| https://github.com/edwardkim/rhwp/actions/runs/28656138418 | `refs/pull/1855/merge` | `aa879bd7` | 12m32s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped |

## 관측 요약

모든 표본에서 Build & Test cargo cache는 exact hit였다. cache reservation/read-only/save failure는 관측되지
않았다. 따라서 이번 현상은 cache miss 또는 save 실패가 아니라, restored `target` 내부 산출물의 Cargo
fingerprint 재사용 한계로 보는 것이 맞다.

## step별 compile 패턴

### Build

| 구간 | profile | 관측 |
|------|---------|------|
| `devel` push | `release` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 3m18s~3m32s |
| PR | `release-test` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 1m14s~1m37s |

`devel` push Build는 #1849 정책상 release smoke이므로 `release-test` 산출물과 별도로 보는 것이 맞다.
PR Build는 `release-test`인데도 `dependency info changed` 또는 source file timestamp 차이로 local crate가
dirty가 된다.

### Native Skia Tests

| 구간 | profile/features | 관측 |
|------|------------------|------|
| `devel` push | `release-test`, `native-skia skia` | 3개 모두 `Compiling rhwp`, 2m05s~2m17s |
| PR | `release-test`, `native-skia skia` | 3개 모두 `Compiling rhwp`, 1m44s~2m15s |

이 step은 default feature가 아닌 `native-skia skia` 조합을 사용한다. feature set이 다르므로 별도 산출물이
생기는 것은 현재 구조상 정상 비용으로 분류한다.

### Run Lib Tests

| 구간 | profile | 관측 |
|------|---------|------|
| `devel` push | `release-test` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 1m29s~1m36s |
| PR | `release-test` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 1m15s~1m33s |

`cargo test --lib`는 test harness 산출물을 만들기 때문에 `cargo build --profile release-test` 산출물과 완전히
같지 않다. 다만 source file timestamp 차이로 dirty가 나는 로그가 반복되므로, restored `target`이 local crate
재사용에는 크게 기여하지 못하고 있다.

### Run Integration Tests

| 구간 | profile | 관측 |
|------|---------|------|
| `devel` push | `release-test` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 2m39s~2m50s |
| PR | `release-test` | 3개 모두 `Dirty rhwp` + `Compiling rhwp`, 2m07s~2m45s |

integration test는 별도 test target들을 빌드한다. `dependency info changed`가 반복되고, 일부 run에서는 임의
source file timestamp 차이가 dirty 원인으로 기록됐다.

### Check WASM / Clippy

- `Check WASM target`: `Checking rhwp`, `dev` profile, 12s~16s
- `Clippy`: `Checking rhwp`, `dev` profile, 20s~26s

이 둘은 compile/link 비용과 분리한다.

## 원인 해석

### 1. exact cache hit는 정상이다

모든 표본에서 `Linux-cargo-6a1af...` exact hit와 약 1.56GB restore가 확인됐다. save도 skipped로 정상이다.
따라서 #1664의 restore/save 정책 자체가 실패한 것은 아니다.

### 2. local crate는 checkout timestamp / Cargo dep-info 차이로 계속 dirty가 된다

`Dirty rhwp` 원인은 다음 두 형태로 반복됐다.

- `dependency info changed`
- `the file ... has changed (... after last build at ...)`

두 번째 형태는 cached `target`의 dep-info가 기억하는 source mtime과 현재 checkout된 source mtime이 달라서
Cargo가 local crate를 dirty로 판단하는 전형적인 신호다. 파일명은 run마다 달라지므로 특정 파일 변경 문제가
아니라 checkout/cache fingerprint 문제로 보는 것이 타당하다.

### 3. 일부 compile은 구조상 정상 비용이다

- `devel` push Build의 `release` profile compile은 release smoke 정책 비용이다.
- Native Skia tests는 feature set이 달라 별도 산출물이 필요하다.
- lib test harness와 integration test target은 일반 build 산출물과 다르다.

따라서 모든 `Compiling rhwp`를 제거 목표로 잡으면 안 된다.

### 4. 개선 가능성은 cache action 교체보다 cargo invocation / target cache 정책 쪽이다

현재 관측만으로 `Swatinem/rust-cache`를 바로 도입할 근거는 부족하다. timestamp/fingerprint dirty는 cache
action만 바꾼다고 사라진다고 단정할 수 없다.

우선 검토할 후보는 다음 순서가 더 낫다.

1. Build step의 역할 재검토
   - PR에서 `cargo build --profile release-test`가 뒤의 lib/integration/native-skia test와 어떤 검증 차이를
     갖는지 확인한다.
   - 제거 또는 check 전환은 회귀 가드 추적성과 실패 가시성 영향을 먼저 따져야 한다.
2. lib/integration test invocation 구조 검토
   - `cargo test --profile release-test --lib --tests`처럼 합칠 수 있는지 검토한다.
   - 단, step별 before/after 측정과 실패 위치 가시성이 줄어들 수 있다.
3. target cache 유지/축소 검토
   - local crate는 재사용되지 않지만 dependencies는 상당수 `Fresh`로 남는다.
   - 따라서 `target` 제외는 quota에는 유리하지만 Build & Test 시간이 늘 수 있어 실험 없이 적용하면 안 된다.
4. `Swatinem/rust-cache` 후순위 검토
   - SHA pinning이 필수다.
   - 현재 문제의 직접 원인이 action 종류인지 Cargo fingerprint인지 분리된 뒤 검토한다.

## 판단

이번 분석 기준으로 #1667의 `Dirty rhwp` 문제는 "cache exact hit 실패"가 아니다. `target` cache는 dependency
reuse에는 기여하지만, local `rhwp` crate 산출물은 PR merge ref / checkout timestamp / Cargo dep-info /
profile-feature-test-target 조합 때문에 반복적으로 invalidation 된다.

따라서 바로 cache action을 교체하기보다, 다음 단계는 Build & Test invocation 구조를 바꿀 수 있는지 작은
구현계획서에서 후보를 좁히는 것이다. 1차 후보는 step 이름과 명령을 유지한 채 `Build` 직후
`Run integration tests`와 `Run lib tests`를 먼저 실행하고, `Native Skia tests`는 그 뒤로 미루는 순서
재배치 실험이다.

## 다음 단계 제안

1. `Build` step 직후 기본 feature integration/lib test를 배치하는 순서 재배치 실험을 검토한다.
2. 변경 후보가 승인되면 `.github/workflows/ci.yml` 단일 코드 PR로 진행한다.
3. PR CI에서 `Run integration tests`의 `Dirty rhwp` 감소 여부와 test binary 수를 확인한다.
4. 변경 후보가 위험하거나 이득이 작으면, 이 분석 결과를 measurement 문서에 반영하고 `Swatinem/rust-cache`
   검토로 넘기지 않는다.

이번 단계에서는 branch protection / required check 변경이 없고, `tests/**`, `tests/golden_svg/**`도 변경하지
않았다. 회귀 가드 1:1 추적성은 그대로 보존됐다.
