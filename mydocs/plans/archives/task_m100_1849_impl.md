# Task M100 #1849 구현 계획서

## 구현 원칙

- #1849는 #1666 구현을 실패로 보거나 되돌리는 작업이 아니다.
- PR 경로는 #1666의 성공 결과를 유지한다.
- `devel` push 경로만 full `release --tests`에서 `release-test` 전체 회귀 + 제한적 `release` smoke로 조정한다.
- `main` push, tag push, `workflow_dispatch`는 full `release` 검증 경로로 유지한다.
- `Build & Test` job 이름과 required check 표면은 변경하지 않는다.
- `Cargo.toml`, `tests/**`, `tests/golden_svg/**`는 변경하지 않는다.
- `Swatinem/rust-cache` 도입은 #1667, job 병렬화는 #1665 범위로 남긴다.
- 장기 measurement 원천 문서는 CI 관측 후 후속 문서 PR에서 갱신한다.

## 핵심 구현 선택

1차 구현은 수행계획서의 후보 A를 따른다.

| 이벤트 / ref | Build | Native Skia tests | Run lib tests | Run integration tests | 의도 |
|--------------|-------|-------------------|---------------|-----------------------|------|
| `pull_request` | `release-test` | `release-test` | `release-test` | `release-test` | PR 빠른 회귀 검증 유지 |
| `push` `refs/heads/devel` | `release` | `release-test` | `release-test` | `release-test` | merge 후 빠른 전체 회귀 + release build smoke |
| `push` `refs/heads/main` | `release` | `release` | `release` | `release` | main 통합 강검증 |
| tag push `refs/tags/v*` | `release` | `release` | `release` | `release` | 릴리스 검증 |
| `workflow_dispatch` | `release` | `release` | `release` | `release` | 수동 강검증 |

별도 `Release smoke` step은 추가하지 않는다. 기존 `Build` step이 `devel` push에서
`cargo build --release --verbose`를 계속 실행하므로, 이 step을 release smoke로 해석한다.

이 선택은 check 표면과 step 구성을 최소 변경으로 유지하면서 #1666 이후 가장 큰 비용이던 `devel` push
`Run integration tests --release --tests`를 제거한다.

## Stage 1 — profile 선택 조건 정리

`.github/workflows/ci.yml`의 `Build & Test` job에서 cargo step별 조건을 다음처럼 정리한다.

### Build

현재 의미를 거의 유지한다.

- PR: `cargo build --profile release-test --verbose`
- 그 외: `cargo build --release --verbose`

`devel` push에서는 이 step이 release smoke가 되므로 로그에 역할을 드러낸다.

변경 방향:

```bash
if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]]; then
  echo "profile=release-test event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo build --profile release-test --verbose
else
  echo "profile=release event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF} role=release-smoke-or-full"
  cargo build --release --verbose
fi
```

### Native Skia tests

PR과 `devel` push는 `release-test`, 나머지는 `release`로 실행한다.

변경 방향:

```bash
if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]] || [[ "${GITHUB_EVENT_NAME}" == "push" && "${GITHUB_REF}" == "refs/heads/devel" ]]; then
  echo "profile=release-test event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --profile release-test --features native-skia skia --lib --verbose
else
  echo "profile=release event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --release --features native-skia skia --lib --verbose
fi
```

### Run lib tests

PR과 `devel` push는 `release-test`, 나머지는 `release`로 실행한다.

변경 방향:

```bash
if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]] || [[ "${GITHUB_EVENT_NAME}" == "push" && "${GITHUB_REF}" == "refs/heads/devel" ]]; then
  echo "profile=release-test event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --profile release-test --lib --verbose
else
  echo "profile=release event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --release --lib --verbose
fi
```

### Run integration tests

PR과 `devel` push는 `release-test`, 나머지는 `release`로 실행한다.

변경 방향:

```bash
if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]] || [[ "${GITHUB_EVENT_NAME}" == "push" && "${GITHUB_REF}" == "refs/heads/devel" ]]; then
  echo "profile=release-test event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --profile release-test --tests --verbose
else
  echo "profile=release event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF}"
  cargo test --release --tests --verbose
fi
```

## Stage 2 — workflow 주석 보강

조건 분기 바로 앞에 짧은 주석을 추가한다.

기록할 내용:

- PR은 `release-test` 전체 회귀 검증
- `devel` push는 `Build`만 release smoke, test step은 `release-test`
- `main` / tag / manual은 full `release`
- #1849 정책 결정에 따른 분기임

주석은 동작을 설명하는 데 필요한 최소 수준으로 둔다.

## Stage 3 — 로컬 정적 검증

코드 수정 후 다음을 확인한다.

- `git diff --check`
- 가능하면 `actionlint .github/workflows/ci.yml`
- `actionlint`가 없으면 설치하지 않고, YAML 구조와 bash 조건식을 수동 검토한 뒤 한계를 기록한다.
- 변경 파일 확인:
  - `.github/workflows/ci.yml`
  - `mydocs/orders/20260703.md`
  - `mydocs/plans/task_m100_1849.md`
  - `mydocs/plans/task_m100_1849_impl.md`
  - 후속 stage 보고서
