# Task M100 #1668 CI 파이프라인 추적 기록

## 목적

이 문서는 #1668의 하위 이슈인 #1664, #1666, #1849, #1667, #1665 사이의 정책 결정, 측정 기준,
변경 전/후 기준선, PR 상태를 연결해 추적하는 장기 보관 문서다.

각 하위 이슈의 원천 측정값은 개별 측정 문서에 두고, 이 문서는 하위 이슈 간 해석,
의존성, 다음 진행 판단을 기록한다.

관련 문서:

- #1664 원천 측정 문서: `mydocs/report/task_m100_1664_measurement.md`
- #1664 cache 정책: `mydocs/tech/ci_cache_policy_1664.md`
- #1666 원천 측정 문서: `mydocs/report/task_m100_1666_measurement.md`
- #1849 수행 계획서: `mydocs/plans/task_m100_1849.md`
- #1849 최종 보고서: `mydocs/report/task_m100_1849_report.md`
- #1667 원천 측정 문서: `mydocs/report/task_m100_1667_measurement.md`

문서 PR #1701은 정책/측정 기록만 포함한다. 실제 CI workflow 변경은 후속 코드 PR #1702에서 다루며, #1702
관측값은 draft 코드 PR run 기준 비교 자료다. #1702가 merge되기 전에는 workflow 변경이 `devel`에 반영된
것으로 해석하지 않는다.

2026-07-01 후속 문서 PR에서는 #1702 merge 이후 cleanup, trusted branch save, 후속 exact-hit 결과를
부모 추적 문서에도 반영한다.

2026-07-03 후속 문서 PR에서는 #1739 merge 이후 #1666 PR 경로와 `devel` push 경로의 before/after
측정값을 반영한다.

2026-07-03 추가 보강 문서 PR에서는 #1702 merge 이후 #1739 merge 전까지의 순수 #1664 구간 표본을
재집계해 #1664 P50/P90을 보강한다. #1739 이후 run은 #1666 profile 전환 효과가 섞이므로 #1664
P50/P90에는 포함하지 않는다.

2026-07-03 후속 정책 검토에서는 #1666 measurement에서 드러난 `devel` push full `release --tests`
비용을 근거로 #1849를 새 sub-issue로 추가한다. #1849는 #1666 구현 오류 수정이 아니라, PR / `devel` /
`main` / tag / release workflow 사이의 profile 배치 정책을 재검토하는 작업이다.

2026-07-03 #1849 merge 후 관측에서는 `devel` push가 `release` build smoke + `release-test` 전체 회귀
검증으로 분리됐음을 확인했다. `Build & Test`는 #1666 merge 후 P50 56m55s에서 14m13s로 줄었고,
`Run integration tests`는 P50 42m27s에서 3m58s로 줄었다.

2026-07-03 #1667 1차 PR #1857 merge 후 관측에서는 CodeQL Rust cache가 PR restore-only / trusted branch
save-only 구조로 정렬됐음을 확인했다. PR #1857에서는 `refs/pull/1857/merge` 신규 cache가 생성되지 않았고,
merge 후 `devel` push에서는 CodeQL Rust exact-hit restore 후 save skipped, cache reservation/read-only/save
failure 경고 없음이 확인됐다.

## 메인테이너 결정사항

### 회귀 가드 1:1 추적성 보존

CI 단축은 회귀 가드 구조를 보존하면서 프로필, 캐시, 병렬화 축에서만 추진한다.

수용 불가 항목:

- 통합 테스트 파일 통합
- 회귀 가드 명명 규칙 변경
- `tests/golden_svg/issue-NNN/` 자산 구조 변경

`tests/*.rs` 162개와 issue 계열 1:1 회귀 가드 131개의 추적성이 PR마다 유지되는지 확인한다.

### 정책 판단 4건 답변

