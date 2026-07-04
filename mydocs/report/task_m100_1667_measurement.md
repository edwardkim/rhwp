# Task M100 #1667 measurement 기록

## 목적

이 문서는 #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`의
측정 원천 기록이다.

#1667 전체 범위에는 Build & Test cargo cache, CodeQL Rust cache, Render Diff cargo cache, stale PR ref
cleanup, `Swatinem/rust-cache` 검토가 모두 포함된다. 이 문서는 #1667에서 실제 수행한 CodeQL Rust cache,
Render Diff cache, Build & Test target cache 실효성 분석, 수동 cleanup, 최종 after 관측을 한 곳에 보존한다.

부모 추적 문서 `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`에는 요약과 후속 판단만 반영하고,
run별 raw 값과 해석은 이 문서를 기준으로 보존한다.

## 범위

- 코드 PR #1857: `Task #1667: CodeQL Rust cache restore/save 분리`
- 코드 PR #1865: `Task #1667: Render Diff cache restore/save 분리`
- 코드 PR #1872: `Task #1667: Build & Test 기본 test step 정리`
- 운영 조치: closed/merged `refs/pull/*/merge` stale cache 수동 cleanup
- 변경 파일: `.github/workflows/codeql.yml`, `.github/workflows/render-diff.yml`, `.github/workflows/ci.yml`
- 변경하지 않은 것: `tests/**`, `tests/golden_svg/**`, branch protection / required check 설정, `Cargo.toml` profile,
  `Swatinem/rust-cache` 도입

## 측정 기준

부모 이슈 #1668의 공통 측정 기준을 따른다. CodeQL과 Render Diff는 `CI / Build & Test`와 별도 workflow이므로
각 workflow 기준 값을 별도 표로 분리한다. #1872 이후에는 기존 `Run lib tests`와 `Run integration tests`를
합친 step을 `Run default-feature tests`로 기록한다.

- PR checks 완료 시간
- CodeQL `Analyze (rust)` job 시간
- Render Diff `Canvas visual diff` job 시간
- CodeQL 주요 step 시간
  - Restore cargo registry & build cache (rust)
  - Build Rust (for CodeQL)
  - Perform CodeQL Analysis
- Render Diff 주요 step 시간
  - Restore cargo registry & build cache
  - Build WASM package
  - Build native CLI for PDF report
  - Save cargo registry & build cache
- `CI / Build & Test` job 시간과 주요 step 시간
  - Build
  - Run lib tests / Run integration tests 또는 Run default-feature tests
  - Native Skia tests
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

## #1857 중간 해석

#1667 1차 PR #1857은 CodeQL Rust cache를 #1664 정책과 같은 구조로 정렬했다.

- 구현 방식: `actions/cache@v5` 단일 step 제거, `restore@v5` / `save@v5` 명시 분리
- 정책 결과: PR restore-only, trusted branch exact-hit save skipped, trusted branch miss/fallback 시 save 허용
- PR cache 결과: `refs/pull/1857/merge` 신규 cache 0개
- 실패 가시성: cache reservation / read-only / save failure 경고 없음
- check 표면: CodeQL / Build & Test required check 변경 없음

당시 남은 판단:

- 이번 run은 exact hit였으므로 trusted branch save success 경로는 새로 실행되지 않았다.
- exact-hit 이후에도 CodeQL Rust와 Build & Test 일부 step에서 `Compiling rhwp`는 남는다.
- Build release smoke와 Native Skia feature 조합은 현재 정책상 예상 가능한 별도 산출물이다.
- Run lib tests / Run integration tests의 `Dirty rhwp`는 cache fingerprint, checkout timestamp, test target
  산출물 관점에서 후속 분석한다.
- 남은 compile은 #1667 후속 범위인 Build & Test target cache 실효성, Cargo fingerprint, checkout timestamp,
  feature/test target 조합 분석으로 이어진다.
- Render Diff cargo cache는 여전히 #1667 후속 PR에서 별도로 판단해야 한다.

## after 관측 3: #1865 Render Diff cache restore/save 분리

### before 기준선

PR #1865 전 Render Diff before 표본은 full Render Diff 20개와 fast-pass 9개를 분리해 봤다.

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| `Render Diff` workflow 완료 시간 | 20 | 4m00s | 4m14s |
| `Canvas visual diff` job | 20 | 3m47s | 3m57s |
| `Build WASM package` | 20 | 1m15s | 1m18s |
| `Build native CLI for PDF report` | 20 | 1m04s | 1m09s |