- 변경하지 않아야 할 파일 확인:
  - `Cargo.toml`
  - `tests/**`
  - `tests/golden_svg/**`

## Stage 4 — PR CI 관측

PR run에서는 기존 #1666 효과가 유지되는지 확인한다.

| 항목 | 확인 내용 |
|------|-----------|
| PR checks 완료 시간 | run created/updated 기준. 샘플 부족 시 P50/P90 보류 |
| `CI / Build & Test` job 시간 | job started/completed 기준 |
| 주요 step 시간 | Build / Native Skia tests / Run lib tests / Run integration tests |
| cargo profile 로그 | 네 step 모두 `profile=release-test event=pull_request`인지 확인 |
| cache 상태 | restore hit/miss, cache size, PR save skipped |
| 실패 가시성 | 실패 step과 stderr 위치 |
| 회귀 가드 | 해당 PR head 기준 normalized unique test binary와 issue 계열 가드 실행 수 확인 |
| required check | `CI / Build & Test` check 표면 유지 |

PR run의 목표는 #1849 변경이 PR 경로를 악화시키지 않았음을 확인하는 것이다.

## Stage 5 — merge 후 `devel` push 관측

코드 PR merge 후 `devel` push run에서 다음을 확인한다.

| 항목 | 기대값 |
|------|--------|
| Build | `profile=release ... role=release-smoke-or-full`, `cargo build --release --verbose` |
| Native Skia tests | `profile=release-test event=push ref=refs/heads/devel` |
| Run lib tests | `profile=release-test event=push ref=refs/heads/devel` |
| Run integration tests | `profile=release-test event=push ref=refs/heads/devel` |
| cache | exact hit 또는 save 정책 정상. reservation/read-only/save failure 경고 없음 |
| 회귀 가드 | 해당 `devel` SHA 기준 전체 회귀 가드 1:1 추적성 유지 |

측정 비교:

- #1666 merge 후 `devel` `Build & Test` P50 56m55s / P90 59m14s 대비 감소 여부
- #1666 merge 후 `devel` `Run integration tests` P50 42m27s / P90 44m14s 대비 감소 여부
- Build release smoke 시간이 #1664 exact-hit 기준선 3m32s 및 #1666 이후 3m35s P50 범위와 크게 다르지 않은지 확인

## Stage 6 — main/tag/manual 경로 의미 확인

실제 main push나 tag push를 새로 만들지는 않는다. PR diff와 workflow 조건으로 다음을 확인한다.

- `refs/heads/main` push는 test step이 `release`로 남는다.
- `refs/tags/v*` push는 test step이 `release`로 남는다.
- `workflow_dispatch`는 test step이 `release`로 남는다.
- `workflow_dispatch`를 `devel` ref로 실행해도 수동 강검증 의미를 유지하기 위해 `push && refs/heads/devel`
  조건으로만 `devel` 최적화 분기를 적용한다.
- tag push에서는 기존 `wasm-build` job 조건도 유지된다.

main/tag/manual에서 full release 검증이 부족하다고 판단되면, nightly/scheduled full `release --tests` 추가는 별도 후속 정책 이슈로 분리한다.

## Stage 7 — 기록과 PR 구성

코드 PR에 포함할 문서:

- 수행계획서: `mydocs/plans/task_m100_1849.md`
- 구현계획서: `mydocs/plans/task_m100_1849_impl.md`
- 단계 보고서: `mydocs/working/task_m100_1849_stage1.md`

코드 PR에 포함할 코드:

- `.github/workflows/ci.yml`

후속 문서 PR로 분리할 문서:

- `mydocs/report/task_m100_1849_measurement.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`의 after measurement 갱신

코드 PR 본문에는 "measurement 원천 문서는 CI 관측 후 후속 문서 PR에서 갱신"한다고 명시한다.

## 합격 기준

- PR 경로가 계속 `release-test` 전체 회귀 검증을 수행한다.
- `devel` push에서 `Build`는 release smoke로 남고, 세 test step은 `release-test`로 실행된다.
- `main` / tag / manual 경로의 full `release` 검증은 유지된다.
- `CI / Build & Test` check 표면은 바뀌지 않는다.
- `Cargo.toml`, `tests/**`, `tests/golden_svg/**`는 변경되지 않는다.
- cache restore/save 정책은 #1664 이후 상태를 유지한다.
- #1668 공통 측정 기준으로 PR run과 `devel` push run을 보고할 수 있다.

## 롤백 기준

- PR에서 `release-test` 전체 회귀 검증이 깨지는 경우
- `devel` push에서 release smoke가 실행되지 않는 경우
- `main` / tag / manual 경로에서 full `release` 검증이 사라지는 경우
- required check 이름 또는 branch protection 표면이 바뀌는 경우
- 회귀 가드 1:1 추적성이 깨지는 경우

롤백은 #1849에서 추가한 `devel` test step 조건을 제거하고, #1666 직후의 "PR은 `release-test`, 그 외는 `release`" 분기로 되돌리는 방식으로 수행한다.

## 승인 요청 지점

이 구현 계획서 승인 후 `.github/workflows/ci.yml` 변경과 Stage 1 보고서 작성을 진행한다.
