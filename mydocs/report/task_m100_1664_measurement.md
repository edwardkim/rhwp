# Task M100 #1664 측정 기록

## 목적

이 문서는 #1664 적용 전후 CI 측정값을 누적 기록하는 장기 보관 문서다. 단일 작업 완료 보고서가 아니라,
PR run과 `devel` / `main` push run이 쌓일 때마다 값을 추가하는 측정 원천 로그로 사용한다.

문서 PR #1701은 정책/측정 기록만 포함한다. 실제 workflow 변경은 후속 코드 PR #1702에서 다루며, 아래 PR
run 측정값은 #1702 draft 코드 PR 기준 관측값이다. #1702가 merge되기 전에는 이 workflow 변경이 `devel`에
반영된 사실로 해석하지 않는다.

정책/의사결정 원천 문서:

- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
- `mydocs/tech/ci_cache_policy_1664.md`

## 기록 원칙

- GitHub Actions run URL 또는 run id를 함께 기록한다.
- PR run과 trusted branch push run을 분리해 기록한다.
- P50/P90은 샘플 1개로 의미 있게 해석하지 않는다.
- 샘플이 적을 때는 관측값으로만 기록하고, 분포 요약은 보류한다.
- workflow 변경 외 요인이 섞인 run은 비고에 명시한다.

샘플 수 해석 기준:

| 샘플 수 | 해석 |
|---------|------|
| 1-4 | 단일/소수 관측값. P50/P90 판단 보류 |
| 5-9 | 참고값. 방향성만 관찰 |
| 10-19 | 제한적 P50/P90 참고 가능 |
| 20+ | P50/P90을 추세 지표로 사용 가능 |

## 측정 항목

메인테이너 요청 기준:

- PR checks 완료 시간 (P50, P90)
- `CI / Build & Test` job 시간
- 주요 step 시간
  - build
  - lib test
  - integration test
  - native-skia
- cache hit/miss/save 성공 여부
- cache 크기
- 실패 시 원인 가시성
- runner-minutes 변화
- branch protection / required check 변경 여부
- 회귀 가드 162개가 PR마다 모두 실행되는지 확인

## 로컬 정적 검증

| 날짜 | 브랜치 | 항목 | 결과 | 비고 |
|------|--------|------|------|------|
| 2026-06-30 | `local/task1664` / #1702 | `git diff --check` | 통과 | 후속 코드 PR #1702 기준 whitespace 문제 없음 |
| 2026-06-30 | `local/task1664` / #1702 | `actionlint .github/workflows/ci.yml` | 통과 | 후속 코드 PR #1702 기준 workflow 문법 오류 없음 |
| 2026-06-30 | `local/task1664` / #1702 | 변경 범위 확인 | 통과 | 후속 코드 PR #1702 기준 `Cargo.toml`, `tests/` 변경 없음 |
| 2026-06-30 | `local/task1664` / #1702 | required check 표면 | 통과 | `Build & Test` job 이름 유지 |
| 2026-06-30 | `local/task1664` / #1702 | 회귀 가드 구조 | 통과 | 테스트 파일/자산 구조 변경 없음 |

## PR run 측정 로그

PR run에서는 cache save가 skipped 되어야 한다.

| 날짜 | PR | run | head SHA | PR checks 완료 시간 | Build & Test 시간 | build | native-skia | lib test | integration test | restore hit/miss | save 상태 | 회귀 가드 162개 | 비고 |
|------|----|-----|----------|---------------------|-------------------|-------|-------------|----------|------------------|------------------|-----------|----------------|------|
| 2026-06-30 | #1702 | `28430353568` | `69229e7937dc08fb94bf5d6530f205de77c15fe4` | 약 19m23s | 19m08s | 3m33s | 3m57s | 3m46s | 4m51s | 정확히 적중 | skipped | issue 계열 131/131 실행 확인 | 변경 후 표본 1개. P50/P90 보류 |

## trusted branch push 측정 로그

`devel` / `main` push run에서는 exact cache hit가 아니면 cache save가 실행되어야 한다.

| 날짜 | branch | run | SHA | Build & Test 시간 | build | native-skia | lib test | integration test | restore hit/miss | save 상태 | cache 크기 | read-only 경고 | 비고 |
|------|--------|-----|-----|-------------------|-------|-------------|----------|------------------|------------------|-----------|------------|----------------|------|
| TBD | `devel` | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | success/skipped/failure | TBD | 없음 기대 | merge 후 기록 |

## P50/P90 요약

샘플 수가 충분할 때 갱신한다.

| 구간 | 대상 | 샘플 수 | P50 | P90 | 비고 |
|------|------|---------|-----|-----|------|
| before | PR checks 완료 시간 | TBD | TBD | TBD | 기존 run 수집 필요 |
| after | PR checks 완료 시간 | 1 | 보류 | 보류 | #1702 단일 관측값 약 19m23s |
| before | `CI / Build & Test` job 시간 | TBD | TBD | TBD | 기존 run 수집 필요 |
| after | `CI / Build & Test` job 시간 | 1 | 보류 | 보류 | #1702 단일 관측값 19m08s |

## cache 상태 요약

| 날짜 | 총 cache 크기 | 주요 key | PR ref cache 상태 | read-only 여부 | 비고 |
|------|---------------|----------|-------------------|----------------|------|
| TBD | TBD | TBD | TBD | TBD | GitHub cache 목록 확인 후 기록 |

## 진행 중 관측 메모

### 2026-06-30 — 후속 코드 PR #1702 Build & Test 중간 관측

- PR: #1702 `Task #1664: cargo cache save를 trusted branch로 제한`
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28430353568/job/84243307175?pr=1702>
- 상태: 후속/draft 코드 PR 기준 CI 진행 중 관측. 최종 결과와 step별 시간은 run 완료 후 위 표에 정식 반영한다.