| 항목 | 메인테이너 결정 | 적용 이슈 |
|------|----------------|----------|
| PR 릴리스 검증 정책 | PR은 `release-test` 중심, 릴리스 수준 검증은 `devel` push / tag workflow로 이동 | #1666 |
| 캐시 저장 정책 | PR은 복원 전용, `devel` / `main` push에서만 저장 | #1664 |
| job 병렬화 | #1666 적용 후 효과를 재평가한 뒤 결정 | #1665 |
| Rust 캐시 액션 | `Swatinem/rust-cache` 검토 허용. 단 SHA-pinned, 별도 PR, #1664 안정화 측정 후 판단 | #1667 |

### 문서 / 코드 PR 분리

메인테이너 요청에 따라 PR 단위는 다음처럼 분리한다.

- `mydocs/`: 정책 / 의사결정 / 측정 기록
- `.github/workflows/ci.yml`: 실제 CI 변경
- `Cargo.toml` profile 변경: 필요 시 단독 PR

## 확정 진행 순서

| 순서 | 이슈 | 범위 | 현재 판단 |
|------|------|------|-----------|
| 1 | #1664 | 캐시 정책 정리 | 코드 PR #1702 merge 완료. PR save 차단, cleanup 후 trusted branch save, 후속 exact-hit 확인 완료. 순수 #1664 구간 P50/P90 보강 완료 |
| 2 | #1666 | PR `release-test` 프로필 전환 | 코드 PR #1739 merge 완료. PR `Build & Test`는 10m49s로 개선, merge 후 `devel` push는 release integration 비용으로 50분대 확인 |
| 3 | #1849 | `devel` push profile 배치 재조정 | 코드 PR #1851 merge 완료. `devel` push는 release build smoke + `release-test` 전체 회귀로 전환됐고, `Build & Test`는 14m13s로 감소 |
| 4 | #1667 | Rust 캐시 전략 검토 | 1차 CodeQL Rust restore/save 분리 PR #1857 merge 완료. PR cache save 표면 제거 확인. exact-hit 이후 남는 `Dirty rhwp` / `Compiling rhwp`, Render Diff cargo cache, stale PR ref cleanup은 후속 판단 |
| 5 | #1665 | Build & Test job 병렬 분리 | #1849에서 `devel` push profile 정책을 먼저 확정한 뒤 병렬화 효과와 runner-minutes를 재산정 |

## 공통 측정 기준

각 하위 이슈의 before/after 보고에는 다음 항목을 포함한다.

- PR checks 완료 시간 (P50, P90)
- `CI / Build & Test` job 시간
- 주요 step 시간
  - Build
  - Native Skia tests
  - Run lib tests
  - Run integration tests
- 캐시 hit/miss/save 성공 여부
- 캐시 크기
- 실패 시 원인 가시성
- runner-minutes 변화
- branch protection / required check 변경 여부
- 회귀 가드 162개가 PR마다 모두 실행되는지 확인

샘플 수가 1개뿐인 경우 P50/P90은 산출하지 않고 단일 관측값으로만 기록한다.

## #1664 관측 요약

### 후속/draft 코드 PR #1702 변경 후 표본

- PR: #1702
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28430353568>
- Build & Test job: <https://github.com/edwardkim/rhwp/actions/runs/28430353568/job/84243307175>
- head SHA: `69229e7937dc08fb94bf5d6530f205de77c15fe4`
- 결론: 성공
- 해석 범위: #1702 draft 코드 PR 기준 관측값이며, #1702 merge 전에는 `devel` 반영 사실로 단정하지 않는다.

핵심 관측:

- `Restore cargo registry & build cache`: 정확히 적중
- 캐시 크기: 약 1476 MB
- `Save cargo registry & build cache`: PR에서 skipped
- 캐시 예약 / 읽기 전용 / 저장 실패 경고: 관측되지 않음
- `Build`, `Native Skia tests`, `Run lib tests`, `Run integration tests`, `Clippy` 모두 실행
- `Run integration tests` 기준 issue 계열 131/131 실행 확인

시간:

