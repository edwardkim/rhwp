# Task M100 #1664 측정 기록

## 목적

이 문서는 #1664 적용 전후 CI 측정값을 누적 기록하는 장기 보관 문서다. 단일 작업 완료 보고서가 아니라,
PR run과 `devel` / `main` push run이 쌓일 때마다 값을 추가하는 측정 원천 로그로 사용한다.

문서 PR #1701은 정책/측정 기록만 포함한다. 실제 workflow 변경은 후속 코드 PR #1702에서 다루며, 아래 PR
run 측정값은 #1702 draft 코드 PR 기준 관측값이다. #1702가 merge되기 전에는 이 workflow 변경이 `devel`에
반영된 사실로 해석하지 않는다.

2026-07-01 후속 문서 PR에서는 #1702 merge 이후 관측한 cache cleanup, trusted branch save, 후속 exact-hit
결과를 이 문서에 추가한다. GitHub 이슈 코멘트에 남긴 최종 관측을 장기 측정 원천 문서로 이관하는 목적이다.

2026-07-03 후속 문서 PR에서는 #1702 merge 이후 #1739 merge 전까지의 순수 #1664 구간 표본을 추가해
PR run과 trusted `devel` push run의 P50/P90을 보강한다. #1739 이후 run은 #1666 profile 전환 효과가
섞이므로 #1664 P50/P90에는 포함하지 않는다.

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
| 2026-07-01 | `devel` | [`28505355210` attempt 2](https://github.com/edwardkim/rhwp/actions/runs/28505355210/attempts/2) | `5e3b1ec652fda14a74af7cf9afd77962e3bb7903` | 22m53s | 4m43s | 4m55s | 3m44s | 6m23s | fallback `Linux-cargo-`, 426,792,350 B | success, `Linux-cargo-6a1af...` 저장 | 1,637,296,893 B | 없음 | stale PR ref cache cleanup 직후 trusted branch save 성공 |
| 2026-07-01 | `devel` | [`28507949075`](https://github.com/edwardkim/rhwp/actions/runs/28507949075) | `150ca316ee557d6bf95928302166e037d7467b03` | 18m02s | 3m32s | 4m00s | 3m50s | 4m41s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped | 1,637,296,893 B | 없음 | 직전 saved exact key 재사용 확인 |

## P50/P90 요약

샘플 수가 충분할 때 갱신한다. #1664 순수 구간은 #1702 merge 이후부터 #1739 merge 전까지로 잡는다.

- 시작: #1702 merge, 2026-07-01 14:36:10 KST
- 종료: #1739 merge 직전, 2026-07-01 22:58:02 KST
- 이유: #1739 이후에는 #1666 `release-test` profile 전환 효과가 섞여 #1664 cache 정책 효과와 분리해야 한다.

| 구간 | 대상 | 샘플 수 | P50 | P90 | 비고 |
|------|------|---------|-----|-----|------|
| before | PR checks 완료 시간 | TBD | TBD | TBD | 기존 run 수집 필요 |
| after | PR checks 완료 시간 | 20 | 23m25s | 26m50s | #1702 merge 후 #1739 merge 전, `Build & Test` 실행 PR success 기준. #1739/#1666 profile 변경 run 제외 |
| before | `CI / Build & Test` job 시간 | TBD | TBD | TBD | 기존 run 수집 필요 |
| after | PR `CI / Build & Test` job 시간 | 20 | 22m34s | 23m46s | #1664 PR cache 정책 구간. #1739/#1666 profile 변경 run 제외 |
| after | PR Build step | 20 | 4m45s | 4m58s | #1664 PR cache 정책 구간 |
| after | PR Native Skia tests step | 20 | 5m04s | 5m18s | #1664 PR cache 정책 구간 |
| after | PR Run lib tests step | 20 | 3m51s | 4m01s | #1664 PR cache 정책 구간 |
| after | PR Run integration tests step | 20 | 6m12s | 6m31s | #1664 PR cache 정책 구간 |
| after | trusted branch `Build & Test` job 시간 | 12 | 18m21s | 23m22s | #1702 merge 후 #1739 merge 전 `devel` push success |
| after | trusted branch Build step | 12 | 3m32s | 4m49s | #1702 merge 후 #1739 merge 전 |
| after | trusted branch Native Skia tests step | 12 | 4m00s | 5m03s | #1702 merge 후 #1739 merge 전 |
| after | trusted branch Run lib tests step | 12 | 3m45s | 4m01s | #1702 merge 후 #1739 merge 전 |
| after | trusted branch Run integration tests step | 12 | 4m49s | 6m23s | #1702 merge 후 #1739 merge 전 |
| after | trusted branch Clippy step | 12 | 26s | 46s | #1702 merge 후 #1739 merge 전 |

## #1702 merge 후 #1739 merge 전 순수 #1664 표본

이 절은 #1664 cache save 정책을 평가하기 위한 보강 표본이다. #1739 이후 run은 #1666 profile 전환으로
step time이 달라지므로 이 표본에서 제외한다.

### PR run

집계 기준:

- 기간: 2026-07-01 14:36:10 KST - 2026-07-01 22:58:02 KST
- `pull_request` success run 중 `Build & Test`가 실제 실행된 run: 21개
- 그중 #1739/#1666 profile 변경 run 1개는 step-time P50/P90에서 제외
- step-time P50/P90 표본: 20개
- fast-pass로 `Build & Test`가 skipped 된 success run: 10개

| 항목 | n | min | P50 | P90 | max |
|------|---|-----|-----|-----|-----|
| PR checks 완료 시간 | 20 | 15m38s | 23m25s | 26m50s | 34m59s |
| `CI / Build & Test` job | 20 | 15m25s | 22m34s | 23m46s | 24m32s |
| Build | 20 | 2m44s | 4m45s | 4m58s | 5m02s |
| Native Skia tests | 20 | 3m08s | 5m04s | 5m18s | 5m31s |
| Run lib tests | 20 | 2m59s | 3m51s | 4m01s | 4m10s |
| Run integration tests | 20 | 3m40s | 6m12s | 6m31s | 6m38s |

cache save:

| 항목 | 값 |
|------|----|
| `Build & Test` 실행 PR success run | 21개 |
| PR `Save cargo registry & build cache` | 21/21 skipped |
| fast-pass로 `Build & Test` skipped 된 PR success run | 10개 |
| 판단 | #1664의 PR restore-only / save 차단 정책이 누적 PR 표본에서도 유지됨 |

주의:

- #1739/#1666 PR run은 같은 시간창에 있지만 workflow 자체가 `release-test` profile로 바뀌므로 step-time
  P50/P90에는 포함하지 않는다.
- PR checks 완료 시간은 queue, update branch, cancel/retry 영향을 받으므로 `Build & Test` job 시간보다
  보조 지표로 해석한다.

### trusted `devel` push run

집계 기준:

- 기간: 2026-07-01 14:36:10 KST - 2026-07-01 22:58:02 KST
- `devel` push success run: 12개
- #1739 merge 이후 run 제외

| 항목 | n | min | P50 | P90 | max |
|------|---|-----|-----|-----|-----|
| `CI / Build & Test` job | 12 | 15m59s | 18m21s | 23m22s | 24m14s |
| Build | 12 | 2m52s | 3m32s | 4m49s | 4m50s |
| Native Skia tests | 12 | 3m18s | 4m00s | 5m03s | 5m10s |
| Run lib tests | 12 | 3m19s | 3m45s | 4m01s | 4m06s |
| Run integration tests | 12 | 3m53s | 4m49s | 6m23s | 6m40s |
| Clippy | 12 | 20s | 26s | 46s | 48s |

cache save:

| 항목 | 값 |
|------|----|
| trusted `devel` push success run | 12개 |
| `Save cargo registry & build cache` success | 4개 |
| `Save cargo registry & build cache` skipped | 8개 |
| 대표 saved/exact key | `Linux-cargo-6a1af...` |
| 대표 cache 크기 | 1,637,296,893 B |
| 판단 | 초기 trusted branch run에서 save가 수행되고, 이후 exact hit 상태에서는 save skipped로 안정화됨 |

해석:

- #1664의 핵심 정책인 PR save 차단과 trusted branch save 허용은 누적 표본에서도 유지됐다.
- trusted branch `Build & Test` P50은 18m21s로, #1666 적용 전 기준선을 형성한다.
- #1666 이후 `devel` push가 50분대로 증가한 현상은 #1664 cache 정책 효과가 아니라 #1666 release
  integration 정책 효과로 분리해 해석한다.

## cache 상태 요약

| 날짜 | 총 cache 크기 | 주요 key | PR ref cache 상태 | read-only 여부 | 비고 |
|------|---------------|----------|-------------------|----------------|------|
| 2026-07-01 cleanup 전 | 10GB 초과 | stale `refs/pull/*` cache 다수 | closed/merged PR ref cache 잔존 | read-only 원인 상태 | 메인테이너 승인 후 cleanup 진행 |
| 2026-07-01 1차 cleanup 직후 | 약 3.09 GiB | `Linux-cargo-`, `Linux-codeql-rust-` fallback 등 | `refs/pull/*` 0개 | 해소 기대 | closed/merged PR ref cache 18개 삭제 직후 |
| 2026-07-01 최종 확인 | 7,154,189,707 B, 약 6.66 GiB / 7.15 GB | `Linux-cargo-6a1af...` | OPEN PR #1729/#1730 cache만 남김 | Build & Test cargo cache 범위에서 경고 없음 | cleanup 중 merge된 PR #1722 cache 3개 추가 삭제 후 확인 |

## 관측 메모

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

### 2026-07-01 — #1702 merge 후 cache cleanup 및 trusted branch save 관측

- #1664 상세 이슈 코멘트: <https://github.com/edwardkim/rhwp/issues/1664#issuecomment-4853263686>
- #1668 부모 이슈 롤업: <https://github.com/edwardkim/rhwp/issues/1668#issuecomment-4853268643>

cleanup:

| 항목 | 결과 |
|------|------|
| 삭제 범위 | closed/merged `refs/pull/*` cache만 삭제 |
| 삭제 수 | 총 21개 |
| 세부 | 기존 stale PR ref cache 18개 + cleanup 중 merge된 PR #1722 cache 3개 |
| 유지 | `refs/heads/devel`, `refs/heads/main`, OPEN PR #1729/#1730 cache |
| 최종 총량 | 7,154,189,707 B, 약 6.66 GiB / 7.15 GB |
| budget 상태 | 10GB 아래 |

cleanup 직후 `devel` rerun:

| 항목 | 결과 |
|------|------|
| run | <https://github.com/edwardkim/rhwp/actions/runs/28505355210/attempts/2> |
| head | `5e3b1ec652fda14a74af7cf9afd77962e3bb7903` |
| 결론 | success |
| `CI / Build & Test` job | 22m53s |
| restore | fallback `Linux-cargo-`, 426,792,350 B |
| save | 성공, `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| save size | 1,637,296,893 B |
| 실패 원인 가시성 | `Cache reservation failed`, `Failed to save`, `##[error]` 없음 |

후속 `devel` run:

| 항목 | 결과 |
|------|------|
| run | <https://github.com/edwardkim/rhwp/actions/runs/28507949075> |
| head | `150ca316ee557d6bf95928302166e037d7467b03` |
| 결론 | success |
| `CI / Build & Test` job | 18m02s |
| restore | exact hit `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| restore size | 1,637,296,893 B |
| save | skipped, exact hit이므로 기대 동작 |
| 실패 원인 가시성 | `Cache reservation failed`, `Failed to save`, `##[error]` 없음 |

공통 준수사항:

- branch protection / required check 변경 없음.
- run head `5e3b1ec...` 기준 `tests/*.rs` 162개, `tests/issue_*.rs` 131개 유지.
- CI integration test binary 관측은 전체 165개, issue binary 131개다.
- post-merge `devel` run은 PR checks P50/P90 표본이 아니므로 PR checks 분포 산출에는 포함하지 않는다.
- 2026-07-03 보강에서 #1702 merge 후 #1739 merge 전 순수 #1664 구간의 trusted branch `Build & Test`
  표본 12개를 추가 집계해 P50/P90을 산출했다.

최종해석:

- #1702 정책은 end-to-end로 확인됐다. PR에서는 Build & Test cargo cache save가 skipped 되었고,
  trusted branch인 `devel`에서는 quota cleanup 후 exact cargo cache save가 성공했으며, 이후 `devel`
  run에서는 exact-hit로 restore되고 save가 skipped 되었다.
- `actions/cache@v5` PR save reservation/read-only 경고는 Build & Test cargo cache 범위에서 재현되지 않았다.
- quota cleanup은 실효가 있었다. cleanup 전 10GB 초과 상태에서 stale PR ref cache 제거 후 trusted branch save가
  가능해졌고, 최종 cache 총량도 10GB 아래로 유지됐다.
- OPEN PR run은 계속 `refs/pull/*` cache를 만들 수 있다. cleanup 정책은 closed/merged PR ref cache만 대상으로
  유지하는 편이 안전하다.

## 관찰 메모

- #1664 적용 전 read-only 경고가 관측됐다.
- #1664의 1차 목표는 PR save 차단과 trusted branch save 정착이다.
- P50/P90 개선은 #1666 profile 전환 전에는 제한적일 수 있다.
- #1664 안정화 측정 결과는 #1666 profile 전환 전 기준선과 #1667 cache 전략 재평가 기준선으로 사용한다.
