# Task M100 #1667 measurement 기록

## 목적

이 문서는 #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`의
측정 원천 기록이다.

#1667 전체 범위에는 Build & Test cargo cache, CodeQL Rust cache, Render Diff cargo cache, stale PR ref
cleanup, `Swatinem/rust-cache` 검토가 모두 포함된다. 이 문서에는 PR #1857에서 수행한 CodeQL Rust
cache restore/save 분리 관측과, 후속 Render Diff cargo cache 정책 변경 전 before 관측을 함께 기록한다.

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

## Render Diff before 관측: #1861 merge 후 표본 수집

- 기준 계획서 PR: #1861 `Task #1667: Render Diff cache 후속 계획 수립`
- merge commit: `8ea6f3f5e6e5446a8312d59e58eb81c70f8c80c4`
- merge 시각: 2026-07-03 19:57:41 KST
- 표본 수집 시각: 2026-07-03 20:11 KST 전후
- 변경하지 않은 파일: `.github/workflows/render-diff.yml`, `.github/workflows/ci.yml`, `tests/**`, `tests/golden_svg/**`

이 절은 Render Diff cargo cache 코드 변경 전 before 기준선이다. 여러 PR의 최근 successful
`Render Diff` workflow run을 사용했으므로, 전체 PR checks 완료 시간은 PR별 다른 workflow 상황과 섞인다.
따라서 이 절에서는 `Render Diff` workflow 완료 시간을 해당 check의 PR checks 완료 시간 proxy로 기록하고,
후속 코드 PR에서는 해당 PR의 전체 checks 완료 시간을 별도로 기록한다.

### 표본 분리

| 구분 | 표본 | 판단 |
|------|------|------|
| full Render Diff | 20개 successful `Canvas visual diff` 실행 run | P50/P90 참고 가능 |
| fast-pass | 9개 preflight success + `Canvas visual diff` skipped run | full run과 분리 집계 |
| cancelled / in-progress | 제외 | P50/P90에서 제외 |

fast-pass 표본은 7-11초 안에 끝났고, 모두 `Render Diff preflight` success 후 `Canvas visual diff`가 skipped
됐다. full Render Diff와 섞으면 실제 render/cache 비용이 왜곡되므로 별도 지표로만 둔다.

### full Render Diff P50/P90

| 항목 | n | P50 | P90 | min | max |
|------|---|-----|-----|-----|-----|
| `Render Diff` workflow 완료 시간 | 20 | 4m00s | 4m14s | 3m54s | 8m45s |
| `Canvas visual diff` job | 20 | 3m47s | 3m57s | 3m41s | 4m09s |
| `Cache cargo registry & build` | 20 | 1s | 1s | 0s | 2s |
| `Build WASM package` | 20 | 1m15s | 1m18s | 1m12s | 1m19s |
| `Build native CLI for PDF report` | 20 | 1m04s | 1m09s | 1m01s | 1m14s |
| `Setup Node.js` | 20 | 1s | 1s | 0s | 5s |
| `Install Studio dependencies` | 20 | 6s | 7s | 5s | 7s |
| `Install Chromium` | 20 | 7s | 7s | 6s | 7s |
| `Check render diff script syntax` | 20 | 0s | 1s | 0s | 1s |
| `Run canvas visual diff and PDF report` | 20 | 26s | 27s | 25s | 27s |
| `Upload render diff artifacts` | 20 | 1s | 2s | 0s | 2s |
| `Post Cache cargo registry & build` | 20 | 7s | 8s | 6s | 8s |

`Render Diff` workflow 완료 시간의 max 8m45s와 그 다음 outlier 6m03s는 job 자체 시간보다 workflow
created/updated 구간이 긴 경우다. concurrency / queue 영향이 섞일 수 있으므로 runner-minutes proxy는
`Canvas visual diff` job wall time을 중심으로 본다.

### full Render Diff 표본