| 항목 | 시간 |
|------|------|
| PR checks 완료 시간 | 약 19m23s |
| `CI / Build & Test` job | 19m08s |
| Build | 3m33s |
| Native Skia tests | 3m57s |
| Run lib tests | 3m46s |
| Run integration tests | 4m51s |
| Check WASM target | 15s |
| Install native Skia runtime packages | 18s |
| Clippy | 21s |

### #1702 merge 후 trusted branch 표본

- #1664 상세 보고: <https://github.com/edwardkim/rhwp/issues/1664#issuecomment-4853263686>
- #1668 롤업: <https://github.com/edwardkim/rhwp/issues/1668#issuecomment-4853268643>

cleanup:

- closed/merged `refs/pull/*` cache 총 21개 삭제
- `refs/heads/devel`, `refs/heads/main`, OPEN PR cache 유지
- 최종 cache 총량: 7,154,189,707 B, 약 6.66 GiB / 7.15 GB
- 10GB budget 아래로 회복

trusted branch run:

| 구분 | run | SHA | Build & Test | restore | save | read-only 경고 |
|------|-----|-----|--------------|---------|------|----------------|
| cleanup 직후 rerun | <https://github.com/edwardkim/rhwp/actions/runs/28505355210/attempts/2> | `5e3b1ec652fda14a74af7cf9afd77962e3bb7903` | 22m53s | fallback `Linux-cargo-`, 426,792,350 B | success, 1,637,296,893 B | 없음 |
| 후속 devel run | <https://github.com/edwardkim/rhwp/actions/runs/28507949075> | `150ca316ee557d6bf95928302166e037d7467b03` | 18m02s | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B | skipped | 없음 |

주요 step 시간:

| run | Build | Native Skia tests | Run lib tests | Run integration tests | Clippy |
|-----|-------|-------------------|---------------|-----------------------|--------|
| `28505355210` attempt 2 | 4m43s | 4m55s | 3m44s | 6m23s | 46s |
| `28507949075` | 3m32s | 4m00s | 3m50s | 4m41s | 25s |

해석:

- #1664의 핵심 정책인 “PR에서는 save 차단, trusted branch에서 save 허용”이 end-to-end로 확인됐다.
- cleanup 전 cache budget 초과 상태가 trusted branch save를 막고 있었고, stale PR ref cache cleanup 후 새 exact
  cargo cache key 저장이 가능해졌다.
- 후속 `devel` run에서 exact-hit restore와 save skipped가 확인되어, save 조건도 기대대로 동작했다.
- 이 관측은 #1666의 profile 전환 전 기준선과 #1667의 cache 전략 재평가 기준선으로 사용한다.

### #1702 merge 후 #1739 merge 전 순수 #1664 구간 P50/P90 보강

구간:

- 시작: #1702 merge, 2026-07-01 14:36:10 KST
- 종료: #1739 merge 직전, 2026-07-01 22:58:02 KST
- 제외: #1739 이후 run. #1666 profile 전환 효과가 섞이므로 #1664 cache 정책 분포에 포함하지 않음

PR run:

| 항목 | n | P50 | P90 | 비고 |
|------|---|-----|-----|------|
| PR checks 완료 시간 | 20 | 23m25s | 26m50s | `Build & Test` 실행 PR success 기준. #1739/#1666 profile 변경 run 제외 |
| `CI / Build & Test` job | 20 | 22m34s | 23m46s | #1664 PR cache 정책 구간 |
| Build | 20 | 4m45s | 4m58s | #1664 PR cache 정책 구간 |
| Native Skia tests | 20 | 5m04s | 5m18s | #1664 PR cache 정책 구간 |
| Run lib tests | 20 | 3m51s | 4m01s | #1664 PR cache 정책 구간 |
| Run integration tests | 20 | 6m12s | 6m31s | #1664 PR cache 정책 구간 |

PR cache save:

