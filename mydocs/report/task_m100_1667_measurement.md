# Task M100 #1667 measurement 기록

## 목적

이 문서는 #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`의
측정 원천 기록이다.

#1667 전체 범위에는 Build & Test cargo cache, CodeQL Rust cache, Render Diff cargo cache, stale PR ref
cleanup, `Swatinem/rust-cache` 검토가 모두 포함된다. 이 문서의 1차 기록 범위는 PR #1857에서 수행한
CodeQL Rust cache restore/save 분리다.

부모 추적 문서 `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`에는 요약과 후속 판단만 반영하고,
run별 raw 값과 해석은 이 문서를 기준으로 보존한다.

## 범위

- 코드 PR: #1857 `Task #1667: CodeQL Rust cache restore/save 분리`
- merge commit: `aebde2d22948cf5ab712d226fb4b23b3f341e21b`
- merge 시각: 2026-07-03 19:14:33 KST
- 변경 파일: `.github/workflows/codeql.yml`
- 변경하지 않은 파일: `.github/workflows/ci.yml`, `.github/workflows/render-diff.yml`, `Cargo.toml`, `tests/**`

## 측정 기준

부모 이슈 #1668의 공통 측정 기준을 따른다. CodeQL은 `CI / Build & Test`와 별도 workflow이므로
`Analyze (rust)` job 기준 값을 별도 표로 분리한다.

- PR checks 완료 시간
- CodeQL `Analyze (rust)` job 시간
- CodeQL 주요 step 시간
  - Restore cargo registry & build cache (rust)
  - Build Rust (for CodeQL)
  - Perform CodeQL Analysis
- 참고용 `CI / Build & Test` job 시간과 주요 step 시간
- cache hit/miss/save 성공 여부
- cache 크기
- 실패 시 원인 가시성
- runner-minutes 변화
- branch protection / required check 변경 여부
- 회귀 가드 1:1 추적성 보존 여부

## before 기준선

### CodeQL Rust cache

#1667 수행 계획서의 이관 관측을 before 기준으로 사용한다.

- workflow: `.github/workflows/codeql.yml`
- 기존 step: `Cache cargo registry & build (rust)`
- action: `actions/cache@v5`
- key: `Linux-codeql-rust-${Cargo.lock hash}`
- path: `~/.cargo/registry`, `~/.cargo/git`, `target`
- PR run에서도 cache save post-step 표면이 남아 있었다.
- #1702 merge 후 `devel` push에서 fallback `Linux-codeql-rust-` cache hit가 관측됐다.
- 당시 restore cache size는 317,394,514 B였다.
- `Build Rust (for CodeQL)`은 58.97s였다.
- cleanup 전에는 cache budget read-only 상태 때문에 post-cache save reservation 실패가 있었다.
- 실패 위치는 Analyze (rust) log line 2262-2263으로 기록됐다.

이 기준선은 #1857과 동일 commit / 동일 cache key 조건의 직접 전후 비교는 아니다. #1857의 성공 기준은
시간 단축 자체가 아니라, CodeQL Rust cache도 #1664의 정책과 맞게 PR restore-only / trusted branch
save-only 표면으로 바뀌었는지 확인하는 것이다.

### Build & Test 기준선

#1857은 `.github/workflows/ci.yml`을 변경하지 않았다. 따라서 Build & Test 값은 #1857의 직접 성과가 아니라,
#1849 이후 현재 CI 기준선 유지 여부를 보는 참고값이다.

## after 관측 1: #1857 PR run

- PR: #1857
- head SHA: `30a3acaaa01aedbe302cc7762e302875621b8d36`
- 결론: 성공
- CodeQL run: <https://github.com/edwardkim/rhwp/actions/runs/28652708143>
- CI run: <https://github.com/edwardkim/rhwp/actions/runs/28652708175>
- Render Diff run: <https://github.com/edwardkim/rhwp/actions/runs/28652708185>
- PR checks 완료 시간: 약 12m28s
- P50/P90: 단일 PR 표본이므로 산출 보류

### CodeQL Rust

