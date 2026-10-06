# Task M100 #1849 최종 보고서

## 개요

- 이슈: #1849 `[CI] devel push profile 배치 재조정: release-test 회귀 + release smoke`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 코드 PR: #1851 `Task #1849: devel push profile 배치 재조정`
- merge commit: `d76f1997f07cc14f06d80d18cbec1d1b36b0839c`
- merge 시각: 2026-07-03 17:50:55 KST
- 작업 성격: #1666 후속 profile 배치 정책 조정 및 before/after measurement 기록

#1849는 #1666 구현 오류를 수정하는 작업이 아니다. #1666에서 PR 경로를 `release-test` 중심으로
전환한 효과는 유지하고, #1666 merge 후 `devel` push에서 드러난 full `release --tests` integration 비용을
줄이기 위해 `devel` push의 profile 배치를 재조정한 작업이다.

## 최종 정책

| 이벤트 / ref | Build | Native Skia tests | Run lib tests | Run integration tests | 의미 |
|--------------|-------|-------------------|---------------|-----------------------|------|
| `pull_request` | `release-test` | `release-test` | `release-test` | `release-test` | PR 빠른 전체 회귀 검증 |
| `push` `refs/heads/devel` | `release` | `release-test` | `release-test` | `release-test` | merge 후 release build smoke + 빠른 전체 회귀 검증 |
| `push` `refs/heads/main` | `release` | `release` | `release` | `release` | main 통합 강검증 |
| tag push `refs/tags/v*` | `release` | `release` | `release` | `release` | 릴리스 검증 |
| `workflow_dispatch` | `release` | `release` | `release` | `release` | 수동 강검증 |

별도 `Release smoke` job은 추가하지 않았다. 기존 `Build` step이 `devel` push에서
`cargo build --release --verbose`를 계속 실행하므로, 이 step을 release smoke로 해석한다.

## PR CI 관측

- PR: #1851
- 최종 PR head SHA: `5c2afe845ec4fe4afb0eacba49ffb33f22691fc2`
- PR run: <https://github.com/edwardkim/rhwp/actions/runs/28648923176>
- 이벤트: `pull_request`
- 결론: 성공

### 시간

| 항목 | 시간 |
|------|------|
| PR checks 완료 시간 | 12m09s |
| `CI / Build & Test` job | 11m59s |
| Build | 1m30s |
| Check WASM target | 15s |
| Install native Skia runtime packages | 10s |
| Native Skia tests | 2m16s |
| Run lib tests | 1m47s |
| Run integration tests | 3m53s |
| Clippy | 26s |

### profile / cache / 회귀 가드

| 항목 | 관측 |
|------|------|
| Build profile | `profile=release-test event=pull_request ref=refs/pull/1851/merge` |
| Native Skia profile | `profile=release-test event=pull_request ref=refs/pull/1851/merge` |
| Run lib tests profile | `profile=release-test event=pull_request ref=refs/pull/1851/merge` |
| Run integration tests profile | `profile=release-test event=pull_request ref=refs/pull/1851/merge` |
| cache restore | exact hit `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| cache size | 1,637,296,893 B, 약 1561 MB |
| cache save | PR run에서 skipped |
| cache 경고 | reservation/read-only/save failure/`##[error]` 없음 |
| 회귀 가드 | `Run integration tests` normalized test binary 180개, issue 계열 147개 |

PR run에서는 #1666 이후의 `release-test` 전체 회귀 검증 경로가 유지됐다. `Build & Test` job 이름과
check 표면은 변경되지 않았다.

`Dirty rhwp` 신호는 Build, Run lib tests, Run integration tests에서 관측됐다. 다만 PR path는 모두
`release-test`이므로 #1666 이전의 full `release` LTO/codegen 비용으로 되돌아가지는 않았다.

## merge 후 `devel` push 관측

- merge commit: `d76f1997f07cc14f06d80d18cbec1d1b36b0839c`
- CI run: <https://github.com/edwardkim/rhwp/actions/runs/28649575142>
- 이벤트: `push`
- ref: `refs/heads/devel`
- 결론: 성공

### 시간

| 항목 | 시간 |
|------|------|
| CI workflow wall time | 15m06s |
| `CI / Build & Test` job | 14m13s |
| Build | 3m38s |
| Check WASM target | 16s |
| Install native Skia runtime packages | 10s |
| Native Skia tests | 2m13s |
| Run lib tests | 1m48s |
| Run integration tests | 3m58s |
| Clippy | 26s |

### profile / cache / 회귀 가드

| 항목 | 관측 |
|------|------|
| Build profile | `profile=release event=push ref=refs/heads/devel role=release-smoke-or-full` |
| Native Skia profile | `profile=release-test event=push ref=refs/heads/devel` |
| Run lib tests profile | `profile=release-test event=push ref=refs/heads/devel` |
| Run integration tests profile | `profile=release-test event=push ref=refs/heads/devel` |
| cache restore | exact hit `Linux-cargo-6a1af67968af2b829f31637cb42371573b1fc279c0b7634dc63557a90d4227c2` |
| cache size | 1,637,296,893 B, 약 1561 MB |
| cache save | exact hit 상태라 skipped |
| cache 경고 | reservation/read-only/save failure/`##[error]` 없음 |
| 회귀 가드 | `Run integration tests` normalized test binary 180개, issue 계열 147개 |