- `Build & Test` 실행 PR success run 21개에서 save는 21/21 skipped.
- fast-pass로 `Build & Test`가 skipped 된 PR success run은 10개.
- #1739/#1666 PR run은 PR save skipped 검증에는 참고 가능하지만, profile/step-time P50/P90에는 포함하지 않는다.

trusted `devel` push run:

| 항목 | n | P50 | P90 | 비고 |
|------|---|-----|-----|------|
| `CI / Build & Test` job | 12 | 18m21s | 23m22s | #1702 merge 후 #1739 merge 전 `devel` push success |
| Build | 12 | 3m32s | 4m49s | #1666 적용 전 기준선 |
| Native Skia tests | 12 | 4m00s | 5m03s | #1666 적용 전 기준선 |
| Run lib tests | 12 | 3m45s | 4m01s | #1666 적용 전 기준선 |
| Run integration tests | 12 | 4m49s | 6m23s | #1666 적용 전 기준선 |
| Clippy | 12 | 26s | 46s | #1666 적용 전 기준선 |

trusted branch cache save:

- trusted `devel` push success run 12개 중 save success 4개, save skipped 8개.
- 대표 saved/exact key는 `Linux-cargo-6a1af...`, 대표 cache 크기는 1,637,296,893 B.
- 초기 trusted branch run에서 save가 수행되고, 이후 exact hit 상태에서는 save skipped로 안정화된 것으로 해석한다.

판단:

- #1664의 PR save 차단과 trusted branch save 허용 정책은 누적 표본에서도 유지됐다.
- #1664 순수 구간의 trusted branch `Build & Test` P50은 18m21s, P90은 23m22s다.
- #1666 이후 `devel` push가 50분대로 증가한 현상은 #1664 cache 정책 효과가 아니라 #1666 release
  integration 정책 효과로 분리한다.

### #1666 / #1667로 이관할 관측

캐시 복원은 성공했지만 PR merge ref와 프로필/feature 조합 때문에 최종 `rhwp` crate compile은 계속 발생했다.

관측된 compile:

- Build: `Compiling rhwp`, `release` 프로필 3m32s
- Native Skia tests: `Compiling rhwp`, `release` 프로필 3m56s
- Run lib tests: `Compiling rhwp`, `release` 프로필 3m34s
- Run integration tests: `Compiling rhwp`, `release-test` 프로필 3m32s

해석:

- 이는 #1664 실패가 아니다. #1664의 목표는 PR cache save 차단과 trusted branch save 정책 정착이다.
- `native-skia` feature, `release`, `release-test`, PR merge ref 조합 때문에 local crate 산출물이 별도로 컴파일될 수 있다.
- 이 값은 #1666 `release-test` 프로필 전환과 #1667 Rust 캐시 전략 비교의 변경 전 기준으로 사용한다.

## #1666 관측 요약

원천 측정 문서: `mydocs/report/task_m100_1666_measurement.md`

### #1739 PR run

- PR: #1739
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28519297448>
- head SHA: `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: PR에서 skipped
- 실패 / cache reservation / read-only / save 실패 경고: 없음

| 항목 | #1666 before PR | #1739 PR after | 변화 |
|------|-----------------|----------------|------|
| PR checks 완료 시간 | 약 19m23s | 약 20m25s | 원시 queue 포함값. update branch / 취소 run 영향으로 보조 지표 |
| `CI / Build & Test` job | 19m08s | 10m49s | -8m19s, 약 -43.5% |
| Build | 3m33s | 1m30s | -2m03s |
| Native Skia tests | 3m57s | 2m05s | -1m52s |
| Run lib tests | 3m46s | 1m38s | -2m08s |
| Run integration tests | 4m51s | 3m39s | -1m12s |
| Clippy | 21s | 25s | +4s |

판단:

- PR `Build & Test`는 `release-test` 전환으로 개선됐다.
- `Compiling rhwp`는 일부 남았으나, PR의 `release` LTO / codegen 비용 제거 효과가 확인됐다.
- 회귀 가드는 PR head 기준 normalized unique 165개, issue 계열 132개가 실행됐다. 이는 최신 `devel`의 신규
  회귀 가드 포함 영향이며 #1666 변경으로 테스트 구조가 바뀐 것은 아니다.

### #1739 merge 후 `devel` push

- merge commit: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- merge 직후 run: <https://github.com/edwardkim/rhwp/actions/runs/28523008914>
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: skipped
- 실패 / cache reservation / save 실패 / `##[error]`: 없음