before cache 상태:

- cargo/npm cache restore: 20/20 miss
- cargo/npm cache save: 20/20 reservation/read-only 계열 실패
- cache inventory: 전체 30개 / 11,131,139,002 B
- `Linux-render-diff-cargo-*`: 9개 / 4,685,680,935 B
- `node-cache-*`: 9개 / 427,401,927 B
- Render Diff cargo/npm cache 18개는 모두 merged PR ref에 묶여 최신 PR run에 재사용되지 않았다.

판단:

- 현행 Render Diff cache는 최신 PR run에서 복원 이득을 주지 못했다.
- 반면 post-save 실패 로그와 cache quota 부담은 계속 만들고 있었다.
- 따라서 PR save 차단과 npm cache 제거를 우선 적용했다.

### #1865 PR after 표본

| run | head | workflow | `Canvas visual diff` | 결과 |
|-----|------|----------|----------------------|------|
| <https://github.com/edwardkim/rhwp/actions/runs/28657357877> | `8f9bfc5d` | 3m52s | 3m40s | success |
| <https://github.com/edwardkim/rhwp/actions/runs/28658161697> | `19e937e0` | 3m45s | 3m34s | success |
| <https://github.com/edwardkim/rhwp/actions/runs/28658542787> | `ba894ecf` | 4m17s | 4m06s | success |

after 3개 표본의 보조 P50/P90:

| 항목 | n | P50 | P90 | 비고 |
|------|---|-----|-----|------|
| `Render Diff` workflow 완료 시간 | 3 | 3m52s | 4m17s | n=3이라 보조값 |
| `Canvas visual diff` job | 3 | 3m40s | 4m06s | n=3이라 보조값 |

최종 PR run `28658542787` 주요 step:

| 항목 | 값 |
|------|----|
| `Restore cargo registry & build cache` | 2s, miss |
| `Build WASM package` | 1m19s |
| `Build native CLI for PDF report` | 1m10s |
| `Save cargo registry & build cache` | skipped |
| `Setup Node.js` | 3s |
| `Install Studio dependencies` | 8s |
| `Run canvas visual diff and PDF report` | 28s |
| cargo save failure / reservation / read-only 경고 | 없음 |
| npm cache restore/save 로그 | 없음 |
| `refs/pull/1865/merge` 신규 cache | 0개 |

판단:

- PR run cargo save 차단, npm cache save 표면 제거, cache reservation/read-only/save failure 경고 제거가 확인됐다.
- 신규 `refs/pull/1865/merge` Render Diff cargo/npm cache가 생성되지 않았다.
- Render Diff time은 before P50/P90 근처에 머물렀다. 최종 run 하나가 before P90보다 9초 길지만, 3개 표본
  범위에서는 큰 회귀 신호로 보지 않는다.
- `render-diff.yml`은 `pull_request` / `workflow_dispatch`만 있고 push trigger가 없으므로, merge commit 자체로
  Render Diff push run이 생기지 않는 것은 정상이다.

### #1865 merge 후 `devel` push 참고값

- merge commit: `7391325b99b51b54df3489f2a60a618d98451f11`
- CI: <https://github.com/edwardkim/rhwp/actions/runs/28659404284>, success
- CodeQL: <https://github.com/edwardkim/rhwp/actions/runs/28659404289>, success

| 항목 | 값 |
|------|----|
| `CI / Build & Test` | 14m24s |
| Build | 3m33s |
| Native Skia tests | 2m18s |
| Run lib tests | 1m47s |
| Run integration tests | 4m02s |
| Clippy | 26s |
| CodeQL `Analyze (rust)` | 8m22s |

## after 관측 4: stale PR ref cache 수동 cleanup

메인테이너 승인 후 closed/merged `refs/pull/*/merge` cache만 삭제했다.

| 항목 | 값 |
|------|----|
| 삭제 대상 | closed/merged `refs/pull/*/merge` cache |
| 유지 대상 | `refs/heads/devel`, `refs/heads/main`, open PR cache |
| 삭제 결과 | 19개 cache / 약 5.64 GB |
| cleanup 전 총량 | 30개 / 약 11.13 GB |
| cleanup 후 총량 | 11개 / 약 5.49 GB |
| 잔여 `refs/pull/*` cache | 0개 |
| 10GB budget | 아래로 회복 |