| 항목 | 값 |
|------|----|
| `Analyze (rust)` job | 8m18s |
| restore | exact hit |
| restore key | `Linux-codeql-rust-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| cache 크기 | 529,492,545 B, 약 505 MB |
| `Restore cargo registry & build cache (rust)` | 8s |
| `Build Rust (for CodeQL)` | 39s |
| `Perform CodeQL Analysis` | 6m55s |
| `Save cargo registry & build cache (rust)` | skipped |
| cache reservation / read-only / save failure 경고 | 없음 |

판단:

- PR run에서 CodeQL Rust cache save step이 skipped 되어 PR restore-only 정책 결과가 확인됐다.
- `refs/pull/1857/merge` 기준 신규 GitHub Actions cache는 생성되지 않았다.
- CodeQL Rust exact-hit 상태에서도 `Build Rust (for CodeQL)`에서 `Compiling rhwp`는 남았다. 이번 PR의
  목표는 compile 제거가 아니라 PR cache save 표면 제거다.

### Build & Test 참고값

| 항목 | 값 |
|------|----|
| `CI / Build & Test` job | 12m12s |
| restore | exact hit `Linux-cargo-6a1af...` |
| cache 크기 | 1,637,296,893 B, 약 1.56 GB |
| save | skipped |
| Build | 1m33s |
| Native Skia tests | 2m18s |
| Run lib tests | 1m51s |
| Run integration tests | 3m53s |
| Clippy | 25s |
| cache reservation / read-only / save failure 경고 | 없음 |

#1857은 Build & Test workflow를 변경하지 않았으므로 이 표는 회귀 확인용 참고값이다.

## after 관측 2: #1857 merge 후 `devel` push run

- merge commit: `aebde2d22948cf5ab712d226fb4b23b3f341e21b`
- CodeQL run: <https://github.com/edwardkim/rhwp/actions/runs/28653978487>
- CI run: <https://github.com/edwardkim/rhwp/actions/runs/28653978510>
- 결론: 성공
- CodeQL run 완료 시간: 8m31s
- CI run 완료 시간: 14m19s
- P50/P90: 단일 merge 후 표본이므로 산출 보류

### CodeQL Rust

| 항목 | 값 |
|------|----|
| `Analyze (rust)` job | 8m17s |
| restore | exact hit |
| restore key | `Linux-codeql-rust-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| cache 크기 | 529,492,545 B, 약 505 MB |
| `Restore cargo registry & build cache (rust)` | 11s |
| `Build Rust (for CodeQL)` | 37s |
| cargo build 내부 시간 | `dev` profile 25.76s |
| `Perform CodeQL Analysis` | 6m50s |
| `Save cargo registry & build cache (rust)` | skipped |
| cache reservation / read-only / save failure 경고 | 없음 |

로그 근거:

- `Cache Size: ~505 MB (529492545 B)`
- `Cache restored from key: Linux-codeql-rust-6a1af...d4227c2`
- `Compiling rhwp v0.7.17`
- `Finished dev profile ... in 25.76s`

판단:

- trusted branch push에서도 exact hit이면 save skipped 되는 조건이 확인됐다.
- 이번 run은 exact hit였기 때문에 trusted branch save success 경로는 새로 실행되지 않았다.
- save success 경로는 fallback 또는 miss가 발생한 trusted branch run에서만 관측할 수 있다.
- cache reservation / read-only / save failure 경고가 사라진 상태는 유지됐다.

### Build & Test 참고값

| 항목 | 값 |
|------|----|
| `CI / Build & Test` job | 14m08s |
| restore | exact hit `Linux-cargo-6a1af...` |
| cache 크기 | 1,637,296,893 B, 약 1.56 GB |
| save | skipped |
| Build | 3m38s |
| Check WASM target | 16s |
| Install native Skia runtime packages | 10s |
| Native Skia tests | 2m15s |
| Run lib tests | 1m52s |
| Run integration tests | 3m57s |
| Clippy | 26s |
| cache reservation / read-only / save failure 경고 | 없음 |

로그 해석:

- Build step은 `push` event의 release smoke 정책 때문에 `release` profile로 실행됐다.
- Native Skia / lib / integration tests는 #1849 이후 정책대로 `release-test` profile 중심으로 실행됐다.
- 다만 #1666 merge 후 50분대였던 full `release --tests` integration 비용은 재발하지 않았다.

`rhwp` 재컴파일 분류:

