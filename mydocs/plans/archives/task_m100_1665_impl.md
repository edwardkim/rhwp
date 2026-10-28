# Task M100 #1665 구현계획서

## 개요

- 이슈: #1665 `[CI] Build & Test job 병렬 분리 설계`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 수행계획서: `mydocs/plans/task_m100_1665.md`
- 구현 대상: `.github/workflows/ci.yml`
- 구현 원칙: Native Skia tests만 1차로 분리하고, 기존 required check 후보인 `Build & Test` 이름은 aggregate
  gate로 보존한다.

## 구현 목표

현재 `CI / Build & Test`는 단일 job에서 다음을 순차 실행한다.

1. checkout / disk cleanup / Rust toolchain / cargo restore
2. Format check
3. Build
4. Run default-feature tests
5. Check WASM target
6. Install native Skia runtime packages
7. Native Skia tests
8. Clippy
9. trusted branch cache save

1차 구현은 Native Skia tests를 별도 worker job으로 분리해, default-feature 회귀 경로와 native-skia feature
경로를 병렬 실행할 수 있게 한다.

## 변경 요약

### 기존 job

| job id | name | 변경 |
|--------|------|------|
| `preflight` | `CI preflight` | 유지 |
| `build-and-test` | `Build & Test` | aggregate gate로 역할 변경 |
| `wasm-build` | `WASM Build` | 유지 |

### 신규 worker job

| job id | name | 역할 |
|--------|------|------|
| `build-default-feature-tests` | `Build default-feature tests` | fmt, Build, default-feature tests, WASM check, Clippy, 단일 cache save writer |
| `native-skia-tests` | `Native Skia tests` | native package install, `cargo test --features native-skia skia --lib` |

## 세부 구현

### 1. `build-default-feature-tests` job 추가

기존 `build-and-test` job에서 Native Skia 관련 step만 제외한 기본 feature 검증 job을 만든다.

구성:

- `runs-on: ubuntu-latest`
- `needs: preflight`
- `if: ${{ always() && needs.preflight.outputs.fast_pass != 'true' }}`
- `permissions: contents: read`
- steps:
  - `actions/checkout@v5`
  - `Free disk space (remove unused pre-installed toolchains)`
  - `dtolnay/rust-toolchain@stable`
    - toolchain: `1.93.1`
    - components: `clippy, rustfmt`
    - targets: `wasm32-unknown-unknown`
  - `Restore cargo registry & build cache`
    - id: `cargo_cache_restore`
    - 기존 key/path 유지
  - `Format check`
  - `Build`
    - pull_request: `cargo build --profile release-test --verbose`
    - push devel/main/tag/manual: 기존 분기 유지
  - `Run default-feature tests`
    - pull_request / `refs/heads/devel`: `cargo test --profile release-test --tests --verbose`
    - main/tag/manual: `cargo test --release --tests --verbose`
  - `Check WASM target`
  - `Clippy`
  - `Save cargo registry & build cache`
    - 기존 조건 유지
    - 이 job만 save writer 역할 수행

Native Skia 관련 step은 이 job에서 제거한다.

### 2. `native-skia-tests` job 추가

Native Skia feature set은 기본 feature test 산출물과 다르므로 별도 worker job으로 둔다.

구성:

- `runs-on: ubuntu-latest`
- `needs: preflight`
- `if: ${{ always() && needs.preflight.outputs.fast_pass != 'true' }}`
- `permissions: contents: read`
- steps:
  - `actions/checkout@v5`
  - `Free disk space (remove unused pre-installed toolchains)`
  - `dtolnay/rust-toolchain@stable`
    - toolchain: `1.93.1`
  - `Restore cargo registry & build cache`
    - 기존 key/path 유지
    - id는 이 job 안에서만 쓰므로 `cargo_cache_restore` 재사용 가능
  - `Install native Skia runtime packages`
  - `Native Skia tests`
    - pull_request / `refs/heads/devel`: `cargo test --profile release-test --features native-skia skia --lib --verbose`
    - main/tag/manual: `cargo test --release --features native-skia skia --lib --verbose`

이 job에는 cache save step을 두지 않는다.

이유:

- 같은 cache key를 여러 job이 동시에 save하면 reservation 경합이 생길 수 있다.
- #1664 / #1667에서 정리한 PR restore-only 원칙을 유지한다.
- trusted branch에서도 단일 writer만 save하게 해 원인 추적성을 유지한다.

### 3. `build-and-test` job을 aggregate gate로 변경