| 항목 | #1666 before trusted exact-hit | #1739 merge 직후 | 변화 |
|------|--------------------------------|------------------|------|
| `CI / Build & Test` job | 18m02s | 52m55s | +34m53s |
| Build | 3m32s | 3m25s | -7s |
| Native Skia tests | 4m00s | 3m54s | -6s |
| Run lib tests | 3m50s | 3m43s | -7s |
| Run integration tests | 4m41s | 39m23s | +34m42s |
| Clippy | 25s | 24s | -1s |

#1739 merge 이후 성공한 `devel` push 표본 13개의 P50/P90:

| 항목 | n | P50 | P90 |
|------|---|-----|-----|
| `CI / Build & Test` job | 13 | 56m55s | 59m14s |
| Build | 13 | 3m35s | 3m41s |
| Native Skia tests | 13 | 4m02s | 4m15s |
| Run lib tests | 13 | 3m51s | 4m00s |
| Run integration tests | 13 | 42m27s | 44m14s |
| Clippy | 13 | 25s | 27s |

판단:

- PR CI 단축은 성공했다.
- trusted `devel` push에서는 메인테이너 결정대로 release-grade 검증이 실행됐다.
- 다만 release integration step이 40분대가 되어 `Build & Test` 전체가 50분대가 됐다.
- 이는 구현 오류라기보다 "release-grade 검증을 trusted event로 이동"한 정책의 실제 비용이다.
- 후속으로 `devel` push integration을 계속 `release`로 유지할지, `release-test`로 되돌리고 tag / release workflow에서
  full release integration을 수행할지 정책 판단이 필요할 수 있다.

## branch protection / required check 영향

#1664 PR #1702, #1666 PR #1739, #1849 PR #1851 merge 후 관측 기준:

- `Build & Test` job 이름 유지
- `CI / Build & Test` check 표면 유지
- branch protection / required check 변경 없음
- job 병렬화 없음
- #1666 이후 runner-minutes proxy인 `Build & Test` wall time은 PR에서 감소, trusted `devel` push에서 증가
- #1849 이후 runner-minutes proxy인 `Build & Test` wall time은 trusted `devel` push에서 다시 감소

## #1849 관측 요약

원천 최종 보고서: `mydocs/report/task_m100_1849_report.md`

### #1851 최종 PR run

- PR: #1851
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28648923176>
- head SHA: `5c2afe845ec4fe4afb0eacba49ffb33f22691fc2`
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: PR에서 skipped
- 실패 / cache reservation / read-only / save 실패 경고: 없음

| 항목 | 시간 |
|------|------|
| PR checks 완료 시간 | 12m09s |
| `CI / Build & Test` job | 11m59s |
| Build | 1m30s |
| Native Skia tests | 2m16s |
| Run lib tests | 1m47s |
| Run integration tests | 3m53s |
| Clippy | 26s |

판단:

- PR 경로는 #1666의 `release-test` 전체 회귀 검증을 유지했다.
- 회귀 가드는 normalized test binary 180개, issue 계열 147개가 실행됐다.
- 단일 after 표본이므로 #1849 PR P50/P90은 산출하지 않는다.

### #1851 merge 후 `devel` push

- merge commit: `d76f1997f07cc14f06d80d18cbec1d1b36b0839c`
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28649575142>
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: exact hit 상태라 skipped
- 실패 / cache reservation / read-only / save 실패 경고: 없음

