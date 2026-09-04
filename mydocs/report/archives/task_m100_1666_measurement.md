# Task M100 #1666 measurement 기록

## 목적

이 문서는 #1666 `[CI] PR용 Rust profile 재검토: --release vs release-test`의 before/after
측정 원천 기록이다.

부모 추적 문서 `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`에는 요약과 후속 판단만
반영하고, run별 raw 값과 해석은 이 문서를 기준으로 보존한다.

## 범위

- 코드 PR: #1739 `Task #1666: PR CI를 release-test profile 중심으로 전환`
- merge commit: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- merge 시각: 2026-07-01 22:58:02 KST
- 변경 파일: `.github/workflows/ci.yml`
- 변경하지 않은 파일: `Cargo.toml`, `tests/**`, `tests/golden_svg/**`

## 측정 기준

부모 이슈 #1668의 공통 측정 기준을 따른다.

- PR checks 완료 시간
- `CI / Build & Test` job 시간
- 주요 step 시간
  - Build
  - Native Skia tests
  - Run lib tests
  - Run integration tests
- cache hit/miss/save 성공 여부
- cache 크기
- 실패 시 원인 가시성
- runner-minutes 변화
- branch protection / required check 변경 여부
- 회귀 가드 1:1 추적성 보존 여부

## before 기준선

### PR 기준선: #1702

#1664 merge 전후 측정값 중 cache가 안정화된 PR run을 #1666의 PR before 기준선으로 사용한다.

- PR: #1702
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28430353568>
- head SHA: `69229e7937dc08fb94bf5d6530f205de77c15fe4`
- cache: exact hit, 약 1,547,590,748 B, save skipped
- 실패 / cache reservation / read-only / save 실패 경고: 없음
- P50/P90: 단일 표본이므로 산출 보류

| 항목 | before |
|------|--------|
| PR checks 완료 시간 | 약 19m23s |
| `CI / Build & Test` job | 19m08s |
| Build | 3m33s |
| Native Skia tests | 3m57s |
| Run lib tests | 3m46s |
| Run integration tests | 4m51s |
| Clippy | 21s |

### trusted branch 기준선

#1664 cleanup 후 exact-hit가 확인된 `devel` push run을 trusted branch before 기준선으로 사용한다.

- Run: <https://github.com/edwardkim/rhwp/actions/runs/28507949075>
- SHA: `150ca316ee557d6bf95928302166e037d7467b03`
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B, save skipped

| 항목 | before |
|------|--------|
| `CI / Build & Test` job | 18m02s |
| Build | 3m32s |
| Native Skia tests | 4m00s |
| Run lib tests | 3m50s |
| Run integration tests | 4m41s |
| Clippy | 25s |

## after 관측 1: #1739 PR run

- PR: #1739
- Run: <https://github.com/edwardkim/rhwp/actions/runs/28519297448>
- Build & Test job: <https://github.com/edwardkim/rhwp/actions/runs/28519297448/job/84541003903>
- head SHA: `8614c95b4a46f5500ff46362af8cbe16ac9bcddd`
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: PR run에서 skipped
- 실패 / cache reservation / read-only / save 실패 경고: 없음
- P50/P90: 단일 PR 표본이므로 산출 보류

| 항목 | before #1702 | after #1739 PR | 변화 |
|------|--------------|----------------|------|
| PR checks 완료 시간 | 약 19m23s | 약 20m25s | 원시 queue 포함값. update branch / 취소 run 영향으로 전후 비교 보조 지표 |
| `CI / Build & Test` job | 19m08s | 10m49s | -8m19s, 약 -43.5% |
| Build | 3m33s | 1m30s | -2m03s |
| Native Skia tests | 3m57s | 2m05s | -1m52s |
| Run lib tests | 3m46s | 1m38s | -2m08s |
| Run integration tests | 4m51s | 3m39s | -1m12s |
| Clippy | 21s | 25s | +4s |

해석:

- PR의 주요 cargo step이 `release-test` profile 중심으로 실행됐다.
- `release` profile의 LTO / codegen 비용이 PR 경로에서 제거되어 `Build & Test` wall time이 크게 줄었다.
- `Compiling rhwp`는 완전히 사라지지 않았다. profile 불일치 비용은 줄었지만, feature 조합, test target,
  Cargo dirty detection 때문에 local crate compile은 일부 남았다.
- `Run integration tests` 기준 normalized unique test binary는 165개, issue 계열은 132개가 실행됐다.
  이는 #1666 변경으로 테스트가 추가된 것이 아니라, PR head가 최신 `devel`의 신규 회귀 가드를 포함했기 때문이다.

## after 관측 2: merge 후 `devel` push run

### merge 직후 run

- Run: <https://github.com/edwardkim/rhwp/actions/runs/28523008914>
- Build & Test job: <https://github.com/edwardkim/rhwp/actions/runs/28523008914/job/84551911430>
- SHA: `1a7a8305d765830605a4ae8f9bbb99f61febb82c`
- 결론: 성공
- cache: exact hit `Linux-cargo-6a1af...`, 1,637,296,893 B
- save: skipped
- 실패 / cache reservation / save 실패 / `##[error]`: 없음

| 항목 | before trusted exact-hit | after merge 직후 | 변화 |
|------|--------------------------|------------------|------|
| `CI / Build & Test` job | 18m02s | 52m55s | +34m53s |
| Build | 3m32s | 3m25s | -7s |
| Native Skia tests | 4m00s | 3m54s | -6s |
| Run lib tests | 3m50s | 3m43s | -7s |
| Run integration tests | 4m41s | 39m23s | +34m42s |
| Clippy | 25s | 24s | -1s |

해석:

- `devel` push에서는 의도대로 `profile=release event=push`가 선택됐다.
- `Build`, `Native Skia tests`, `Run lib tests`는 before와 거의 같은 수준이다.
- 증가분은 거의 전부 `Run integration tests`에서 발생했다.
- merge 직후 로그에서 `Compiling rhwp`는 주요 cargo step 4곳에서 관측됐다.
  특히 integration step은 `Dirty rhwp ... dependency info changed` 이후 `Finished release ... in 38m10s`로
  끝나 전체 시간을 지배했다.
- `Run integration tests` 기준 normalized unique test binary는 165개, issue 계열은 132개가 실행됐다.
  raw `Running` 라인은 166개로, 보조/중복 binary 1개가 섞여 있다.

### merge 후 성공한 `devel` push 표본

#1739 merge 이후 생성된 `devel` push CI 중 완료된 성공 run만 집계했다. cancelled run과 in-progress run은
제외했다. paths-ignore로 애초에 생성되지 않은 push는 표본에 나타나지 않는다.

| run | SHA | Build & Test | Build | Native | Lib | Integration | Clippy | Restore | Save | 제목 |
|-----|-----|--------------|-------|--------|-----|-------------|--------|---------|------|------|
| <https://github.com/edwardkim/rhwp/actions/runs/28523008914> | `1a7a8305` | 52m55s | 3m25s | 3m54s | 3m43s | 39m23s | 24s | 19s | skipped | #1739 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28524069192> | `abc1ebb2` | 54m20s | 3m22s | 3m48s | 3m40s | 41m15s | 26s | 25s | skipped | #1741 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28583447597> | `f50aa4ef` | 57m15s | 3m39s | 4m10s | 3m59s | 43m07s | 27s | 27s | skipped | #1743 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28584895082> | `22092369` | 55m34s | 3m43s | 4m11s | 4m00s | 41m35s | 25s | 20s | skipped | #1656 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28586733799> | `4da379b4` | 58m38s | 3m37s | 4m02s | 3m54s | 44m01s | 29s | 28s | skipped | #1744 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28590887898> | `aa8f18fd` | 56m18s | 3m35s | 3m58s | 3m51s | 42m26s | 26s | 33s | skipped | #1802 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28596102299> | `716fbca9` | 56m55s | 3m41s | 4m13s | 3m56s | 42m27s | 25s | 19s | skipped | #1810 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28597839813> | `5aa0222b` | 55m30s | 3m31s | 4m03s | 3m51s | 42m03s | 25s | 19s | skipped | docs follow-up |
| <https://github.com/edwardkim/rhwp/actions/runs/28603206176> | `6a27dcd2` | 59m14s | 3m38s | 4m15s | 3m53s | 42m32s | 25s | 19s | skipped | #1758 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28605887389> | `c33b51ef` | 59m49s | 3m40s | 4m22s | 4m02s | 45m24s | 27s | 20s | skipped | #1814 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28610898350> | `a28c1b0f` | 55m15s | 3m29s | 3m58s | 3m47s | 42m02s | 25s | 18s | skipped | #1815 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28613218238> | `7acbc314` | 58m41s | 3m31s | 4m01s | 3m45s | 44m13s | 25s | 29s | skipped | #1817 merge |
| <https://github.com/edwardkim/rhwp/actions/runs/28618228488> | `9c25eb62` | 57m52s | 3m21s | 3m47s | 3m38s | 44m14s | 25s | 26s | skipped | #1806 merge |