기존 `build-and-test` job id와 `name: Build & Test`를 유지하되, 실제 build/test step은 worker job으로 옮긴다.

구성 후보:

```yaml
build-and-test:
  name: Build & Test
  runs-on: ubuntu-latest
  needs:
    - preflight
    - build-default-feature-tests
    - native-skia-tests
  if: ${{ always() }}
  permissions:
    contents: read
  steps:
    - name: Check Build & Test worker results
      run: |
        echo "preflight=${{ needs.preflight.result }}"
        echo "fast_pass=${{ needs.preflight.outputs.fast_pass }}"
        echo "build_default=${{ needs['build-default-feature-tests'].result }}"
        echo "native_skia=${{ needs['native-skia-tests'].result }}"

        if [[ "${{ needs.preflight.result }}" != "success" ]]; then
          echo "::error::CI preflight did not succeed: ${{ needs.preflight.result }}"
          exit 1
        fi

        if [[ "${{ needs.preflight.outputs.fast_pass }}" == "true" ]]; then
          echo "CI preflight fast-pass accepted."
          exit 0
        fi

        failed=0
        if [[ "${{ needs['build-default-feature-tests'].result }}" != "success" ]]; then
          echo "::error::Build default-feature tests result: ${{ needs['build-default-feature-tests'].result }}"
          failed=1
        fi
        if [[ "${{ needs['native-skia-tests'].result }}" != "success" ]]; then
          echo "::error::Native Skia tests result: ${{ needs['native-skia-tests'].result }}"
          failed=1
        fi
        exit "${failed}"
```

동작 기대:

- fast-pass PR: worker jobs는 skipped, aggregate `Build & Test`는 success.
- 일반 PR / push: 두 worker가 모두 success여야 aggregate `Build & Test` success.
- worker failure: aggregate `Build & Test` failure. 로그에 실패 worker 이름이 남는다.

## branch protection / check 표면

목표:

- 기존 required check context로 쓰이던 `Build & Test` 이름을 유지한다.
- 새 worker check는 추가로 보이지만 required check로 요구하지 않는다.
- PR에서 `CI / Build & Test`가 aggregate 결과를 대표한다.

확인할 점:

- GitHub check-run name은 job `name`을 기준으로 노출된다.
- 기존 preflight의 trailing review-only fast-pass 검사는 candidate SHA의 `Build & Test` check-run을 조회한다.
- 따라서 aggregate job 이름이 `Build & Test`이면 preflight의 기존 조회 로직과도 맞는다.

리스크:

- 기존 `Build & Test` check가 직접 build/test job에서 aggregate job으로 의미가 바뀐다.
- PR에서 required check가 새 worker job을 요구하지 않는지 확인해야 한다.
- fast-pass PR에서 기존에는 `Build & Test`가 skipped였을 수 있으나, 변경 후 aggregate가 success가 될 수 있다.
  이는 failure를 숨기는 것이 아니라 preflight가 성공적으로 fast-pass를 승인했다는 명시 결과다.

## cache 정책

| job | restore | save |
|-----|---------|------|
| `Build default-feature tests` | 기존 key/path restore | trusted branch exact miss/fallback일 때만 save |
| `Native Skia tests` | 기존 key/path restore | 없음 |
| `Build & Test` aggregate | 없음 | 없음 |

cache key/path:

- key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- restore key: `${{ runner.os }}-cargo-`
- path:
  - `~/.cargo/registry`
  - `~/.cargo/git`
  - `target`

보존하는 정책:

- PR은 restore-only.
- trusted branch save는 `devel` / `main` push에서만 허용.
- save writer는 하나만 둔다.
- `Swatinem/rust-cache` 도입 없음.
- cache key/path 변경 없음.

## 예상 효과

### wall time

#1872 merge 후 `devel` push 기준:

| 항목 | 기존 |
|------|------|
| `Build & Test` job | 12m45s |
| Build | 3m29s |
| Run default-feature tests | 5m03s |
| Native Skia tests | 2m15s |
| Clippy | 22s |

Native Skia를 별도 job으로 분리하면 기존 직렬 경로에서 native package install 9s + Native Skia tests 2m15s가
기본 feature 경로와 겹칠 수 있다.

예상:

- wall time 개선: 약 1m30s-2m30s
- runner-minutes 증가: Native Skia worker의 checkout/toolchain/cache restore/setup 비용 때문에 약 3-4분 증가 가능
- 실제 판단은 GitHub Actions run time 기준으로 한다.

### 실패 가시성

개선:

- Native Skia 실패가 별도 job으로 분리되어 더 빨리 보일 수 있다.
- aggregate 로그에서 실패 worker가 명시된다.

주의:

- 최종 required check는 aggregate `Build & Test`가 대표한다.
- 상세 원인은 worker job 로그를 따라가야 한다.

## 검증 계획

### 로컬 정적 검증

- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"`
- `actionlint .github/workflows/ci.yml`
- `git diff --check`

`actionlint`가 로컬에 없으면 설치하지 않고 실행 불가를 기록한다.

### PR CI 관측

PR run에서 확인할 항목:

- `CI preflight` 성공
- `Build default-feature tests` 성공
- `Native Skia tests` 성공
- aggregate `Build & Test` 성공
- `WASM Build` 기존 조건 유지
- `Build default-feature tests` cache restore hit/miss와 cache 크기
- `Native Skia tests` cache restore hit/miss와 cache 크기
- PR에서 save step은 `Build default-feature tests`에서도 skipped
- `Native Skia tests`에는 save step 없음
- cache reservation / read-only / save failure 경고 없음
- `Run default-feature tests`: 2081 tests 유지
- `Native Skia tests`: 48 tests 유지
- `tests/*.rs`, `tests/issue*.rs` executable 추적성 유지

### merge 후 `devel` push 관측

merge 후 run에서 확인할 항목:

- aggregate `Build & Test` 성공
- trusted branch에서 save writer가 하나뿐인지 확인
- exact hit이면 save skipped가 정상
- miss/fallback이면 `Build default-feature tests`에서만 save 시도
- `Native Skia tests`는 restore-only
- runner-minutes proxy:
  - aggregate gate wall time
  - `Build default-feature tests` wall time
  - `Native Skia tests` wall time
  - worker 합산 wall time
  - 전체 CI wall time

## measurement 기록 계획

코드 PR에는 implementation/stage 운영 문서까지만 포함한다.

코드 PR merge 후 별도 measurement 문서 PR에서 다음을 반영한다.

- `mydocs/report/task_m100_1665_measurement.md`
- `mydocs/report/task_m100_1665_report.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`

## 구현 단계

### Stage 1: workflow 구조 변경

- 기존 `build-and-test`의 build/test step을 `build-default-feature-tests`로 이동
- Native Skia 관련 step을 `native-skia-tests`로 이동
- 기존 `build-and-test`는 aggregate gate로 변경
- `wasm-build`는 변경하지 않음

### Stage 2: 문서와 PR 설명 보강

- `mydocs/working/task_m100_1665_stage1.md` 작성
- PR body에 중요 변경점 명시
  - `Build & Test`가 aggregate gate가 됨
  - 실제 build/test worker가 2개로 분리됨
  - Native Skia는 restore-only
  - save writer는 `Build default-feature tests` 하나
  - required check 변경 여부는 PR CI에서 확인 필요

### Stage 3: 검증과 관측

- 로컬 YAML/actionlint/diff 검증
- PR CI 관측
- Ready for review 전 branch protection / required check 영향 확인
- merge 후 `devel` push 관측

## 성공 기준

- PR과 `devel` push에서 aggregate `Build & Test`가 성공한다.
- worker job 실패가 aggregate failure로 전달된다.
- fast-pass PR에서 aggregate `Build & Test`가 잘못 fail하지 않는다.
- cache save 경합이 없다.
- PR save skipped 정책이 유지된다.
- `Run default-feature tests`와 Native Skia test count가 유지된다.
- wall time이 기존 #1872 after 기준보다 최소 1분 줄어든다.
- worker wall time 합산 proxy가 기존 단일 job 대비 +40% 이상 늘면 성공으로 보지 않고 재검토한다.

## 롤백 기준

- branch protection required check가 깨진다.
- aggregate가 worker 실패를 success로 잘못 전달한다.
- worker job skipped/cancelled 상태를 잘못 해석한다.
- cache save failure / reservation 경고가 재발한다.
- wall time 개선이 거의 없고 runner-minutes만 증가한다.
- CI 화면 복잡도가 과도하거나 maintainer가 required check 의미 변경을 수용하기 어렵다고 판단한다.

## 현재 구현 판단

1차 구현은 진행 가능하다.

다만 이 변경은 workflow required check 표면과 GitHub Actions result propagation에 영향을 주므로, 구현 후 바로 merge하지
않고 PR CI에서 aggregate 동작을 먼저 검증해야 한다. 특히 `Build & Test`가 aggregate gate로 바뀐다는 점을 PR 본문
상단에 중요 변경으로 명시한다.