| step | 관측 | 해석 |
|------|------|------|
| Build | `Dirty rhwp` + `Compiling rhwp`, `release` profile 3m38s | `devel` push release smoke라 별도 profile 산출물 생성은 현재 정책상 정상 |
| Check WASM target | `Checking rhwp`, `dev` profile 15.62s | compile/link가 아니라 check 계열 |
| Native Skia tests | `Compiling rhwp`, `release-test` profile 2m13s | `native-skia skia` feature 조합이라 별도 산출물 생성은 현재 구조상 예상 가능 |
| Run lib tests | `Dirty rhwp` + `Compiling rhwp`, `release-test` profile 1m40s | lib test harness 산출물과 cache fingerprint 영향이 섞인 후속 분석 대상 |
| Run integration tests | `Dirty rhwp` + `Compiling rhwp`, `release-test` profile 2m43s | integration test target 산출물과 cache fingerprint 영향이 섞인 후속 분석 대상 |
| Clippy | `Checking rhwp`, `dev` profile 25.64s | check 계열. 별도 link compile은 아님 |

## cache 상태

2026-07-03 19:29 KST 전후 GitHub Actions cache API 기준:

| ref | key | 크기 | last accessed |
|-----|-----|------|---------------|
| `refs/heads/devel` | `Linux-codeql-rust-6a1af...d4227c2` | 529,492,545 B | 2026-07-03T10:28:37Z |
| `refs/heads/devel` | `Linux-cargo-6a1af...d4227c2` | 1,637,296,893 B | 2026-07-03T10:29:32Z |
| `refs/pull/1857/merge` | 없음 | 0 B | 신규 cache 없음 |

## branch protection / required check 영향

- `Analyze (rust)` job 이름은 유지됐다.
- CodeQL workflow의 check 표면은 유지됐다.
- `Build & Test` job 이름과 required check 표면은 유지됐다.
- `devel` branch protection summary 기준 required status check context는 `Build & Test` 그대로다.
- branch protection / required check 설정 변경은 없었다.

## runner-minutes 해석

GitHub Actions timing API의 public repository billable 값은 0으로 노출될 수 있으므로, 이 문서에서는 job wall
time을 runner-minutes proxy로 사용한다.

| 구간 | before | after | 해석 |
|------|--------|-------|------|
| CodeQL Rust `Build Rust (for CodeQL)` | 58.97s | PR 39s / merge 후 37s | 직접 동등 조건은 아니지만 악화 없음 |
| CodeQL `Analyze (rust)` job | 기준 분포 없음 | PR 8m18s / merge 후 8m17s | 단일 표본. P50/P90 보류 |
| PR checks 완료 시간 | 기준 분포 없음 | 12m28s | PR 전체 checks 단일 표본 |
| `devel` push 전체 완료 | 기준 분포 없음 | CodeQL 8m31s / CI 14m19s | #1857 변경으로 check 표면 증가 없음 |

## 회귀 가드 추적성

#1857은 `.github/workflows/codeql.yml`의 cache step만 변경했다.

- `tests/*.rs` 변경 없음
- `tests/golden_svg/**` 변경 없음
- 통합 테스트 파일 통합 없음
- 회귀 가드 명명 규칙 변경 없음
- PR run과 merge 후 `devel` push run에서 `Build & Test`가 모두 성공

따라서 회귀 가드 1:1 추적성은 보존됐다.

## 최종 해석

#1667 1차 PR #1857은 CodeQL Rust cache를 #1664 정책과 같은 구조로 정렬했다.

- 구현 방식: `actions/cache@v5` 단일 step 제거, `restore@v5` / `save@v5` 명시 분리
- 정책 결과: PR restore-only, trusted branch exact-hit save skipped, trusted branch miss/fallback 시 save 허용
- PR cache 결과: `refs/pull/1857/merge` 신규 cache 0개
- 실패 가시성: cache reservation / read-only / save failure 경고 없음
- check 표면: CodeQL / Build & Test required check 변경 없음

남은 판단:

- 이번 run은 exact hit였으므로 trusted branch save success 경로는 새로 실행되지 않았다.
- exact-hit 이후에도 CodeQL Rust와 Build & Test 일부 step에서 `Compiling rhwp`는 남는다.
- Build release smoke와 Native Skia feature 조합은 현재 정책상 예상 가능한 별도 산출물이다.
- Run lib tests / Run integration tests의 `Dirty rhwp`는 cache fingerprint, checkout timestamp, test target
  산출물 관점에서 후속 분석한다.
- 남은 compile은 #1667 후속 범위인 Build & Test target cache 실효성, Cargo fingerprint, checkout timestamp,
  feature/test target 조합 분석으로 이어진다.
- Render Diff cargo cache는 여전히 #1667 후속 PR에서 별도로 판단해야 한다.