| 항목 | #1666 merge 후 P50 | #1666 merge 후 P90 | #1849 merge 후 | 판단 |
|------|--------------------|--------------------|----------------|------|
| `CI / Build & Test` job | 56m55s | 59m14s | 14m13s | P50 대비 -42m42s, -75.0% |
| Build | 3m35s | 3m41s | 3m38s | release smoke 유지 |
| Native Skia tests | 4m02s | 4m15s | 2m13s | `release-test` 전환 |
| Run lib tests | 3m51s | 4m00s | 1m48s | `release-test` 전환 |
| Run integration tests | 42m27s | 44m14s | 3m58s | P50 대비 -38m29s, -90.7% |
| Clippy | 25s | 27s | 26s | 동일 수준 |

판단:

- `devel` push에서 Build는 `release` smoke로 남고, Native Skia / lib / integration test는 `release-test`로 실행됐다.
- #1666 이후 가장 큰 비용이던 full `release --tests` integration이 `devel` push에서 제거됐다.
- `Dirty rhwp` 신호는 남지만, 비용이 full release integration에서 40분대로 증폭되지 않는다.
- 잔존 dirty/fingerprint 원인은 #1667에서 계속 다룬다.

## #1667 관측 요약

원천 측정 문서: `mydocs/report/task_m100_1667_measurement.md`

### #1857 PR run

- PR: #1857
- head SHA: `30a3acaaa01aedbe302cc7762e302875621b8d36`
- CodeQL run: <https://github.com/edwardkim/rhwp/actions/runs/28652708143>
- CI run: <https://github.com/edwardkim/rhwp/actions/runs/28652708175>
- 결론: 성공
- PR checks 완료 시간: 약 12m28s
- P50/P90: 단일 표본이므로 산출 보류

CodeQL Rust:

| 항목 | 값 |
|------|----|
| `Analyze (rust)` job | 8m18s |
| restore | exact hit `Linux-codeql-rust-6a1af...`, 529,492,545 B |
| `Restore cargo registry & build cache (rust)` | 8s |
| `Build Rust (for CodeQL)` | 39s |
| `Perform CodeQL Analysis` | 6m55s |
| save | skipped |
| cache reservation / read-only / save failure 경고 | 없음 |
| `refs/pull/1857/merge` 신규 cache | 0개 |

Build & Test 참고값:

| 항목 | 값 |
|------|----|
| `CI / Build & Test` job | 12m12s |
| Build | 1m33s |
| Native Skia tests | 2m18s |
| Run lib tests | 1m51s |
| Run integration tests | 3m53s |
| Clippy | 25s |

판단:

- PR run에서 CodeQL Rust cache save step이 skipped 되어 PR restore-only 정책 결과가 확인됐다.
- #1857은 Build & Test workflow를 변경하지 않았으므로 Build & Test 값은 회귀 확인용 참고값이다.

### #1857 merge 후 `devel` push

- merge commit: `aebde2d22948cf5ab712d226fb4b23b3f341e21b`
- CodeQL run: <https://github.com/edwardkim/rhwp/actions/runs/28653978487>
- CI run: <https://github.com/edwardkim/rhwp/actions/runs/28653978510>
- 결론: 성공
- CodeQL run 완료 시간: 8m31s
- CI run 완료 시간: 14m19s

CodeQL Rust:

| 항목 | 값 |
|------|----|
| `Analyze (rust)` job | 8m17s |
| restore | exact hit `Linux-codeql-rust-6a1af...`, 529,492,545 B |
| `Restore cargo registry & build cache (rust)` | 11s |
| `Build Rust (for CodeQL)` | 37s, cargo 내부 `dev` profile 25.76s |
| `Perform CodeQL Analysis` | 6m50s |
| save | skipped |
| cache reservation / read-only / save failure 경고 | 없음 |

Build & Test 참고값:

| 항목 | 값 |
|------|----|
| `CI / Build & Test` job | 14m08s |
| restore | exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B |
| save | skipped |
| Build | 3m38s |
| Native Skia tests | 2m15s |
| Run lib tests | 1m52s |
| Run integration tests | 3m57s |
| Clippy | 26s |