`devel` push에서 의도한 정책 분기가 확인됐다. Build는 `release` smoke로 남고, Native Skia / lib /
integration test는 모두 `release-test`로 실행됐다.

`Dirty rhwp` 신호는 Build, Run lib tests, Run integration tests에서 관측됐다. 그러나 #1666 이후
full `release --tests` 경로에서 40분대였던 integration 비용은 `release-test` 경로에서 3m58s로 줄었다.
잔존 dirty/fingerprint 원인은 #1667에서 cache key, Cargo fingerprint, checkout timestamp, feature/test target
조합 관점으로 계속 다룬다.

## before / after 비교

### PR 경로

| 항목 | #1666 before PR | #1739 PR after | #1851 최종 PR | 판단 |
|------|-----------------|----------------|---------------|------|
| PR checks 완료 시간 | 약 19m23s | 약 20m25s | 12m09s | queue/update branch 영향이 큰 보조 지표 |
| `CI / Build & Test` job | 19m08s | 10m49s | 11m59s | #1666 개선 효과 유지 |
| Build | 3m33s | 1m30s | 1m30s | 유지 |
| Native Skia tests | 3m57s | 2m05s | 2m16s | 같은 범위 |
| Run lib tests | 3m46s | 1m38s | 1m47s | 같은 범위 |
| Run integration tests | 4m51s | 3m39s | 3m53s | 같은 범위 |

#1851 최종 PR run은 #1739 PR보다 `Build & Test`가 1m10s 길지만, 핵심 cargo step은 같은 범위다.
#1849 변경은 PR 경로를 악화시키지 않았고, `release-test` 전체 회귀 검증을 유지했다.

### `devel` push 경로

| 항목 | #1666 merge 후 direct | #1666 merge 후 P50 | #1666 merge 후 P90 | #1849 merge 후 | direct 대비 | P50 대비 | P90 대비 |
|------|-----------------------|--------------------|--------------------|----------------|-------------|----------|----------|
| `CI / Build & Test` job | 52m55s | 56m55s | 59m14s | 14m13s | -38m42s, -73.1% | -42m42s, -75.0% | -45m01s, -76.0% |
| Run integration tests | 39m23s | 42m27s | 44m14s | 3m58s | -35m25s, -89.9% | -38m29s, -90.7% | -40m16s, -91.0% |

`devel` push의 가장 큰 병목이던 full `release --tests` integration 비용이 제거됐다. 동시에 release profile 자체의
compile/link 파손은 `Build` step의 `cargo build --release --verbose` smoke로 계속 조기 감지한다.

## 공통 측정 기준 충족 여부

| 기준 | 결과 |
|------|------|
| PR checks 완료 시간 | #1851 최종 PR run 12m09s. 단일 after 표본이므로 #1849 PR P50/P90은 보류 |
| `CI / Build & Test` job 시간 | PR 11m59s, merge 후 `devel` 14m13s |
| 주요 step 시간 | Build / Native Skia / lib / integration / Clippy 기록 완료 |
| cache hit/miss/save | PR과 `devel` 모두 exact hit. PR save skipped, `devel` exact hit save skipped |
| cache 크기 | 1,637,296,893 B |
| 실패 시 원인 가시성 | 실패 없음. profile echo와 Cargo dirty line으로 원인 추적 가능 |
| runner-minutes 변화 | Build & Test wall time proxy 기준 `devel` P50 56m55s에서 14m13s로 감소 |
| branch protection / required check | `Build & Test` job 이름과 `CI / Build & Test` check 표면 변경 없음. workflow에서 required check 이름 변경 없음 |
| 회귀 가드 1:1 추적성 | `Run integration tests` normalized test binary 180개, issue 계열 147개 실행 |

## 해석

#1849는 목표를 달성했다.

- PR 경로는 #1666의 `release-test` 전체 회귀 검증 효과를 유지했다.
- `devel` push는 release build smoke와 `release-test` 전체 회귀 검증으로 분리됐다.
- #1666 이후 관측된 50분대 `devel` push 비용은 14m13s로 줄었다.
- integration 병목은 40분대에서 3m58s로 줄었다.
- cache restore/save 정책과 check 표면은 바뀌지 않았다.
- 회귀 가드 1:1 추적성은 유지됐다.

다만 `Dirty rhwp` 신호는 여전히 남는다. 이 문제는 #1849의 실패가 아니라 #1667의 대상이다. #1849는
dirty/fingerprint 원인을 제거한 것이 아니라, 해당 비용이 `devel` push full release integration에서 과도하게
증폭되지 않도록 profile 배치를 조정한 작업이다.

## 후속 연결

- #1667: exact cache hit 이후에도 남는 `Dirty rhwp` / local crate 재컴파일 원인을 cache/fingerprint 관점에서 분석
- #1665: #1849 이후에도 남는 `devel` push 14분대 wall time을 기준으로 job 병렬화 필요성을 재평가
- scheduled/nightly full `release --tests`: main/tag/manual full release 경로는 유지됐지만, release-only failure 탐지 주기를
  더 줄일 필요가 있으면 별도 정책 이슈로 검토

## 완료 판단

#1849는 완료로 판단한다.

코드 PR #1851은 merge됐고, 최종 PR run과 merge 후 `devel` push run 모두 성공했다. #1668 공통 측정 기준에
맞춘 before/after 보고를 이 문서와 이슈 코멘트에 남긴 뒤 #1849를 close한다.
