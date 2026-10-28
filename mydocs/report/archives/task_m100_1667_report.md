# Task M100 #1667 최종 보고서

## 결론

#1667은 `Swatinem/rust-cache`를 즉시 도입하지 않고, 현행 `actions/cache`를 유지하면서 cache 저장 표면과
중복 실행을 줄이는 방향으로 완료했다.

완료 범위:

- #1857: CodeQL Rust cache를 `restore@v5` / `save@v5`로 분리
- #1865: Render Diff cargo cache를 `restore@v5` / `save@v5`로 분리하고 npm cache 제거
- 수동 cleanup: closed/merged `refs/pull/*/merge` cache 삭제
- #1872: Build & Test 기본 feature test 중복 실행 제거

도입하지 않은 것:

- `Swatinem/rust-cache`
- cleanup 자동화 workflow
- branch protection / required check 변경
- 회귀 가드 구조 변경

## 주요 결과

### CodeQL Rust cache

| 항목 | 결과 |
|------|------|
| PR run save | skipped |
| merge 후 `devel` push | exact hit, save skipped |
| cache 크기 | 529,492,545 B, 약 505 MB |
| reservation/read-only/save failure | 없음 |
| 신규 PR ref cache | 0개 |

### Render Diff cache

| 항목 | before | after |
|------|--------|-------|
| `Canvas visual diff` | P50 3m47s / P90 3m57s | 3개 표본 3m34s~4m06s |
| cargo cache restore | 20/20 miss | miss 허용 |
| cargo cache save | 20/20 실패 | PR에서 skipped |
| npm cache | restore/save 실패 표면 존재 | 제거 |
| 신규 PR ref cache | 누적 발생 | #1865 PR 기준 0개 |

### stale PR ref cleanup

| 항목 | 값 |
|------|----|
| 삭제 | 19개 cache / 약 5.64 GB |
| cleanup 전 | 30개 / 약 11.13 GB |
| cleanup 후 | 11개 / 약 5.49 GB |
| 잔여 `refs/pull/*` cache | 0개 |

### Build & Test

| 항목 | before #1873 devel push | after #1872 merge devel push | 변화 |
|------|--------------------------|-------------------------------|------|
| CI run 완료 시간 | 13m53s | 12m57s | -56s |
| `CI / Build & Test` job | 13m43s | 12m45s | -58s |
| lib + integration 계열 | `Run lib tests` 1m47s + `Run integration tests` 4m08s | `Run default-feature tests` 5m03s | -52s |
| Native Skia tests | 2m15s | 2m15s | 동일 |
| cache restore/save | exact hit / skipped | exact hit / skipped | 동일 |

`Run lib tests` 제거는 coverage 축소가 아니라 중복 실행 제거다. 현재 Cargo target 구성에서
`cargo test --tests`가 기본 feature lib test harness와 integration test executable을 함께 실행하며,
`cargo test --lib --no-run`에는 `--tests`에 없는 고유 실행 target이 없었다.

## 공통 준수사항 확인

| 항목 | 확인 |
|------|------|
| PR checks 완료 시간 | PR별 단일 표본과 Render Diff before P50/P90 기록. #1872 after devel push는 단일 표본이라 P50/P90 보류 |
| `CI / Build & Test` job 시간 | #1857, #1865 참고값, #1872 before/after 기록 |
| 주요 step 시간 | Build, default-feature tests, Native Skia, Clippy, Render Diff 주요 step 기록 |
| cache hit/miss/save | CodeQL, Render Diff, Build & Test 각각 기록 |
| cache 크기 | CodeQL 약 505 MB, Build & Test 약 1.56 GB, cleanup 전후 총량 기록 |
| 실패 시 원인 가시성 | cache save failure 경고 제거 확인. 실패 run 없음 |
| runner-minutes 변화 | #1872 Build & Test wall time proxy 13.72 min -> 12.75 min |
| branch protection / required check 변경 | 없음 |
| 회귀 가드 1:1 추적성 | `tests/**`, `tests/golden_svg/**`, 회귀 가드 구조 변경 없음. #1872 after에서 issue executable 151개 유지 |

## 남은 판단

현재 상태에서는 `Swatinem/rust-cache` 도입보다 현행 `actions/cache` 유지가 더 적합하다.

이유:

- PR restore-only와 trusted branch save-only 정책을 명시적으로 표현할 수 있다.
- CodeQL / Render Diff / Build & Test 각각에서 PR ref cache 누적 표면을 줄였다.
- cleanup 후 cache quota가 안정화됐다.
- third-party action 도입 리스크 없이 원인 추적성이 유지된다.

남은 `Compiling rhwp`는 cache miss가 아니라 profile, feature, test harness 산출물 차이로 보는 것이 맞다.
특히 Native Skia tests는 feature set이 달라 Build & Test 기본 feature 산출물을 그대로 재사용하기 어렵다.

따라서 다음 최적화 축은 #1665의 job 병렬화 재평가다. cleanup 자동화는 quota 문제가 다시 반복될 때
allowlist 기반 `pull_request.closed` workflow로 별도 이슈/PR에서 검토한다.