판단:

- trusted branch push에서도 exact hit이면 save skipped 되는 조건이 확인됐다.
- 이번 run은 exact hit였기 때문에 trusted branch save success 경로는 새로 실행되지 않았다.
- CodeQL Rust와 Build & Test 일부 step에서 `Compiling rhwp`는 남는다. Build release smoke와 Native Skia
  feature 조합은 현재 정책상 예상 가능한 별도 산출물이고, Run lib tests / Run integration tests의
  `Dirty rhwp`는 cache fingerprint, checkout timestamp, test target 산출물 관점의 후속 분석 대상이다.
- branch protection / required check 변경은 없고, `devel` required status check context는 `Build & Test` 그대로다.

## P50/P90 상태

| 구간 | 대상 | 샘플 수 | 판단 |
|------|------|---------|------|
| #1664 초기 단일 관측 | PR checks 완료 시간 | 1 | P50/P90 산출 보류 |
| #1664 초기 단일 관측 | `CI / Build & Test` job 시간 | 1 | P50/P90 산출 보류 |
| #1664 초기 trusted branch 관측 | trusted branch `Build & Test` job 시간 | 2 | P50/P90 산출 보류. 22m53s / 18m02s 관측 |
| #1664 merge 후 순수 구간 | PR checks 완료 시간 | 20 | P50 23m25s, P90 26m50s |
| #1664 merge 후 순수 구간 | PR `CI / Build & Test` job 시간 | 20 | P50 22m34s, P90 23m46s |
| #1664 merge 후 순수 구간 | trusted branch `Build & Test` job 시간 | 12 | P50 18m21s, P90 23m22s |
| #1666 변경 전 | PR #1702 관측값 | 1 | `release-test` 전환 전 비교 기준으로 보존 |
| #1666 변경 후 | PR #1739 관측값 | 1 | PR P50/P90 산출 보류. `Build & Test` 10m49s |
| #1666 merge 후 | trusted branch `Build & Test` job 시간 | 13 | P50 56m55s, P90 59m14s |
| #1666 merge 후 | trusted branch `Run integration tests` 시간 | 13 | P50 42m27s, P90 44m14s |
| #1849 변경 후 | PR #1851 최종 관측값 | 1 | PR P50/P90 산출 보류. `Build & Test` 11m59s |
| #1849 merge 후 | trusted branch `Build & Test` job 시간 | 1 | 단일 관측값 14m13s. #1666 P50 56m55s 대비 감소 |
| #1849 merge 후 | trusted branch `Run integration tests` 시간 | 1 | 단일 관측값 3m58s. #1666 P50 42m27s 대비 감소 |
| #1667 변경 후 | PR #1857 관측값 | 1 | PR P50/P90 산출 보류. CodeQL `Analyze (rust)` 8m18s, PR cache save skipped |
| #1667 merge 후 | trusted branch CodeQL `Analyze (rust)` job 시간 | 1 | 단일 관측값 8m17s. exact hit라 save skipped |
| #1667 merge 후 | trusted branch `Build & Test` job 시간 | 1 | 참고값 14m08s. #1857은 Build & Test workflow 변경 없음 |

## 다음 확인 항목

1. #1667에서는 exact cache hit 이후에도 `Dirty rhwp` / `Compiling rhwp`가 남는 원인을 Build & Test cargo cache와
   CodeQL Rust analyze cache 범위를 분리해 판단한다.
2. Render Diff cargo cache는 PR ref 누적 규모가 크므로 #1667 후속 PR에서 정책을 별도로 결정한다.
3. #1665에서는 #1849 이후에도 남는 `devel` push wall time과 runner-minutes를 병렬화 효과 산정의 주요 입력으로 사용한다.
4. OPEN PR cache는 계속 생성될 수 있으므로, cleanup은 closed/merged PR ref만 대상으로 유지한다.
