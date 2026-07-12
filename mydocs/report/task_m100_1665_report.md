# Task M100 #1665 최종 보고서

## 개요

- 이슈: #1665 `[CI] Build & Test job 병렬 분리 설계`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 코드 PR: #1877
- 측정 문서: `mydocs/report/task_m100_1665_measurement.md`
- 결론: 완료

## 변경 결과

#1877에서 기존 단일 `CI / Build & Test` job을 다음 구조로 바꿨다.

| job | 역할 |
|-----|------|
| `Build default-feature tests` | fmt, build, default-feature tests, WASM check, clippy, 단일 cache save writer |
| `Native Skia tests` | native-skia feature lib tests |
| `Build & Test` | 기존 required check 이름을 유지하는 aggregate gate |

테스트 파일 통합, 회귀 가드 명명 규칙 변경, `tests/golden_svg/**` 구조 변경은 하지 않았다.

## 최종 측정 요약

before 기준선:

- #1872 merge 후 `devel` push run
- `CI / Build & Test`: 12m45s

after 표본:

- PR: successful full CI run 22개
- `devel` push: successful full CI run 19개

| 구간 | 항목 | P50 | P90 | 판단 |
|------|------|-----|-----|------|
| PR | checks 완료 시간 | 9m12s | 9m51s | queue 포함 운영 지표 |
| PR | Build & Test critical path | 8m47s | 9m10s | before 12m45s 대비 개선 |
| PR | runner-minutes proxy | 12m44s | 13m22s | P50은 before와 거의 동일 |
| `devel` push | checks 완료 시간 | 10m49s | 17m39s | P90은 queue 대기 영향 |
| `devel` push | Build & Test critical path | 10m38s | 11m34s | before 12m45s 대비 개선 |
| `devel` push | runner-minutes proxy | 14m42s | 15m50s | 별도 worker setup 비용으로 증가 |

주요 step:

| 구간 | Build | Run default-feature tests | Native Skia tests | Check WASM | Clippy |
|------|-------|---------------------------|-------------------|------------|--------|
| before | 3m29s | 5m03s | 2m15s | 15s | 22s |
| PR P50 | 1m34s | 4m58s | 2m16s | 15s | 25s |
| PR P90 | 1m39s | 5m11s | 2m24s | 16s | 27s |
| `devel` push P50 | 3m31s | 4m57s | 2m13s | 15s | 25s |
| `devel` push P90 | 3m46s | 5m11s | 2m23s | 17s | 26s |

## cache / required check / 회귀 가드

cache:

- 대표 after run에서 `Build default-feature tests`, `Native Skia tests` 모두 exact hit
- cache key: `Linux-cargo-6a1af...`
- cache size: 1,637,296,893 B, 약 1561 MB
- PR save: skipped 유지
- Native Skia job: save step 없음
- cache reservation / read-only / save failure 경고 없음

required check:

- `CI / Build & Test` 이름을 aggregate gate로 유지
- branch protection / required check 이름 변경 없음으로 관측
- 새 worker check는 추가 노출되지만 required check 이름을 대체하지 않음

회귀 가드:

- 최신 대표 run 기준 `Run default-feature tests`: 192개 test harness, 2859 passed, 21 ignored
- `Native Skia tests`: 48 passed, 2112 filtered out
- 최신 워크트리 기준 `tests/*.rs` 189개, `tests/issue*.rs` 158개
- 메인테이너가 최초 기준으로 언급한 162개 회귀 가드는 줄지 않았고, 현재 증가한 회귀 가드도 PR마다 실행된다.

## 최종 해석

이번 개선은 step 자체의 속도를 크게 줄인 작업이 아니라, Native Skia 경로를 기본 feature 회귀 경로와
겹쳐 실행하도록 만든 작업이다.

따라서 성과는 다음처럼 평가한다.

- PR 경로는 critical path P50 8m47s, P90 9m10s로 개선됐다.
- `devel` push 경로도 critical path P50 10m38s, P90 11m34s로 개선됐다.
- `devel` push checks 완료 P90 17m39s는 workflow 구조보다 runner queue 대기 영향이 크다.
- runner-minutes proxy는 특히 `devel` push에서 증가했지만, 수행계획서의 재검토 기준인 +40% 증가에는
  도달하지 않았다.
- failure visibility는 좋아졌다. Native Skia 실패와 default-feature 실패가 별도 worker job으로 분리되고,
  aggregate `Build & Test`는 실패 worker를 `::error::`로 표시한다.

## 결론

#1665는 완료로 본다.

현재 1차 병렬화 구조는 유지할 가치가 있다. 추가 병렬화는 setup/cache restore 중복으로 runner-minutes가
더 늘 가능성이 높으므로, 바로 쪼개기보다 queue 대기와 runner-minutes 추이를 더 본 뒤 별도 이슈로 판단하는
것이 적절하다.
