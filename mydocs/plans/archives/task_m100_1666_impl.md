# Task M100 #1666 구현 계획서

## 구현 원칙

- #1666은 PR `Build & Test`의 Rust profile을 `release-test` 중심으로 전환한다.
- `Cargo.toml`의 profile 정의는 이미 있으므로 변경하지 않는다.
- `tests/`, `tests/golden_svg/`, 회귀 가드 명명 규칙은 변경하지 않는다.
- `Build & Test` job 이름과 required check 표면은 유지한다.
- job 병렬화, `Swatinem/rust-cache`, nextest 도입은 하지 않는다.
- 장기 measurement 원천 문서는 CI 관측 후 후속 문서 PR에서 갱신한다.
- 코드 PR에는 `.github/workflows/ci.yml`과 #1666 하이퍼-워터폴 절차 문서만 포함한다.

## 핵심 구현 선택

기존 `Build & Test` job의 step 이름은 유지하되, 각 cargo command 안에서 이벤트별 profile을 분기한다.

| 이벤트 | profile | 의도 |
|--------|---------|------|
| `pull_request` | `release-test` | PR 피드백 루프 단축, profile 불일치 감소 |
| `push` to `devel` / `main` | `release` | release-grade 검증 보완 |
| tag push | `release` | 릴리스 검증 유지 |
| `workflow_dispatch` | `release` | 수동 강검증 경로 유지 |

이 방식은 `Build & Test` check 이름을 바꾸지 않고, PR과 trusted event의 검증 의미를 분리한다.

## Stage 1 — PR profile 전환

`.github/workflows/ci.yml`의 `Build & Test` job에서 다음 step을 조건부 profile 실행으로 바꾼다.

### Build

현재:

```yaml
run: cargo build --release --verbose
```

변경 방향:

```yaml
run: |
  if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]]; then
    cargo build --profile release-test --verbose
  else
    cargo build --release --verbose
  fi
```

### Native Skia tests

현재:

```yaml
run: cargo test --release --features native-skia skia --lib --verbose
```

변경 방향:

```yaml
run: |
  if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]]; then
    cargo test --profile release-test --features native-skia skia --lib --verbose
  else
    cargo test --release --features native-skia skia --lib --verbose
  fi
```

### Run lib tests

현재:

```yaml
run: cargo test --release --lib --verbose
```

변경 방향:

```yaml
run: |
  if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]]; then
    cargo test --profile release-test --lib --verbose
  else
    cargo test --release --lib --verbose
  fi
```

### Run integration tests

현재:

```yaml
run: cargo test --profile release-test --tests --verbose
```

변경 방향:

```yaml
run: |
  if [[ "${GITHUB_EVENT_NAME}" == "pull_request" ]]; then
    cargo test --profile release-test --tests --verbose
  else
    cargo test --release --tests --verbose
  fi
```

PR에서는 기존과 같은 `release-test` 통합 테스트를 유지하고, trusted event에서는 메인테이너 결정사항에 따라
`--release --tests` 검증을 수행한다.

## Stage 2 — 로그 가시성 보강

각 조건부 step에는 실행 profile이 로그에 보이도록 짧은 `echo`를 추가한다.

예:

```bash
echo "profile=release-test event=${GITHUB_EVENT_NAME}"
```

또는

```bash
echo "profile=release event=${GITHUB_EVENT_NAME}"
```

이 로그는 after 분석에서 다음을 확인하는 기준으로 사용한다.

- PR run에서 `Finished release-test ...` 중심으로 바뀌었는지
- trusted event에서 `Finished release ...`가 유지되는지
- `Compiling rhwp`, `Fresh`, `Dirty rhwp ... has changed`가 어느 step에 남는지

## Stage 3 — 로컬 정적 검증

코드 수정 후 다음을 확인한다.

- `git diff --check`
- `actionlint .github/workflows/ci.yml` 사용 가능 시 실행
- actionlint가 없으면 YAML 구조와 shell syntax를 수동 검토하고 한계를 기록
- 변경 파일 확인:
  - `.github/workflows/ci.yml`
  - `mydocs/orders/20260701.md`
  - `mydocs/plans/task_m100_1666.md`
  - `mydocs/plans/task_m100_1666_impl.md`
  - 후속 stage 보고서
- 변경하지 않아야 할 파일 확인:
  - `Cargo.toml`
  - `tests/**`
  - `tests/golden_svg/**`

## Stage 4 — PR CI 관측

코드 PR의 PR run에서 다음을 기록한다.

| 항목 | 기록 내용 |
|------|-----------|
| PR checks 완료 시간 | run created/updated 기준. 샘플 부족 시 P50/P90 보류 |
| `CI / Build & Test` job 시간 | job started/completed 기준 |
| 주요 step 시간 | Build / Native Skia tests / Run lib tests / Run integration tests |
| cargo profile 로그 | `profile=release-test`, `Finished release-test ...`, `Finished release ...` 여부 |
| 중복 compile 로그 | `Compiling rhwp`, `Fresh`, `Dirty rhwp ... has changed` |
| cache 상태 | restore hit/miss, cache size, PR save skipped |
| 실패 가시성 | 실패 step과 stderr 위치 |
| 회귀 가드 | `tests/*.rs` 162개, issue 계열 131개 실행 여부 |
| required check | `Build & Test` 이름 유지 여부 |

raw 측정값은 우선 PR body 또는 #1666 이슈 코멘트에 남긴다. 확정 measurement 원천 문서는 CI 관측 후 별도 문서
PR에서 갱신한다.

## Stage 5 — merge 후 trusted event 관측

코드 PR merge 후 `devel` push run에서 다음을 확인한다.

- `Build`, `Native Skia tests`, `Run lib tests`, `Run integration tests`가 trusted event에서 `release` profile로 실행되는지
- `Run integration tests`가 `--release --tests`로 실행되어 PR에서 이동한 release-grade 검증을 보완하는지
- cache restore/save 정책이 #1664 이후 상태를 유지하는지
- read-only / reservation / save failure 경고가 없는지
- trusted event 비용 증가가 어느 정도인지

이 값도 우선 #1666 이슈 코멘트 또는 PR 후속 코멘트에 남긴 뒤, 후속 문서 PR에서 장기 기록으로 반영한다.

## 합격 기준

- PR `Build & Test`에서 일반 build/test step이 `release-test` profile로 실행된다.
- PR `Run integration tests`는 기존처럼 162개 통합 테스트와 issue 계열 회귀 가드를 실행한다.
- trusted event에서는 release-grade 검증이 유지된다.
- `Build & Test` job 이름과 required check 표면이 바뀌지 않는다.
- `Cargo.toml`, `tests/`, `tests/golden_svg/`는 변경하지 않는다.
- cache restore/save 정책은 #1664 이후 상태를 유지한다.
- cargo 로그로 profile 불일치와 link/codegen 비용 변화 해석이 가능하다.

## 롤백 기준

- PR에서 `release-test` profile 실행이 실패하거나 profile 인식 오류가 발생하는 경우
- `native-skia`가 `release-test`에서만 실패하고 원인이 profile 차이로 보이는 경우
- trusted event에서 release-grade 검증이 실행되지 않는 경우
- required check 이름 또는 branch protection 표면이 바뀌는 경우
- 통합 테스트 162개 또는 issue 계열 131개 추적성이 깨지는 경우

롤백은 조건부 profile 분기를 제거하고 기존 command로 되돌리는 방식으로 수행한다.

## 승인 요청 지점

이 구현 계획서 승인 후 `.github/workflows/ci.yml` 변경과 Stage 1 보고서 작성을 진행한다.
