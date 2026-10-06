# Task M100 #1668 최종 보고서

## 개요

- 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 작업 기간: 2026-06-29 ~ 2026-07-04
- 성격: CI Build & Test 파이프라인 정책 결정, 단계별 구현, before/after 측정 총괄
- 부모 추적 문서: `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`

이 문서는 #1668 하위 이슈 #1664, #1666, #1849, #1667, #1665의 최종 성과를 한 번에 보기 위한
요약 보고서다. 개별 raw 측정값과 run URL은 각 하위 이슈의 measurement 문서와 부모 추적 문서에 보존한다.

## 결론 요약

#1668은 목표를 달성했다.

메인테이너 결정사항인 “회귀 가드 1:1 추적성 보존, PR은 빠른 회귀 검출, trusted branch와 release 지점에서
더 깊은 검증, PR cache restore-only, 문서/코드 PR 분리”를 유지하면서 PR CI wall time을 크게 줄였다.

핵심 성과:

| 지표 | before | after | 변화 |
|------|--------|-------|------|
| PR checks 완료 P50 | 23m25s | 9m12s | -14m13s, -60.7% |
| PR checks 완료 P90 | 26m50s | 9m51s | -16m59s, -63.3% |
| PR Build & Test 비교 wall time P50 | 22m34s | 8m47s | -13m47s, -61.1% |
| PR Build & Test 비교 wall time P90 | 23m46s | 9m10s | -14m36s, -61.4% |
| trusted branch Build & Test 비교 wall time P50 | 18m21s | 10m38s | -7m43s, -42.1% |
| trusted branch Build & Test 비교 wall time P90 | 23m22s | 11m34s | -11m48s, -50.5% |

주의:

- before는 #1702 merge 후 #1739 merge 전의 순수 #1664 구간이다. 이 구간은 cache 정책이 정리됐지만
  profile 전환과 병렬화가 적용되기 전이므로, #1668 최적화 전 기준선으로 사용한다.
- after는 마지막 하위 이슈인 #1665 merge 이후 successful full CI run 표본이다.
- #1665 이후 `Build & Test` job 자체는 aggregate gate가 되었으므로, 변경 전 단일 job과 비교할 때는
  `Build & Test critical path`를 사용한다.

## 하위 이슈별 역할

| 이슈 | 역할 | 결과 |
|------|------|------|
| #1664 | PR cache save 차단, trusted branch save 정리 | PR save skipped, trusted branch save/exact-hit 확인, stale PR ref cleanup |
| #1666 | PR profile을 `release-test` 중심으로 전환 | PR `Build & Test` 19m08s -> 10m49s, 단 `devel` full release 비용 증가 관측 |
| #1849 | `devel` push profile 배치 재조정 | `devel` push full `release --tests` 50분대 비용을 release smoke + `release-test` 회귀로 조정 |
| #1667 | Rust cache/fingerprint/cache cleanup 실효성 분석 | CodeQL/Render Diff PR save 표면 정리, read-only/reservation 경고 제거, 중복 test step 정리 |
| #1665 | Build & Test 1차 병렬화 | Native Skia worker 분리 + aggregate gate로 PR/devel critical path 추가 감소 |

## before / after 기준

### before

before는 #1668 진행 중 가장 이른 안정 P50/P90 표본인 “#1664 merge 후 #1666 merge 전 순수 구간”을 사용한다.

이 구간의 의미:

- PR cache save 차단과 trusted branch save 정책은 정리된 상태
- 아직 PR `release-test` 전환, `devel` push profile 재조정, cache 세부 정리, job 병렬화는 적용 전
- 따라서 “캐시 정책 안정화 이후, 실질적 시간 단축 전” 기준선으로 적합

before PR 표본:

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| PR checks 완료 시간 | 20 | 23m25s | 26m50s |
| PR `CI / Build & Test` job | 20 | 22m34s | 23m46s |
| Build | 20 | 4m45s | 4m58s |
| Native Skia tests | 20 | 5m04s | 5m18s |
| Run lib tests | 20 | 3m51s | 4m01s |
| Run integration tests | 20 | 6m12s | 6m31s |

before trusted branch 표본:

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| trusted branch `Build & Test` job | 12 | 18m21s | 23m22s |
| Build | 12 | 3m32s | 4m49s |
| Native Skia tests | 12 | 4m00s | 5m03s |
| Run lib tests | 12 | 3m45s | 4m01s |
| Run integration tests | 12 | 4m49s | 6m23s |
| Clippy | 12 | 26s | 46s |

### after

after는 #1665 merge 이후 successful full CI run 표본을 사용한다.

