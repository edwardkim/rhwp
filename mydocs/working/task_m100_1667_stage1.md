# Task M100 #1667 Stage 1 완료 보고서

## 단계 목표

CodeQL Rust cache를 `actions/cache@v5` 단일 step에서 restore/save 분리 구조로 바꿔, PR run에서는
restore-only로 동작하고 trusted branch push에서만 save되도록 한다.

이번 단계는 #1667 전체 cache 전략 중 1차 코드 변경이다. Render Diff cache 정책, stale PR ref cleanup,
cleanup 자동화, `Swatinem/rust-cache` 도입은 포함하지 않았다.

## 변경 파일

| 파일 | 변경 |
|------|------|
| `.github/workflows/codeql.yml` | CodeQL Rust cargo cache를 restore/save 분리 |

## 구현 내용

기존 CodeQL Rust cache step은 다음처럼 단일 `actions/cache@v5`를 사용했다.

- step: `Cache cargo registry & build (rust)`
- key: `${{ runner.os }}-codeql-rust-${{ hashFiles('**/Cargo.lock') }}`
- path: `~/.cargo/registry`, `~/.cargo/git`, `target`

이번 단계에서 다음 구조로 변경했다.

- restore step
  - name: `Restore cargo registry & build cache (rust)`
  - id: `codeql_rust_cache_restore`
  - action: `actions/cache/restore@v5`
  - Rust matrix에서만 실행
- save step
  - name: `Save cargo registry & build cache (rust)`
  - action: `actions/cache/save@v5`
  - 조건: `push` + `refs/heads/devel` 또는 `refs/heads/main` + exact hit 아님

즉 이번 변경의 정책 결과는 다음과 같다.

- PR run: restore-only
- `devel/main` push run: miss 또는 fallback일 때 save 허용
- exact hit: save skipped

## 유지한 사항

- CodeQL workflow 이름과 check 이름은 변경하지 않았다.
- `Analyze (rust)` matrix 구조는 변경하지 않았다.
- `Build Rust (for CodeQL)` 명령은 `cargo build` 그대로 유지했다.
- cache key namespace `Linux-codeql-rust-*`는 유지했다.
- cache path `~/.cargo/registry`, `~/.cargo/git`, `target`은 유지했다.

## 제외한 사항

- `.github/workflows/ci.yml`의 Build & Test cache 변경 없음
- `.github/workflows/render-diff.yml`의 Render Diff cache 변경 없음
- `wasm-build` cache 변경 없음
- stale PR ref cache 삭제 없음
- cleanup 자동화 없음
- `Swatinem/rust-cache` 도입 없음
- branch protection / required check 변경 없음

## 로컬 검증

| 검증 | 결과 |
|------|------|
| `git diff --check` | 통과 |
| `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/codeql.yml")'` | 통과 |
| `actionlint .github/workflows/codeql.yml` | 통과 |

Ruby YAML parse 실행 중 로컬 gem `ffi-1.13.1` 확장 경고가 출력됐지만, workflow parse 자체는 `yaml ok`로 완료됐다.

## PR CI 관측 항목

PR run에서 다음을 확인한다.

- `Analyze (rust)` job 성공 여부
- `Restore cargo registry & build cache (rust)` hit/miss
- `Save cargo registry & build cache (rust)` skipped 여부
- cache reservation/read-only/save failure 경고가 없는지
- `Build Rust (for CodeQL)` 시간
- `Perform CodeQL Analysis` 시간
- required check 이름 변경 없음

merge 후 `devel` push run에서 다음을 확인한다.

- restore exact hit / fallback hit / miss
- exact hit이면 save skipped
- miss 또는 fallback이면 save 성공 여부
- saved cache 크기
- cache reservation/read-only/save failure 경고 여부

## 후속 관측

PR #1857 merge 후 PR run과 `devel` push run 관측값은
`mydocs/report/task_m100_1667_measurement.md`에 원천 측정값으로 분리 기록한다.

이 단계에서 확인할 핵심은 CodeQL Rust cache의 PR restore-only / trusted branch save-only 정책 정렬이다.
exact cache hit 이후에도 남는 `Dirty rhwp` / `Compiling rhwp`, Render Diff cargo cache, stale PR ref cleanup은
#1667 후속 단계에서 별도로 판단한다.