판단:

- #1865 이후 신규 Render Diff PR ref cache는 생성되지 않았지만, 이미 쌓인 closed/merged PR ref cache는 자동으로
  사라지지 않는다.
- 수동 cleanup은 quota 안정화에 직접 효과가 있었다.
- 자동 cleanup workflow는 이번 #1667 코드 변경에는 포함하지 않았다. open PR cache를 잘못 삭제하지 않도록
  필요 시 별도 이슈/PR에서 `pull_request.closed` 기반 allowlist 삭제 정책으로 다루는 것이 안전하다.

## after 관측 5: #1872 Build & Test 기본 test step 정리

### 범위

- PR: #1872
- 제목: `Task #1667: Build & Test 기본 test step 정리`
- merge commit: `00f42a66329452ee47f4cbe8a6439ea30a587821`
- merge 시각: 2026-07-04 02:50:55 KST
- merge 후 `devel` run: <https://github.com/edwardkim/rhwp/actions/runs/28676046470>
- `Build & Test` job: <https://github.com/edwardkim/rhwp/actions/runs/28676046470/job/85049578280>
- head SHA: `00f42a66329452ee47f4cbe8a6439ea30a587821`
- 결론: 성공

#1872는 cache action 교체가 아니다. `actions/cache/restore@v5` / `actions/cache/save@v5`, cache key/path,
branch protection / required check는 그대로 유지하고, Build & Test 안의 기본 feature test step 중복을 줄였다.

### PR head 참고값

PR #1872 최종 head의 `gh pr checks` 기준:

| check | 결과 | 시간 |
|-------|------|------|
| `Build & Test` | pass | 10m30s |
| `Analyze (rust)` | pass | 8m25s |
| `Canvas visual diff` | pass | 3m43s |
| `CI preflight` | pass | 4s |
| `WASM Build` | skipped | 0s |

PR head에는 문서 follow-up commit이 섞여 있으므로, 최종 after 평가는 merge 후 `devel` push run을 기준으로 둔다.

### before / after 비교

before는 #1872 merge 직전의 성공한 `devel` push run #1873을 사용한다.

- before run: <https://github.com/edwardkim/rhwp/actions/runs/28672832712>
- before head SHA: `e5ff8ab9311ff0a22c127f5a5081c4f1cacc421d`
- after run: <https://github.com/edwardkim/rhwp/actions/runs/28676046470>
- after head SHA: `00f42a66329452ee47f4cbe8a6439ea30a587821`

| 항목 | before #1873 devel push | after #1872 merge devel push | 변화 |
|------|--------------------------|-------------------------------|------|
| CI run 완료 시간 | 13m53s | 12m57s | -56s |
| `CI / Build & Test` job | 13m43s | 12m45s | -58s |
| Build | 3m31s | 3m29s | -2s |
| Check WASM target | 15s | 15s | 동일 |
| Install native Skia runtime packages | 10s | 9s | -1s |
| Native Skia tests | 2m15s | 2m15s | 동일 |
| Run lib tests | 1m47s | 제거 | 중복 step 제거 |
| Run integration tests | 4m08s | `Run default-feature tests` 5m03s | step 역할 변경 |
| lib + integration 합산 | 5m55s | `Run default-feature tests` 5m03s | -52s |
| Clippy | 22s | 22s | 동일 |
| Save cargo registry & build cache | skipped | skipped | 동일 |

### cache 관측