after PR 표본:

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| PR checks 완료 시간 | 22 | 9m12s | 9m51s |
| Build & Test critical path | 22 | 8m47s | 9m10s |
| runner-minutes proxy | 22 | 12m44s | 13m22s |
| Build | 22 | 1m34s | 1m39s |
| Run default-feature tests | 22 | 4m58s | 5m11s |
| Native Skia tests | 22 | 2m16s | 2m24s |
| Check WASM target | 22 | 15s | 16s |
| Clippy | 22 | 25s | 27s |

after `devel` push 표본:

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| checks 완료 시간 | 19 | 10m49s | 17m39s |
| Build & Test critical path | 19 | 10m38s | 11m34s |
| runner-minutes proxy | 19 | 14m42s | 15m50s |
| Build | 19 | 3m31s | 3m46s |
| Run default-feature tests | 19 | 4m57s | 5m11s |
| Native Skia tests | 19 | 2m13s | 2m23s |
| Check WASM target | 19 | 15s | 17s |
| Clippy | 19 | 25s | 26s |

`devel` push checks 완료 P90 17m39s는 worker critical path P90 11m34s보다 길다. 대표 outlier들은 runner queue
대기가 컸기 때문에, workflow 구조 개선 효과는 critical path로 해석한다.

## 단계별 성과

### #1664 cache 정책

목표:

- PR에서는 cache restore만 수행하고 save를 차단한다.
- trusted branch에서만 save를 허용한다.
- stale PR ref cache cleanup으로 read-only/reservation 문제를 해소한다.

결과:

- PR `Build & Test` 실행 표본에서 save skipped 유지
- cleanup 후 trusted branch에서 `Linux-cargo-6a1af...` save 성공
- 후속 `devel` run에서 exact-hit restore 후 save skipped 확인
- closed/merged `refs/pull/*` cache cleanup 후 10GB budget 아래 회복

의미:

- 이후 #1666, #1667, #1665 측정이 cache quota/read-only 노이즈에 덜 흔들리게 됐다.
- PR cache poisoning 위험과 save 경합 표면을 줄였다.

### #1666 PR profile 전환

목표:

- PR에서 full `release` profile 비용을 제거하고 `release-test` 중심으로 회귀 검출을 빠르게 한다.

결과:

| 항목 | before | after PR | 변화 |
|------|--------|----------|------|
| PR `CI / Build & Test` | 19m08s | 10m49s | -8m19s, -43.5% |
| Build | 3m33s | 1m30s | -2m03s |
| Native Skia tests | 3m57s | 2m05s | -1m52s |
| Run lib tests | 3m46s | 1m38s | -2m08s |
| Run integration tests | 4m51s | 3m39s | -1m12s |

의미:

- PR 피드백 루프 단축의 가장 큰 단일 변화였다.
- 반대로 trusted `devel` push에서 full `release --tests`를 엄격히 수행하면서 50분대 비용이 드러났고,
  이것이 #1849의 근거가 됐다.

### #1849 `devel` push profile 배치 재조정

목표:

- #1666 이후 `devel` push에서 드러난 full `release --tests` 비용을 줄인다.
- release profile 자체의 파손은 release smoke로 조기 감지한다.

결과:

| 항목 | #1666 merge 후 P50 | #1849 merge 후 | 변화 |
|------|--------------------|----------------|------|
| `devel` `Build & Test` | 56m55s | 14m13s | -42m42s, -75.0% |
| `devel` Run integration tests | 42m27s | 3m58s | -38m29s, -90.7% |

의미:

- `devel` push가 다시 운영 가능한 시간대로 내려왔다.
- PR은 빠른 회귀 검출, `devel`은 release smoke + `release-test` 전체 회귀, release/tag/nightly는 더 깊은
  release-grade 검증이라는 역할 분리가 명확해졌다.

### #1667 cache/fingerprint 정리

목표:

- CodeQL Rust / Render Diff cache save 표면을 정리한다.
- exact hit 이후에도 남는 `Dirty rhwp` / `Compiling rhwp`를 cache 실패와 구분한다.
- Build & Test의 중복 test step을 정리한다.

결과:

- CodeQL Rust cache restore/save 분리
- Render Diff cargo cache restore/save 분리, npm cache 제거
- Render Diff PR run의 cache reservation/read-only/save failure 경고 제거
- closed/merged PR ref cache 수동 cleanup으로 cache 총량 10GB 아래 회복
- `Run lib tests` 중복 step 제거 후 `devel` `Build & Test` 13m43s -> 12m45s

의미:

- cache 실패와 Cargo 산출물 경계 문제를 분리해 볼 수 있게 됐다.
- `Dirty rhwp`가 남는 것은 cache 정책 실패가 아니라 profile/feature/test target 산출물 경계의 결과로 정리됐다.

### #1665 1차 병렬화

목표:

- Native Skia tests를 별도 worker로 분리하되, required check 이름은 `CI / Build & Test`로 유지한다.

