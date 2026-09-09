# Task M100 #1665 Stage 1 보고서

## 단계 목표

Build & Test의 1차 병렬화 후보로 Native Skia tests를 별도 worker job으로 분리하고, 기존 `Build & Test`
check 이름은 aggregate gate로 유지하는 workflow 구조를 구현했다.

이번 단계는 `.github/workflows/ci.yml` 구조 변경과 운영 문서 보강이다. measurement / tracking 문서는 PR CI와
merge 후 `devel` push 관측이 끝난 뒤 별도 문서 PR에서 반영한다.

## 변경 파일

- `.github/workflows/ci.yml`
- `mydocs/plans/task_m100_1665.md`
- `mydocs/plans/task_m100_1665_impl.md`
- `mydocs/working/task_m100_1665_stage1.md`
- `mydocs/orders/20260704.md`

## workflow 변경 요약

### 기존 구조

기존에는 `build-and-test` job 하나가 `name: Build & Test`로 다음 step을 순차 실행했다.

1. Format check
2. Build
3. Run default-feature tests
4. Check WASM target
5. Install native Skia runtime packages
6. Native Skia tests
7. Clippy
8. Save cargo registry & build cache

### 변경 후 구조

| job id | name | 역할 |
|--------|------|------|
| `preflight` | `CI preflight` | 기존 fast-pass 판정 유지 |
| `build-default-feature-tests` | `Build default-feature tests` | fmt, Build, default-feature tests, WASM check, Clippy, 단일 cache save writer |
| `native-skia-tests` | `Native Skia tests` | native package install, native-skia feature lib tests |
| `build-and-test` | `Build & Test` | worker 결과를 모으는 aggregate gate |
| `wasm-build` | `WASM Build` | 기존 tag/manual 조건 유지 |

## 구현 세부

### Build default-feature tests

기존 `Build & Test`의 기본 feature 검증 경로를 옮겼다.

- checkout, disk cleanup, Rust toolchain, cargo restore 유지
- `Format check` 유지
- `Build` profile 분기 유지
  - PR: `release-test`
  - non-PR: `release`
- `Run default-feature tests` profile 분기 유지
  - PR / `devel` push: `release-test`
  - main/tag/manual: `release`
- `Check WASM target` 유지
- `Clippy` 유지
- `Save cargo registry & build cache`는 이 job에만 유지

### Native Skia tests

Native Skia는 feature set이 달라 기본 feature 산출물 재사용성이 낮으므로 별도 job으로 분리했다.

- checkout, disk cleanup, Rust toolchain, cargo restore 수행
- native Skia runtime package 설치 수행
- `cargo test --features native-skia skia --lib` profile 분기 유지
- cache save step 없음

### Build & Test aggregate

기존 `Build & Test` check 이름을 유지하기 위해 `build-and-test` job id와 `name: Build & Test`를 aggregate
gate로 바꿨다.

동작:

- `preflight` 실패 시 aggregate fail
- fast-pass PR이면 worker jobs가 skipped 되어도 aggregate success
- 일반 run이면 `Build default-feature tests`와 `Native Skia tests`가 모두 success일 때만 aggregate success
- worker failure/skipped/cancelled 상태는 aggregate 로그의 `::error::`로 표시

## cache 정책

| job | restore | save |
|-----|---------|------|
| `Build default-feature tests` | 기존 cargo key/path | trusted branch exact miss/fallback일 때만 save |
| `Native Skia tests` | 기존 cargo key/path | 없음 |
| `Build & Test` aggregate | 없음 | 없음 |

보존한 것:

- PR restore-only 정책
- `devel` / `main` push에서만 save 허용
- cache key/path
- `Swatinem/rust-cache` 미도입

### Native Skia cache 한계

이번 1차 분리는 cache save writer를 `Build default-feature tests` 하나로 제한한다. 변경 전 단일
`Build & Test` job에서는 trusted branch cache save 시점에 default-feature 산출물과 Native Skia 산출물이 같은
`target/`에 함께 포함될 수 있었다. 변경 후에는 Native Skia job의 `target/` 산출물이 별도 runner에서 생성되고
저장되지 않는다.

이 선택은 같은 cache key에 여러 job이 동시에 save하면서 cache reservation 경합이나 read-only 경고가 재발하는
것을 피하기 위한 보수적 설계다. 대신 Native Skia job은 exact restore 이후에도 필요한 산출물을 다시 빌드할 수
있으며, runner-minutes 또는 Native Skia job 시간이 기대보다 줄지 않을 수 있다.

merge 후 `devel` push와 후속 PR run에서 Native Skia compile 시간, cache hit 여부, queue 대기,
runner-minutes를 관측한 뒤 별도 native-skia cache key 또는 별도 target dir 도입 여부를 판단한다.

## 회귀 가드 보존

이번 변경은 workflow job 구조만 바꿨다.

- `tests/**` 변경 없음
- `tests/golden_svg/**` 변경 없음
- 통합 테스트 파일 통합 없음
- 회귀 가드 명명 규칙 변경 없음
- `cargo test --profile release-test --tests --verbose` 명령 유지
- `cargo test --profile release-test --features native-skia skia --lib --verbose` 명령 유지

따라서 회귀 가드 1:1 추적성은 명령 구조상 보존된다. 실제 test count는 PR CI에서 확인한다.

## PR CI 확인 항목

- `Build default-feature tests` 성공 여부
- `Native Skia tests` 성공 여부
- aggregate `Build & Test` 성공 여부
- fast-pass가 아닌 PR에서 worker jobs가 실행되는지
- PR run에서 `Save cargo registry & build cache`가 skipped 되는지
- Native Skia job에 save step이 없는지
- cache reservation / read-only / save failure 경고가 없는지
- `Run default-feature tests` test count 유지
- `Native Skia tests` 48 tests 유지
- `CI / Build & Test` required check가 aggregate 결과로 유지되는지

## 예상 효과와 리스크

기대 효과:

- Native Skia package install + test 약 2분대가 default-feature 경로와 겹쳐 wall time 감소 가능
- Native Skia 실패가 별도 job으로 더 명확히 드러남

리스크:

- checkout/toolchain/cache restore가 Native Skia job에서 한 번 더 발생해 runner-minutes는 늘 수 있음
- 기존 `Build & Test`가 aggregate gate로 바뀌므로 required check 의미가 달라짐
- aggregate가 worker failure/skipped 상태를 잘못 전달하면 required check 신뢰성이 떨어질 수 있음

## 로컬 검증

| 항목 | 결과 |
|------|------|
| YAML parse | 통과. `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"` |
| `actionlint .github/workflows/ci.yml` | 통과 |
| `git diff --check` | 통과 |

비고:

- Ruby 실행 시 local gem 경고 `Ignoring ffi-1.13.1 because its extensions are not built`가 출력됐지만,
  YAML parse 자체는 `yaml ok`로 통과했다.

## 다음 단계

1. 코드 PR 생성
2. PR CI 관측
3. 필요 시 aggregate 조건 보정
4. Ready for review 전환 판단