| 항목 | 값 |
|------|----|
| restore | exact hit |
| restore key | `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| cache 크기 | 1,637,296,893 B, 약 1.56 GB |
| save | skipped |
| cache reservation / read-only / save failure 경고 | 없음 |

exact hit 상태였으므로 trusted branch save success 경로는 새로 실행되지 않았다. exact hit이면 save skipped가 정상이다.

### compile / test 관측

| step | 관측 | 해석 |
|------|------|------|
| Build | `release` profile, `Compiling rhwp`, 3m29s | `devel` push release smoke라 정상 비용 |
| Run default-feature tests | `release-test` profile, `Compiling rhwp`, 5m03s | 기본 feature lib test harness + integration test 실행 |
| Native Skia tests | `release-test` profile, `native-skia skia`, `Compiling rhwp`, 2m15s | feature set 차이로 별도 산출물 생성 |
| Clippy | `Checking rhwp`, 22s | check 계열 |

`Run default-feature tests` 로그:

- `running 2081 tests`
- `2075 passed; 0 failed; 6 ignored`
- 최신 `devel` 기준 `tests/*.rs` integration test executable 수: 182
- 최신 `devel` 기준 `tests/issue*.rs` 회귀 가드 executable 수: 151

`Native Skia tests` 로그:

- `running 48 tests`
- `48 passed; 0 failed`
- `2080 filtered out`

판단:

- `Run lib tests` 제거는 coverage 축소가 아니라 현재 Cargo target 구성에서 `cargo test --tests`가 이미 포함하는
  기본 feature lib test harness 중복 실행을 제거한 것이다.
- `cargo test --profile release-test --tests --no-run --message-format=json` 기준 `--tests`에 `rhwp` lib test
  executable이 포함된다.
- `cargo test --profile release-test --lib --no-run --message-format=json`에는 `--tests`에 없는 고유 실행 target이 없었다.
- Native Skia는 default feature와 feature set이 달라 별도 step 유지가 맞다.
- `Compiling rhwp`는 0회가 되지 않았다. Build release smoke, default-feature test harness, Native Skia feature
  조합은 서로 다른 산출물이므로 현재 상태에서 정상 비용으로 남는다.

### runner-minutes / check 표면

GitHub Actions public repository timing API의 billable 값은 0으로 노출될 수 있으므로, job wall time을
runner-minutes proxy로 사용한다.

| 항목 | before | after | 변화 |
|------|--------|-------|------|
| `CI / Build & Test` wall time proxy | 13.72 min | 12.75 min | -0.97 min |
| 전체 CI run wall time proxy | 13.88 min | 12.95 min | -0.93 min |

branch protection / required check 영향:

- `Build & Test` job 이름 유지
- `CI / Build & Test` check 표면 유지
- `WASM Build` skip 정책 유지
- branch protection / required check 변경 없음

### P50/P90 상태

- #1872 after `devel` push 표본은 1개이므로 P50/P90은 산출하지 않는다.
- 직접 비교 가능한 merge 직전/직후 단일 값 기준으로는 `Build & Test`가 13m43s에서 12m45s로 약 58초 줄었다.
- 후속 PR/devel push 표본이 더 쌓이면 #1665 병렬화 판단의 입력으로 재집계한다.

## #1667 최종 해석

#1667에서는 `Swatinem/rust-cache`를 바로 도입하지 않고, 현행 `actions/cache`를 유지하면서 cache 저장 표면과
중복 실행을 좁히는 방향으로 정리했다.

완료된 항목:

- CodeQL Rust cache: PR restore-only / trusted branch save-only 구조로 정렬
- Render Diff cargo cache: PR save 차단, npm cache 제거, PR ref 신규 cache 생성 차단
- stale PR ref cache: closed/merged PR ref 수동 cleanup으로 10GB budget 아래 회복
- Build & Test target cache 실효성: exact hit 이후에도 남는 compile을 profile/feature/test harness 차이로 분류
- Build & Test 기본 feature tests: 별도 `Run lib tests` 중복 step 제거, `Run default-feature tests`로 통합

최종 판단:

- 현재 관측만으로는 `Swatinem/rust-cache` 도입 필요성이 높지 않다. third-party action 도입 리스크를 감수하기보다
  현행 `actions/cache` restore/save 분리와 cleanup 운영이 더 단순하고 추적 가능하다.
- `target` cache exact hit가 local crate compile을 완전히 제거하지는 못하지만, 남은 compile은 cache 실패라기보다
  release smoke, test harness, native-skia feature 산출물 차이다.
- #1872 이후 Build & Test는 약 1분 줄었고, 회귀 가드 실행 구조는 유지됐다.
- 남은 큰 최적화 후보는 #1665의 job 병렬화다. 특히 Native Skia tests는 feature set이 달라 기본 feature test
  산출물을 그대로 재사용하기 어렵기 때문에 병렬 분리의 후보로 남긴다.
- cleanup 자동화는 이번 #1667에서 바로 구현하지 않았다. 수동 cleanup 원칙이 확인됐으므로, quota 문제가 반복될 때
  allowlist 기반 `pull_request.closed` cleanup workflow를 별도 이슈/PR에서 검토한다.