결과:

| 구간 | before | after P50 | after P90 | 판단 |
|------|--------|-----------|-----------|------|
| PR Build & Test 비교 wall time | 12m45s | 8m47s | 9m10s | Native Skia 경로 병렬화 효과 |
| `devel` Build & Test 비교 wall time | 12m45s | 10m38s | 11m34s | queue 제외 critical path 개선 |

의미:

- 마지막 단계에서 PR critical path를 9분 안팎으로 낮췄다.
- runner-minutes proxy는 별도 worker setup 때문에 `devel` push에서 증가했지만, 재검토 기준인 +40%에는
  미치지 않았다.

## 공통 준수사항 결과

| 항목 | 결과 |
|------|------|
| PR checks 완료 시간 P50/P90 | before 23m25s/26m50s -> after 9m12s/9m51s |
| CI Build & Test 시간 | before PR 22m34s/23m46s -> after PR critical path 8m47s/9m10s |
| 주요 step 시간 | build/default-feature/native-skia/WASM/clippy를 하위 measurement 문서에 기록 |
| cache hit/miss/save | PR restore-only, trusted branch save-only, representative exact-hit 확인 |
| cache 크기 | 대표 cargo cache 1,637,296,893 B, 약 1561 MB |
| 실패 시 원인 가시성 | aggregate `Build & Test`가 실패 worker를 `::error::`로 표시. Native Skia/default-feature job 분리 |
| runner-minutes 변화 | PR proxy P50 22m34s -> 12m44s, `devel` proxy P50 18m21s -> 14m42s |
| branch protection / required check | `CI / Build & Test` check 이름 유지 |
| 회귀 가드 1:1 추적성 | 테스트 파일 통합/명명 변경/golden 구조 변경 없음. 최신 `tests/*.rs` 189개, `tests/issue*.rs` 158개 |

## 시각화용 데이터

추후 PDF/그래프 작성 시 아래 데이터를 그대로 사용한다.

### PR 전체 성과

| 지표 | before P50 seconds | after P50 seconds | before P90 seconds | after P90 seconds |
|------|--------------------|-------------------|--------------------|-------------------|
| PR checks 완료 | 1405 | 552 | 1610 | 591 |
| PR Build & Test 비교 wall time | 1354 | 527 | 1426 | 550 |
| PR runner-minutes proxy | 1354 | 764 | 1426 | 802 |

### trusted branch 성과

| 지표 | before P50 seconds | after P50 seconds | before P90 seconds | after P90 seconds |
|------|--------------------|-------------------|--------------------|-------------------|
| trusted Build & Test 비교 wall time | 1101 | 638 | 1402 | 694 |
| trusted runner-minutes proxy | 1101 | 882 | 1402 | 950 |

### 단계별 `Build & Test` 대표값

| 단계 | PR / trusted | 값 |
|------|--------------|----|
| #1664 순수 PR baseline | PR P50 | 22m34s |
| #1666 PR profile 전환 | PR 단일 | 10m49s |
| #1666 full release trusted 비용 | trusted P50 | 56m55s |
| #1849 devel profile 재조정 | trusted 단일 | 14m13s |
| #1667 중복 step 정리 | trusted 단일 | 12m45s |
| #1665 최종 병렬화 | PR P50 critical path | 8m47s |
| #1665 최종 병렬화 | trusted P50 critical path | 10m38s |

## 남은 운영 판단

현재 #1668의 하위 작업은 모두 완료됐다. 다만 다음 항목은 별도 이슈 또는 운영 정책으로 남길 수 있다.

- GitHub Actions runner queue 대기 관측: `checks 완료 시간`과 `critical path`의 차이를 계속 분리해 본다.
- cleanup 자동화: 문제가 반복될 때만 `pull_request.closed` + allowlist 기반으로 별도 이슈/PR에서 검토한다.
- 추가 병렬화: runner-minutes 증가와 required check 표면 복잡도가 커지므로 현 시점에서는 보류한다.
- release-grade 검증: tag/release/nightly에서 full `release` 검증을 유지하거나 강화하는 방향으로 별도 추적한다.

## 최종 판단

#1668은 close 가능하다.

이번 작업은 CI를 단순히 빠르게 만든 것이 아니라, rhwp의 회귀 가드 1:1 추적성을 유지하면서 CI 역할을
분리했다는 점이 핵심이다.

- PR: 빠른 `release-test` 전체 회귀 검출
- `devel`: release smoke + `release-test` 전체 회귀
- release/tag/nightly: release-grade 검증 후보
- cache: PR restore-only, trusted branch save-only
- required check: `CI / Build & Test` 이름 유지

따라서 #1668은 “정책 결정, 단계별 구현, measurement 문서화, 최종 before/after 보고”까지 완료된 상태다.
