# Task M100 #1664 Stage 3 완료 보고서

## 단계 목표

workflow 문법, 변경 범위, required check 표면, 회귀 가드 보존 여부를 최종 점검한다.

이 Stage 보고서는 문서 PR #1701에 포함되는 작업 기록이다. 실제 `.github/workflows/ci.yml` 변경은 후속 코드
PR #1702에만 포함되며, #1701 자체에는 workflow 변경이 없다. 아래 workflow 상태와 CI 관측은 #1702
draft 코드 PR 기준이다.

## 변경 범위 확인

후속 코드 PR #1702 workflow 변경:

- `.github/workflows/ci.yml`

문서 PR #1701 문서 변경:

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

`Cargo.toml`과 `tests/`에는 변경이 없다.

## 후속 코드 PR #1702 기준 workflow 상태

- restore:
  - step: `Restore cargo registry & build cache`
  - action: `actions/cache/restore@v5`
  - id: `cargo_cache_restore`
- save:
  - step: `Save cargo registry & build cache`
  - action: `actions/cache/save@v5`
  - 조건: `push` 이벤트의 `refs/heads/devel` 또는 `refs/heads/main`
  - exact cache hit 시 save 생략

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| `git diff --check` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | workflow 문법 오류 없음 |
| `git diff --name-only -- tests Cargo.toml .github/workflows/ci.yml` | 통과 | 대상 파일은 `.github/workflows/ci.yml`뿐 |
| `rg -n "name: Build & Test\|cargo test\|cargo build\|cargo clippy" .github/workflows/ci.yml` | 통과 | job 이름과 build/test/clippy command 유지 확인 |

## 메인테이너 결정사항 반영 확인

- PR cache restore-only: `pull_request`에서는 save 조건이 false다.
- trusted branch save: `push`의 `devel` / `main`에서만 save 조건이 true가 될 수 있다.
- `actions/cache` 기반 유지: third-party action을 도입하지 않았다.
- `Cargo.toml` profile 변경 없음: #1666 범위를 침범하지 않았다.
- job 병렬화 없음: #1665 범위를 침범하지 않았다.
- 회귀 가드 구조 변경 없음: `tests/`와 `tests/golden_svg/`를 수정하지 않았다.

## 정책 / 측정 원천 문서

메인테이너의 문서/코드 PR 분리 요청에 맞춰 하이퍼-워터폴 절차 문서와 별도로 다음 장기 문서를 추가했다.

- `mydocs/tech/ci_cache_policy_1664.md`
  - #1668 메인테이너 결정사항과 #1664 cache 정책의 원천 기록
  - non-goals, required check 표면, 운영 확인 기준 기록
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
  - #1668 하위 이슈 간 정책, 측정 기준, 진행 순서, 관측값 이관 판단 기록
  - #1664 관측값을 #1666 / #1667 변경 전 기준으로 연결하는 부모 추적 문서
- `mydocs/report/task_m100_1664_measurement.md`
  - PR run / trusted branch push run 측정 로그 템플릿
  - P50/P90 샘플 수 해석 기준과 로컬 정적 검증 기록

## CI 이후 측정 필요 항목

이 로컬 단계에서는 GitHub Actions run이 아직 없으므로 다음 값은 PR run과 `devel` push run 이후 기록해야 한다.

- PR checks 완료 시간 (P50, P90)
- `CI / Build & Test` job 시간
- 주요 step 시간: build / lib test / integration test / native-skia
- cache hit/miss/save 성공 여부
- cache 크기
- 실패 시 원인 가시성
- runner-minutes 변화
- branch protection / required check 변경 여부
- 회귀 가드 162개가 PR마다 모두 실행되는지 확인

후속 갱신:

- 후속/draft 코드 PR #1702 run 완료 후 PR run 측정값은 `mydocs/report/task_m100_1664_measurement.md`에 기록했다.
- #1664 관측값과 #1666 / #1667로 이관할 비교 기준은
  `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`에 기록했다.
- `devel` push run의 trusted branch save 동작은 #1702 병합 이후 추가 확인해야 한다.

## 결론

#1664의 workflow 코드 변경은 후속 코드 PR #1702에서 계획한 범위 안으로 준비됐다. 문서 PR #1701은 정책,
측정, 작업 기록만 포함하며, #1702가 merge되기 전에는 workflow 변경이 `devel`에 반영된 것으로 단정하지 않는다.
