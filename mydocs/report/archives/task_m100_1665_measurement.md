# Task M100 #1665 최종 측정 기록

## 개요

- 이슈: #1665 `[CI] Build & Test job 병렬 분리 설계`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 코드 PR: #1877
- merge commit: `71e6d8ec7fb65ec91351a2545fc7d6da26baed5f`
- 측정일: 2026-07-04

이 문서는 #1877 merge 이후 누적된 PR / `devel` push CI 표본을 기준으로 #1665의
before/after 측정값을 정리한다.

## 측정 방법

수집 대상:

- workflow: `CI`
- 포함 조건: successful full CI run, `Build default-feature tests`, `Native Skia tests`,
  aggregate `Build & Test`가 모두 확인되는 run
- 제외 조건: fast-pass로 worker job이 skipped 된 run, worker job 누락 run, 실패 run
- PR 표본 기간: 2026-07-03 18:50 UTC 이후
- `devel` push 표본 기간: #1877 merge 시각인 2026-07-03 20:01:13 UTC 이후
- P50: 중앙값
- P90: nearest-rank 방식

시간 정의:

| 항목 | 정의 |
|------|------|
| PR checks 완료 시간 | 동일 head SHA에서 필요한 check workflow가 모두 끝난 시간. queue 대기 포함 |
| CI run 완료 시간 | `CI` workflow run createdAt -> updatedAt |
| Build & Test critical path | 두 worker job 시작 중 가장 이른 시각 -> aggregate `Build & Test` 완료 |
| runner-minutes proxy | `Build default-feature tests` + `Native Skia tests` + aggregate `Build & Test` job duration 합 |
| aggregate `Build & Test` job | required check 이름을 보존하는 판정 job 자체의 duration |

주의:

- #1665 이후 `Build & Test` job은 실제 build/test를 수행하는 job이 아니라 aggregate gate다.
  따라서 변경 전 단일 `CI / Build & Test` job과 직접 비교할 때는 aggregate job duration이 아니라
  `Build & Test critical path`를 사용한다.
- `checks 완료 시간`은 queue 대기를 포함한다. runner 위에서 실제로 걸린 시간은 `critical path`가 더 잘 나타낸다.

## before 기준선

기준선은 #1872 merge 후 `devel` push run이다.

- run: <https://github.com/edwardkim/rhwp/actions/runs/28676046470>
- `Build & Test` job: <https://github.com/edwardkim/rhwp/actions/runs/28676046470/job/85049578280>
- cache: exact hit `Linux-cargo-6a1af...`
- cache size: 1,637,296,893 B, 약 1561 MB
- save: skipped
- cache reservation / read-only / save failure 경고: 없음

| 항목 | before |
|------|--------|
| CI run 완료 시간 | 12m57s |
| `CI / Build & Test` job | 12m45s |
| Build | 3m29s |
| Run default-feature tests | 5m03s |
| Check WASM target | 15s |
| Install native Skia runtime packages | 9s |
| Native Skia tests | 2m15s |
| Clippy | 22s |

회귀 가드:

- `Run default-feature tests`: 2081 tests, 2075 passed, 6 ignored
- `Native Skia tests`: 48 tests, 48 passed
- 당시 `tests/*.rs`: 182개
- 당시 `tests/issue*.rs`: 151개

## after: PR 표본

표본:

- successful full PR CI run 22개
- fast-pass / worker 누락 / 실패 등 제외 run 20개

| 항목 | n | min | P50 | P90 | max |
|------|---|-----|-----|-----|-----|
| PR checks 완료 시간 | 22 | 8m33s | 9m12s | 9m51s | 10m47s |
| CI run 완료 시간 | 22 | 8m33s | 9m12s | 9m42s | 10m47s |
| Build & Test critical path | 22 | 8m23s | 8m47s | 9m10s | 9m38s |
| runner-minutes proxy | 22 | 12m01s | 12m44s | 13m22s | 13m43s |
| Build default-feature tests job | 22 | 8m24s | 8m39s | 9m07s | 9m32s |
| Native Skia tests job | 22 | 3m29s | 3m46s | 4m16s | 4m17s |
| aggregate `Build & Test` job | 22 | 2s | 4s | 4s | 4s |
| Build step | 22 | 1m30s | 1m34s | 1m39s | 1m42s |
| Run default-feature tests step | 22 | 4m48s | 4m58s | 5m11s | 5m24s |
| Native Skia tests step | 22 | 1m46s | 2m16s | 2m24s | 2m26s |
| Check WASM target | 22 | 14s | 15s | 16s | 17s |
| Clippy | 22 | 22s | 25s | 27s | 27s |