| run | branch | workflow | `Canvas visual diff` | WASM build | native CLI build | visual diff | post cache | URL |
|-----|--------|----------|----------------------|------------|------------------|-------------|------------|-----|
| `28655648394` | `task1853-float-stack-overcapture-fix` | 8m45s | 3m53s | 1m17s | 1m08s | 25s | 8s | <https://github.com/edwardkim/rhwp/actions/runs/28655648394> |
| `28654634119` | `task1853-float-stack-overcapture-fix` | 4m03s | 3m52s | 1m19s | 1m09s | 27s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28654634119> |
| `28652708185` | `task-1667-rust-cache-strategy` | 4m06s | 3m53s | 1m12s | 1m07s | 27s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28652708185> |
| `28652234615` | `task1853-float-stack-overcapture-fix` | 4m04s | 3m51s | 1m18s | 1m07s | 26s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28652234615> |
| `28651915212` | `task/m100-1733-residual-overpagination` | 6m03s | 4m09s | 1m18s | 1m12s | 27s | 8s | <https://github.com/edwardkim/rhwp/actions/runs/28651915212> |
| `28651721938` | `task1853-float-stack-overcapture-fix` | 3m56s | 3m43s | 1m13s | 1m04s | 26s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28651721938> |
| `28648923163` | `task-1849-ci-profile-policy` | 4m00s | 3m47s | 1m16s | 1m05s | 25s | 6s | <https://github.com/edwardkim/rhwp/actions/runs/28648923163> |
| `28646705382` | `task/pr1850-review-followup` | 4m11s | 3m57s | 1m16s | 1m06s | 27s | 8s | <https://github.com/edwardkim/rhwp/actions/runs/28646705382> |
| `28645898070` | `task-1849-ci-profile-policy` | 3m59s | 3m46s | 1m15s | 1m05s | 27s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28645898070> |
| `28645764974` | `pr/devel-1841` | 3m54s | 3m41s | 1m14s | 1m03s | 26s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28645764974> |
| `28645144211` | `pr/devel-1841` | 4m06s | 3m47s | 1m15s | 1m04s | 27s | 8s | <https://github.com/edwardkim/rhwp/actions/runs/28645144211> |
| `28642075330` | `pr/devel-1831` | 4m14s | 4m03s | 1m17s | 1m14s | 25s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28642075330> |
| `28641500202` | `pr/devel-1831` | 3m58s | 3m46s | 1m15s | 1m05s | 27s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28641500202> |
| `28640393193` | `pr/devel-1831` | 4m01s | 3m48s | 1m13s | 1m03s | 25s | 8s | <https://github.com/edwardkim/rhwp/actions/runs/28640393193> |
| `28639826638` | `batch-pr1823-1840-review` | 3m54s | 3m43s | 1m15s | 1m03s | 25s | 6s | <https://github.com/edwardkim/rhwp/actions/runs/28639826638> |
| `28636401511` | `pr/devel-1811` | 4m02s | 3m49s | 1m15s | 1m04s | 26s | 6s | <https://github.com/edwardkim/rhwp/actions/runs/28636401511> |
| `28634205697` | `pr/devel-1809-v2` | 3m59s | 3m47s | 1m14s | 1m03s | 25s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28634205697> |
| `28632916505` | `pr/devel-1773` | 3m55s | 3m44s | 1m12s | 1m01s | 25s | 6s | <https://github.com/edwardkim/rhwp/actions/runs/28632916505> |
| `28631050240` | `pr/devel-1829` | 3m56s | 3m46s | 1m16s | 1m04s | 26s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28631050240> |
| `28629296615` | `pr/devel-1827` | 4m00s | 3m48s | 1m14s | 1m04s | 26s | 7s | <https://github.com/edwardkim/rhwp/actions/runs/28629296615> |

### cache hit/miss/save 관측