관측 로그:

```text
Native Skia tests:
Dirty rhwp v0.7.17 (/home/runner/work/rhwp/rhwp): the file `src/parser/hwp3/mod.rs` has changed
(1782807499.561337796s, 21h 2m 26s after last build at 1782731753.488450083s)
   Compiling rhwp v0.7.17

Run lib tests:
Dirty rhwp v0.7.17 (/home/runner/work/rhwp/rhwp): the file `src/model/footnote.rs` has changed
(1782807499.556391453s, 20h 58m 35s after last build at 1782731984.950573421s)
   Compiling rhwp v0.7.17
```

임시 해석:

- `native-skia` feature가 켜진 lib test와 일반 lib test는 Cargo feature set이 달라 `rhwp` crate 산출물을
  각각 컴파일할 수 있다. 이 부분은 일부 정상 비용이다.
- 다만 restored `target` cache 이후에도 `Dirty rhwp ... has changed` 판정으로 local crate가 다시 컴파일되는
  현상은 target cache 실효성 문제일 수 있다.
- 이 관측은 #1666의 `--release` / `release-test` profile 전환 검토와 #1667의 Rust cache 전략 검토 근거로
  후속 정리한다.
- 단일 run 중간 관측이므로 #1666/#1667 이슈 코멘트는 CI 완료 후 전체 시간, step별 시간, cache restore/save
  상태와 함께 남긴다.

### 2026-06-30 — 후속/draft 코드 PR #1702 최종 CI 관측

- PR: #1702 `Task #1664-CI:cargo cache save를 trusted branch로 제한`
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28430353568>
- Build & Test job: <https://github.com/edwardkim/rhwp/actions/runs/28430353568/job/84243307175>
- 이벤트: `pull_request`
- head SHA: `69229e7937dc08fb94bf5d6530f205de77c15fe4`
- 결론: 성공
- 해석 범위: #1702 draft 코드 PR 기준 관측값이며, #1702 merge 전에는 `devel` 반영 사실로 기록하지 않는다.

시간:

| 항목 | 시간 | 비고 |
|------|------|------|
| PR checks 완료 시간 | 약 19m23s | run created `08:17:48Z`, updated `08:37:11Z` |
| `CI / Build & Test` job | 19m08s | `08:18:02Z` - `08:37:10Z` |
| `Restore cargo registry & build cache` | 37s | 정확히 적중 |
| `Format check` | 4s | 실행됨 |
| `Build` | 3m33s | cargo finished `release` 3m32s |
| `Check WASM target` | 15s | 실행됨 |
| `Install native Skia runtime packages` | 18s | 실행됨 |
| `Native Skia tests` | 3m57s | cargo finished `release` 3m56s |
| `Run lib tests` | 3m46s | cargo finished `release` 3m34s |
| `Run integration tests` | 4m51s | cargo finished `release-test` 3m32s |
| `Clippy` | 21s | cargo finished `dev` 20.63s |
| `Save cargo registry & build cache` | skipped | PR restore-only 정책 확인 |

캐시:

| 항목 | 값 |
|------|----|
| restore 상태 | 정확히 적중 |
| cache key | `Linux-cargo-882d5ae97f721072735d2156a0d55566d62b9f4193a3e2c2fb1fa56ab8525f42` |
| cache 크기 | 약 1476 MB (`1547590748 B`) |
| restore 결과 | `Cache restored successfully` |
| save 상태 | skipped |
| cache reservation/read-only/save 실패 경고 | 관측되지 않음 |

컴파일 관측:

| step | cargo 관측 | 해석 |
|------|------------|------|
| Build | `Compiling rhwp`, `Finished release ... 3m 32s` | restore 이후에도 local crate compile 발생 |
| Native Skia tests | `Compiling rhwp`, `Finished release ... 3m 56s` | feature 조합 차이로 별도 산출물 가능 |
| Run lib tests | `Compiling rhwp`, `Finished release ... 3m 34s` | `release` profile local crate compile 발생 |
| Run integration tests | `Compiling rhwp`, `Finished release-test ... 3m 32s` | profile 차이로 별도 산출물 가능 |
| Clippy | `Finished dev ... 20.63s` | dev/check 계열 비용 |

회귀 가드 추적성:

| 항목 | 값 |
|------|----|
| 로컬 `tests/*.rs` | 162개 |
| 로컬 `tests/issue_*.rs` | 131개 |
| CI `Run integration tests` 실행 test binary | 165개 |
| CI issue 계열 실행 | 131개 |
| 판단 | issue 계열 131/131 실행 확인. 1:1 회귀 가드 추적성 보존 |

해석:

- #1664 목표인 PR cache save 차단은 의도대로 동작했다.
- cache restore는 정확히 적중했고 read-only/reservation/save 실패 경고는 관측되지 않았다.
- cache restore가 성공했더라도 PR merge ref와 프로필/feature 조합 때문에 최종 `rhwp` crate compile은 계속 발생했다.
- 이는 #1664 실패가 아니며, #1666 `release-test` profile 전환과 #1667 Rust cache 전략 비교의 before 기준으로 사용한다.
- after sample이 1개뿐이므로 P50/P90은 산출하지 않는다.
- runner-minutes 변화는 before/after 표본이 더 쌓인 뒤 판단한다.

## 관찰 메모

- #1664 적용 전 read-only 경고가 관측됐다.
- #1664의 1차 목표는 PR save 차단과 trusted branch save 정착이다.
- P50/P90 개선은 #1666 profile 전환 전에는 제한적일 수 있다.
- #1664 안정화 측정 후 #1667 진행 여부를 판단한다.