before 대비:

| 항목 | before | after PR P50 | 변화 |
|------|--------|--------------|------|
| 비교용 Build & Test wall time | 12m45s | 8m47s | -3m58s |
| Build & Test P90 | 12m45s | 9m10s | -3m35s |
| runner-minutes proxy | 12m45s | 12m44s | -1s |
| runner-minutes proxy P90 | 12m45s | 13m22s | +37s |

해석:

- PR 경로는 Native Skia 경로를 기본 feature 경로와 겹쳐 실행하면서 critical path가 P50 기준
  8m47s까지 줄었다.
- step 자체가 크게 빨라진 것이 아니라, Native Skia package install/test 시간이 default-feature job과
  병렬로 겹쳐진 것이 주된 개선 요인이다.
- PR runner-minutes proxy는 P50 기준 before와 거의 같고, P90도 +37s 수준이라 수행계획서의
  재검토 기준인 +40% 증가에 해당하지 않는다.

## after: `devel` push 표본

표본:

- #1877 merge 이후 successful full `devel` push CI run 19개

| 항목 | n | min | P50 | P90 | max |
|------|---|-----|-----|-----|-----|
| checks 완료 시간 | 19 | 9m01s | 10m49s | 17m39s | 18m38s |
| CI run 완료 시간 | 19 | 8m07s | 10m49s | 17m39s | 18m38s |
| Build & Test critical path | 19 | 7m56s | 10m38s | 11m34s | 12m11s |
| runner-minutes proxy | 19 | 11m21s | 14m42s | 15m50s | 15m58s |
| Build default-feature tests job | 19 | 7m56s | 10m29s | 11m29s | 12m04s |
| Native Skia tests job | 19 | 3m21s | 4m09s | 4m18s | 4m21s |
| aggregate `Build & Test` job | 19 | 2s | 3s | 4s | 4s |
| Build step | 19 | 2m24s | 3m31s | 3m46s | 4m08s |
| Run default-feature tests step | 19 | 3m09s | 4m57s | 5m11s | 5m40s |
| Native Skia tests step | 19 | 1m43s | 2m13s | 2m23s | 2m24s |
| Check WASM target | 19 | 10s | 15s | 17s | 17s |
| Clippy | 19 | 15s | 25s | 26s | 27s |

before 대비:

| 항목 | before | after push P50 | 변화 |
|------|--------|----------------|------|
| 비교용 Build & Test wall time | 12m45s | 10m38s | -2m07s |
| Build & Test P90 | 12m45s | 11m34s | -1m11s |
| runner-minutes proxy | 12m45s | 14m42s | +1m57s |
| runner-minutes proxy P90 | 12m45s | 15m50s | +3m05s |

queue 대기:

- `devel` push의 checks 완료 P90 17m39s는 worker critical path P90 11m34s보다 길다.
- 대표 outlier run들은 workflow createdAt 이후 runner job 시작까지 5-7분대 queue 대기가 있었다.
- 따라서 `devel` push에서 실제 workflow 구조 개선 효과는 `critical path`로 보는 것이 맞고,
  checks 완료 P90은 runner 대기까지 포함한 운영 지표로 별도 보존한다.

해석:

- `devel` push는 PR보다 Build step이 길다. 이는 trusted event에서 의도대로 `release` build smoke를
  수행하기 때문이다.
- 그래도 Native Skia 분리로 critical path P50은 before 12m45s에서 10m38s로 줄었다.
- runner-minutes proxy는 P50 기준 +1m57s, P90 기준 +3m05s 늘었다. wall time 감소를 얻기 위해
  setup/toolchain/cache restore가 Native Skia job에서 한 번 더 수행되는 비용을 지불한 결과다.

## cache 관측

