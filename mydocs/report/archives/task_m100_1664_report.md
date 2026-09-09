# Task M100 #1664 최종 결과 보고서

## 개요

- 이슈: #1664
- 부모 이슈: #1668
- 브랜치: `local/task1664`
- 주제: Actions cache quota/read-only 상태 해소 및 PR cache 저장 정책 정리
- 문서 PR: #1701 (`mydocs/**` 정책/측정 기록 전용)
- 코드 PR: #1702 (`.github/workflows/ci.yml` workflow 변경, merge 완료)
- 후속 문서 PR: #1702 merge 이후 최종 measurement를 `mydocs/`에 보존

## PR 분리 기준

문서 PR #1701은 정책, 의사결정, 측정 기록만 포함한다. 실제 `.github/workflows/ci.yml` 변경은 후속 코드 PR
#1702에서만 다루며, #1701이 merge되어도 workflow 변경이 `devel`에 반영된 것은 아니다.

## 코드 PR #1702 기준 변경 요약

코드 PR #1702에서는 `.github/workflows/ci.yml`의 `Build & Test` cargo cache를 restore/save 분리 구조로
변경했다.

- 기존: `actions/cache@v5` 단일 step
- 변경:
  - `actions/cache/restore@v5`로 모든 run에서 restore
  - `actions/cache/save@v5`로 `devel` / `main` push에서만 save
  - exact cache hit이면 save 생략

정책/측정 원천 문서는 문서 PR #1701에 별도로 추가했다.

- `mydocs/tech/ci_cache_policy_1664.md`: 정책과 의사결정 기록
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`: 부모 이슈 #1668 기준 하위 이슈 간 추적 기록
- `mydocs/report/task_m100_1664_measurement.md`: PR/devel run 누적 측정 로그

2026-07-01 후속 문서 PR에서는 #1702 merge 이후 최종 관측값을 위 원천 문서에 반영했다.

## 정책 판단 4건 반영

1. PR `--release` + LTO 검증 유지 여부
   - #1664에서는 profile을 변경하지 않았다.
   - PR profile 전환은 #1666 범위로 분리했다.

2. PR cache restore-only, `devel` / `main`만 save
   - 후속 코드 PR #1702의 핵심 변경으로 반영했다.
   - #1702 기준 `pull_request`, tag, `workflow_dispatch`에서는 cargo cache save가 실행되지 않는다.

3. `Build & Test` job 병렬 분리
   - job 구조를 바꾸지 않았다.
   - 병렬화는 #1666 이후 재평가하는 #1665 범위로 유지했다.

4. `Swatinem/rust-cache` 검토
   - 도입하지 않았다.
   - 현행 `actions/cache` 기반 정책 정리 후 안정화 측정을 선행한다.

## PR #1701 변경 파일

- `mydocs/orders/20260630.md`
- `mydocs/plans/task_m100_1664.md`
- `mydocs/plans/task_m100_1664_impl.md`
- `mydocs/working/task_m100_1664_stage1.md`
- `mydocs/working/task_m100_1664_stage2.md`
- `mydocs/working/task_m100_1664_stage3.md`
- `mydocs/tech/ci_cache_policy_1664.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
- `mydocs/report/task_m100_1664_measurement.md`
- `mydocs/report/task_m100_1664_report.md`

코드 PR #1702 변경 대상:

- `.github/workflows/ci.yml`

## 보존한 항목

- `Build & Test` job 이름
- build/test/clippy command
- cargo cache key와 restore key
- profile 정책
- job 구조
- `Cargo.toml`
- 통합 테스트 파일 구조
- 회귀 가드 명명 규칙
- `tests/golden_svg/issue-NNN/` 자산 구조

## 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| 문서 PR #1701 `git diff --check` | 통과 | `mydocs/**` whitespace 문제 없음 |
| 후속 코드 PR #1702 `git diff --check` | 통과 | workflow 변경 기준 whitespace 문제 없음 |
| 후속 코드 PR #1702 `actionlint .github/workflows/ci.yml` | 통과 | workflow 문법 오류 없음 |
| 문서 PR #1701 변경 범위 확인 | 통과 | `mydocs/**` 문서 전용 |
| 후속 코드 PR #1702 변경 범위 확인 | 통과 | `Cargo.toml`, `tests/` 변경 없음 |
| required check 표면 | 통과 | #1702 기준 `Build & Test` job 이름 유지 |
| 회귀 가드 구조 | 통과 | #1702 기준 테스트 파일/자산 구조 변경 없음 |

## before/after 측정 기준

메인테이너 요청 기준으로 다음 항목은 PR run과 `devel` push run 이후
`mydocs/report/task_m100_1664_measurement.md`에 누적 기록한다.

| 항목 | 현재 상태 |
|------|-----------|
| PR checks 완료 시간 (P50, P90) | 후속/draft 코드 PR #1702 단일 관측값 기록 완료. P50/P90은 표본 부족으로 보류 |
| `CI / Build & Test` job 시간 | 후속/draft 코드 PR #1702 19m08s 기록 완료 |
| 주요 step 시간 | 후속/draft 코드 PR #1702 build / lib test / integration test / native-skia 기록 완료 |
| cache hit/miss/save 성공 여부 | PR run save skipped, cleanup 후 `devel` save success, 후속 `devel` exact-hit save skipped 확인 |
| cache 크기 | PR run 약 1476 MB, post-merge cargo exact cache 1,637,296,893 B 기록 |
| 실패 시 원인 가시성 | `Cache reservation failed`, `Failed to save`, `##[error]` 재현 없음 |
| runner-minutes 변화 | PR 표본 1개와 trusted branch 표본 2개뿐이므로 장기 증감 판단 보류 |
| branch protection / required check 변경 여부 | #1702 기준 job 이름 유지, GitHub 설정 변경 없음 |
| 회귀 가드 162개 PR 실행 여부 | 후속/draft 코드 PR #1702에서 issue 계열 131/131 실행 확인 |

## 리스크

- cache save가 trusted branch로 제한되어 PR별 두 번째 run cache 개선폭은 줄어들 수 있다.
- 기존 cache quota 자체가 부족하면 cleanup 또는 budget 조정이 추가로 필요할 수 있다. 2026-07-01에는
  closed/merged PR ref cache cleanup 후 trusted branch save가 성공했다.
- exact key hit가 계속 발생하면 save step은 생략되므로, 새 key 생성 여부는 `Cargo.lock` 변경 또는 cache miss 상황에서 확인해야 한다.

## 후속

- 후속/draft 코드 PR #1702에서 PR save step이 skipped 되는지 확인했다.
- #1702 merge 후 `devel` push에서 save step이 조건부 실행되는지 확인했다.
- 후속/draft 코드 PR #1702와 #1702 merge 후 `devel` run에서 cache read-only 경고가 관측되지 않았다.
- 안정화 측정 결과는 #1666 profile 전환 전 기준선과 #1667 cache 전략 재평가 기준선으로 사용한다.