### `devel` push P50/P90

| 항목 | n | min | P50 | P90 | max |
|------|---|-----|-----|-----|-----|
| `CI / Build & Test` job | 13 | 52m55s | 56m55s | 59m14s | 59m49s |
| Build | 13 | 3m21s | 3m35s | 3m41s | 3m43s |
| Native Skia tests | 13 | 3m47s | 4m02s | 4m15s | 4m22s |
| Run lib tests | 13 | 3m38s | 3m51s | 4m00s | 4m02s |
| Run integration tests | 13 | 39m23s | 42m27s | 44m14s | 45m24s |
| Clippy | 13 | 24s | 25s | 27s | 29s |

## runner-minutes 해석

GitHub Actions timing API의 public repository billable 값은 0으로 노출되므로, 이 문서에서는
`Build & Test` wall time을 runner-minutes proxy로 사용한다.

| 구간 | 기준 | after | 해석 |
|------|------|-------|------|
| PR `Build & Test` | 19m08s | 10m49s | 약 -8m19s, PR 피드백 루프 개선 |
| trusted `devel` merge 직후 | 18m02s | 52m55s | 약 +34m53s, release integration 검증 비용 증가 |
| trusted `devel` 누적 P50 | 18m02s | 56m55s | 약 +38m53s, integration step이 지배 |

## branch protection / required check 영향

- workflow job 이름 `Build & Test`는 유지됐다.
- check 표면 `CI / Build & Test`는 유지됐다.
- branch protection 설정 파일이나 GitHub required check 설정은 변경하지 않았다.
- 따라서 #1666 자체로 인한 required check 이름 변경은 없다.

## 최종 해석

#1666은 PR 경로에서는 목적을 달성했다. PR `Build & Test`는 19m08s에서 10m49s로 줄었고, `release`
profile의 LTO / codegen 비용을 PR 피드백 루프에서 제거했다.

다만 merge 후 `devel` push에서는 release-grade 검증을 trusted event로 이동한다는 결정이 실제 비용으로
드러났다. 누적 표본 13개에서 `Build & Test` P50은 56m55s, P90은 59m14s이며, 증가분은 거의 전부
`Run integration tests`의 `release` profile 실행에서 발생했다.

따라서 #1666의 평가는 다음처럼 분리한다.

- PR CI 단축: 성공
- 회귀 가드 1:1 추적성: 보존
- cache 정책: #1664 이후 상태 유지
- trusted branch release-grade 검증: 실행 확인
- trusted branch 비용: 크게 증가. 정책 판단 또는 후속 최적화 필요

후속 판단:

- #1667에서는 exact cache hit 이후에도 `Dirty rhwp` / `Compiling rhwp`가 남는 원인을 cache key,
  Cargo fingerprint, checkout timestamp, feature/test target 조합 관점에서 계속 본다.
- #1665에서는 job 병렬화 판단 시 PR run뿐 아니라 `devel` push의 50분대 integration 비용도 함께 고려한다.
- 별도 정책 판단이 필요하다면 `devel` push의 `Run integration tests`를 계속 `release`로 유지할지,
  `release-test`로 되돌리고 tag / release workflow에서만 full `release` integration을 수행할지 논의한다.