대표 최신 `devel` push run:

- run: <https://github.com/edwardkim/rhwp/actions/runs/28708494922>
- head SHA: `08bc3fc2c15a2a83e4cc2587644a1e5c1389ce0c`

`Build default-feature tests`:

- restore: exact hit
- cache key: `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2`
- cache size: 약 1561 MB, 1,637,296,893 B
- save: exact hit이므로 skipped
- `Compiling rhwp`: Build step과 Run default-feature tests step에서 2회

`Native Skia tests`:

- restore: exact hit
- cache key: `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2`
- cache size: 약 1561 MB, 1,637,296,893 B
- save: step 없음
- `Compiling rhwp`: native-skia feature test harness에서 1회

cache warning:

- representative PR / push run에서 cache reservation 경고 없음
- read-only 경고 없음
- save failure 경고 없음

판단:

- #1664 / #1667에서 정리한 “PR restore-only, trusted branch 단일 writer save” 정책은 #1665 후에도 유지됐다.
- Native Skia job은 별도 runner이므로 target 산출물을 default-feature job과 직접 공유하지 않는다.
  따라서 `Compiling rhwp`가 각 feature/profile 조합에서 남는 것은 cache 실패가 아니라 산출물 경계의 결과다.

## 회귀 가드 / test count

최신 대표 run `28708494922` 기준:

| 항목 | 값 |
|------|----|
| `Run default-feature tests` | 192개 test harness, 2859 passed, 21 ignored |
| `Native Skia tests` | 48 passed, 2112 filtered out |
| `tests/*.rs` | 189개 |
| `tests/issue*.rs` | 158개 |
| `tests/golden_svg` 파일 | 7개 |

판단:

- 메인테이너가 최초 기준으로 언급한 162개 회귀 가드는 줄지 않았다.
- 이후 저장소에 test file과 issue 계열 회귀 가드가 늘어난 상태에서도 `cargo test --tests` 경로가 전체
  default-feature integration harness를 계속 실행한다.
- #1665는 테스트 파일 통합, 명명 규칙 변경, golden asset 구조 변경을 하지 않았다.

## branch protection / required check

- workflow job `name: Build & Test`는 aggregate gate로 유지했다.
- required check 표면은 `CI / Build & Test` 이름을 유지한다.
- 새 worker job인 `Build default-feature tests`, `Native Skia tests`는 별도 check로 보이지만 required check
  이름을 대체하지 않는다.
- #1877 merge 이후 후속 PR들이 같은 `CI / Build & Test` aggregate context로 통과/merge됐다.

주의:

- GitHub branch protection REST endpoint는 현재 권한/설정에서 404를 반환해 설정 원문은 문서화하지 못했다.
- 관측 가능한 check 표면과 merge 결과 기준으로는 required check 이름 변경이 없다.

## 실패 시 원인 가시성

- aggregate `Build & Test`는 worker result를 로그에 출력한다.
- worker failure / cancelled / skipped가 발생하면 aggregate step이 `::error::`로 실패 worker를 표시한다.
- worker job이 분리되어 Native Skia 실패와 default-feature 실패는 GitHub Actions UI에서 별도 job으로
  직접 확인 가능하다.
- 이번 after 표본에 포함한 successful full run에서는 실패 stderr 위치 사례가 없었다.

## 최종 판단

#1665의 1차 병렬화는 효과가 있다.

- PR 경로 critical path P50: 12m45s -> 8m47s
- PR 경로 critical path P90: 12m45s -> 9m10s
- `devel` push critical path P50: 12m45s -> 10m38s
- `devel` push critical path P90: 12m45s -> 11m34s

다만 runner-minutes proxy는 `devel` push에서 증가했다. 이는 Native Skia worker의 checkout, disk cleanup,
toolchain, cache restore가 별도 runner에서 반복되기 때문이다. 증가 폭은 수행계획서의 재검토 기준
(+40% 이상)에는 미치지 않는다.

따라서 현 구조는 유지할 수 있다. 다음 최적화가 필요하다면 #1665 안에서 더 쪼개기보다, queue 대기와
runner-minutes trade-off를 별도 운영 지표로 추적한 뒤 판단하는 편이 낫다.
