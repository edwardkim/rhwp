# Task M100 #1667 v3 구현 계획서

## 개요

- 이슈: #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 수행 계획서: `mydocs/plans/task_m100_1667_v3.md`
- 분석 보고서: `mydocs/working/task_m100_1667_stage3.md`
- 작업 브랜치: `task-1667-dirty-rhwp-analysis`
- 구현 범위: Build & Test step 순서 재배치 실험

## 구현 판단

Stage 3 분석에서 최근 `devel` push 3개와 PR 3개의 Build & Test run을 확인했다.

- 모든 표본에서 `Linux-cargo-6a1af...` exact hit가 확인됐다.
- cache save는 skipped였고 reservation/read-only/save failure 경고는 없었다.
- 따라서 남은 `Dirty rhwp` / `Compiling rhwp`는 cache miss 문제가 아니라 Cargo fingerprint와 step 구조 문제다.
- `Swatinem/rust-cache` 즉시 도입, target cache 제거, cache key 분리는 아직 근거가 부족하다.

가장 좁은 실험은 step 이름과 실행 범위를 유지한 채 실행 순서만 바꾸는 것이다.

## 현재 순서

현재 Build & Test의 주요 Rust step 순서는 다음이다.

1. `Format check`
2. `Build`
3. `Check WASM target`
4. `Install native Skia runtime packages`
5. `Native Skia tests`
6. `Run lib tests`
7. `Run integration tests`
8. `Clippy`

이 순서에서는 기본 feature의 `Build` 이후에 WASM check와 native-skia feature test가 먼저 실행된다.
그 뒤 `Run lib tests`, `Run integration tests`가 실행되면서 `Dirty rhwp`가 반복된다.

## 변경안

주요 step 이름과 명령 자체는 유지하고 순서만 다음처럼 바꾼다.

1. `Format check`
2. `Build`
3. `Run integration tests`
4. `Run lib tests`
5. `Check WASM target`
6. `Install native Skia runtime packages`
7. `Native Skia tests`
8. `Clippy`

### 의도

- `Build`는 기본 feature의 `release-test` 산출물을 만든다.
- `Run integration tests`는 기본 feature의 library dependency를 사용하므로 `Build` 직후에 배치하는 편이
  local crate reuse 가능성이 가장 높다.
- `Run lib tests`는 test harness 산출물을 만들기 때문에 별도 compile이 남을 가능성이 높다.
- `Native Skia tests`는 `native-skia skia` feature 조합이 달라 별도 compile이 정상 비용이다.
- 따라서 기본 feature integration test를 feature 변경 step과 lib test harness step보다 먼저 실행해
  불필요한 fingerprint invalidation 가능성을 줄인다.

## 변경하지 않는 것

- cache action은 `actions/cache/restore@v5` / `actions/cache/save@v5` 그대로 유지한다.
- cache key와 cache path는 바꾸지 않는다.
- `Swatinem/rust-cache`는 도입하지 않는다.
- `Build`, `Run integration tests`, `Run lib tests`, `Native Skia tests` 명령은 바꾸지 않는다.
- branch protection / required check는 바꾸지 않는다.
- `tests/**`, `tests/golden_svg/**` 구조와 회귀 가드 파일은 바꾸지 않는다.

## 기대 효과

| step | 기대 |
|------|------|
| `Run integration tests` | `Build` 직후 실행되어 default feature `release-test` 산출물 재사용 가능성 증가 |
| `Run lib tests` | test harness compile은 남을 수 있으나 integration test 뒤로 이동해 normal lib fingerprint 영향 축소 |
| `Native Skia tests` | feature set 차이 때문에 compile은 남는 것을 정상 비용으로 분류 |
| cache quota | cache key/path 변경이 없으므로 quota 리스크 증가 없음 |
| 실패 가시성 | step 이름 유지로 기존 step별 실패 위치 가시성 유지 |

## 리스크

| 리스크 | 대응 |
|--------|------|
| integration test가 먼저 실패하면 lib/native-skia step은 실행되지 않음 | GitHub Actions의 기존 fail-fast step 동작과 동일한 범주. required check 단위는 그대로 유지 |
| integration test 시간이 개선되지 않을 수 있음 | after 관측에서 `Dirty rhwp`, step 시간, 전체 Build & Test 시간을 before와 비교해 판단 |
| hidden dependency가 step 순서에 의존할 수 있음 | 각 step 명령은 그대로 유지하므로 실패 시 원인 가시성이 높음 |
| 메인테이너가 기존 주요 step 순서를 선호할 수 있음 | PR 본문에 "순서 재배치 실험이며 실행 범위는 동일"하다고 명시 |

## 구현 절차

1. `.github/workflows/ci.yml`에서 `Run integration tests`와 `Run lib tests`를 `Build` 직후로 이동한다.
2. `Check WASM target`, `Install native Skia runtime packages`, `Native Skia tests`를 그 뒤로 이동한다.
3. step 이름과 cargo 명령이 바뀌지 않았는지 확인한다.
4. 로컬에서는 YAML diff와 문서 정합성만 확인한다.
5. PR CI에서 before/after를 측정한다.

## PR 관측 기준

PR CI 완료 후 다음 항목을 확인한다.

| 항목 | 확인 기준 |
|------|-----------|
| PR checks 완료 시간 | PR head run 기준으로 기록. update branch 누적 시간과 분리 |
| CI / Build & Test job 시간 | before 표본과 비교 |
| Build | `Dirty rhwp`, `Compiling rhwp`, `Finished ...` 시간 |
| Run integration tests | Build 직후 배치 후 `Dirty rhwp` 감소 여부와 test binary 수 |
| Run lib tests | test harness compile 유지 여부 |
| Native Skia tests | feature set compile 유지 여부 |
| cache restore/save | exact hit / skipped / save failure 없음 |
| cache 크기 | `Linux-cargo-*` size 기록 |
| 실패 원인 가시성 | 실패 시 step명과 stderr line 위치가 유지되는지 |
| runner-minutes | Build & Test job 시간 변화로 추정 |
| branch protection | 변경 없음 |
| 회귀 가드 추적성 | integration test binary 수와 issue 계열 가드 수 유지 |

## 성공 / 중단 기준

성공 기준:

- `Run integration tests`의 `Dirty rhwp` 또는 step 시간이 before 대비 의미 있게 줄어든다.
- 전체 `CI / Build & Test` 시간이 악화되지 않는다.
- test binary 수와 회귀 가드 추적성이 유지된다.
- cache save failure / reservation 경고가 없다.

중단 또는 rollback 기준:

- integration test 수가 줄거나 회귀 가드 1:1 추적성이 깨진다.
- 실패 위치 가시성이 나빠진다.
- Build & Test 시간이 뚜렷하게 악화된다.
- Cargo 로그상 `Dirty rhwp` 패턴이 개선되지 않고 순서 변경의 효과가 없다고 판단된다.

## 후속 판단

이 실험의 효과가 작으면 workflow 구조 변경을 더 밀어붙이지 않는다.
그 경우 #1667의 남은 결론은 다음 중 하나로 정리한다.

- target cache는 dependency reuse에는 유효하지만 local crate reuse에는 한계가 있다고 문서화
- cache key/path 변경은 quota 리스크 때문에 보류
- `Swatinem/rust-cache`는 별도 SHA-pinned 실험 PR에서만 검토
- cleanup 자동화는 #1667 후속 별도 PR로 분리