| 항목 | 관측 |
|------|------|
| Render Diff cargo restore | full run 20/20 miss |
| Render Diff cargo save | full run 20/20 post-step save 시도 후 실패 |
| cargo save 실패 원인 | cache budget read-only / reservation failure |
| npm restore | full run 20/20 miss |
| npm save | full run 20/20 reservation failure |
| cargo key | `Linux-render-diff-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| npm key | `node-cache-Linux-x64-npm-7e28cd65a573ec1b710bb86f9d35a17472974475ef7e02421a6fbf68f2971390` |

대표 로그 위치:

- run `28655648394`, `Canvas visual diff` job
- cargo restore miss: line 456
- npm restore miss: line 723
- npm save reservation failure: line 875
- cargo save budget/read-only warning: line 878
- cargo save failure: line 879

즉 현재 `actions/cache@v5` 단일 step은 PR run에서 cache를 복원하지 못하고, 종료 시점에는 큰 PR ref cache
save를 시도하다가 실패 로그를 남기는 상태다.

### cache inventory

2026-07-03 20:11 KST 전후 GitHub Actions cache API 기준:

| prefix | 개수 | 합산 크기 | ref 판단 |
|--------|------|-----------|----------|
| 전체 | 30 | 11,131,139,002 B | cache budget 10GB 초과 상태 |
| `Linux-render-diff-cargo-*` | 9 | 4,685,680,935 B | 모두 `refs/pull/*` |
| `node-cache-*` | 9 | 427,401,927 B | 모두 `refs/pull/*` |
| `Linux-cargo-*` | 3 | 3,611,679,991 B | Build & Test 중심 |
| `Linux-codeql-rust-*` | 5 | 2,216,797,926 B | CodeQL Rust |
| `codeql-overlay-*` | 4 | 189,578,223 B | CodeQL overlay |

Render Diff cargo cache 9개는 모두 같은 cargo key지만 ref가 다르다. 각 항목은 약 520 MB이고,
해당 PR은 모두 merged 상태다.

| PR ref | PR 상태 | cargo cache | npm cache |
|--------|---------|-------------|-----------|
| `refs/pull/1656/merge` | merged | 520,527,402 B | 47,488,200 B |
| `refs/pull/1741/merge` | merged | 520,655,103 B | 47,491,813 B |
| `refs/pull/1739/merge` | merged | 520,741,478 B | 47,490,287 B |
| `refs/pull/1738/merge` | merged | 520,739,196 B | 47,489,021 B |
| `refs/pull/1736/merge` | merged | 520,636,087 B | 47,490,104 B |
| `refs/pull/1732/merge` | merged | 520,613,504 B | 47,488,340 B |
| `refs/pull/1730/merge` | merged | 520,642,639 B | 47,487,031 B |
| `refs/pull/1731/merge` | merged | 520,610,270 B | 47,488,490 B |
| `refs/pull/1729/merge` | merged | 520,515,256 B | 47,488,641 B |

해석:

- 현재 남아 있는 Render Diff cargo/npm cache는 closed/merged PR ref에 묶여 있다.
- 최신 PR run들은 이 cache를 복원하지 못한다.
- 새 PR run은 cache miss 후 save를 시도하지만, 이미 budget 초과 상태라 저장에 실패한다.
- 따라서 현행 Render Diff cache는 관측 범위에서는 PR wall time을 줄이지 못하고, cache quota와 실패 로그만 만든다.

### branch protection / required check 영향

- 이번 관측 문서 PR은 workflow 변경이 아니므로 branch protection / required check를 변경하지 않는다.
- `devel` branch는 protected 상태다.
- required status checks 세부 API는 404로 노출되지 않았다. 기존 #1667 기록의 `Build & Test` required
  context 판단은 유지하되, Render Diff 후속 코드 PR에서는 check 이름 변경이 없는지 PR checks 화면으로 다시 확인한다.

### 회귀 가드 추적성

이 before 관측과 문서 PR은 `tests/**`, `tests/golden_svg/**`, 통합 테스트 파일, 회귀 가드 명명 규칙을
변경하지 않는다. Render Diff workflow의 preflight/full-run 분리도 그대로 유지한다. 따라서 회귀 가드 1:1
추적성에는 영향이 없다.

### Render Diff before 해석

현재 Render Diff의 full run 시간 자체는 짧다.

- `Canvas visual diff` job P50은 3m47s, P90은 3m57s다.
- 가장 큰 step은 `Build WASM package` P50 1m15s와 `Build native CLI for PDF report` P50 1m04s다.
- 실제 visual diff 실행은 P50 26s다.

하지만 cache 관점의 효율은 낮다.

- 최신 full run 20개 모두 cargo/npm cache miss다.
- 최신 full run 20개 모두 post-save가 reservation/read-only 실패를 남긴다.
- 현재 남아 있는 Render Diff cargo/npm cache는 모두 merged PR ref cache이며, 최신 PR run에는 재사용되지 않았다.
- cache 총량은 11.13GB로 다시 10GB budget을 초과했다.

따라서 후속 구현계획서에서는 최소한 PR run의 Render Diff cargo/npm save 표면을 제거하는 방향을 우선 검토하는
것이 합리적이다. 다만 Render Diff는 PR 중심 workflow라 trusted seed가 실제 PR restore로 이어지는지,
또는 `target` 제외 path 축소가 시간 대비 quota에 더 유리한지는 구현계획서에서 별도 후보로 확정해야 한다.

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
