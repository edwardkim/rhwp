# Task M100 #1667 v2 구현 계획서

## 개요

- 이슈: #1667 `[CI] Rust cache 전략 개선: actions/cache 유지 vs Swatinem/rust-cache 검토`
- 부모 이슈: #1668 `[Tracking/RFC] CI Build & Test 실행 시간 단계별 단축`
- 선행 계획서: `mydocs/plans/task_m100_1667_v2.md`
- 작업 브랜치: `task-1667-render-diff-cache`
- 구현 대상: `.github/workflows/render-diff.yml`

## 구현 판단

#1861 merge 후 Render Diff before 표본을 수집한 결과, 최신 full `Canvas visual diff` 20개는 cargo/npm cache를
모두 복원하지 못했다.

핵심 관측:

- `Canvas visual diff` job P50/P90: 3m47s / 3m57s
- `Build WASM package` P50/P90: 1m15s / 1m18s
- `Build native CLI for PDF report` P50/P90: 1m04s / 1m09s
- `Install Studio dependencies` P50/P90: 6s / 7s
- cargo restore: 20/20 miss
- cargo save: 20/20 reservation/read-only 계열 실패
- npm restore: 20/20 miss
- npm save: 20/20 reservation failure
- `Linux-render-diff-cargo-*` 9개와 `node-cache-*` 9개는 모두 merged PR ref cache

따라서 이번 구현은 "캐시로 시간을 줄이기"보다 "PR ref cache save 표면을 제거하고 실패 로그를 줄이기"를
우선한다. PR run은 restore-only로 만들고, 필요할 때만 trusted manual seed를 만들 수 있도록
`workflow_dispatch` on `devel/main`에서만 cargo cache save를 허용한다.

## 선택한 후보

수행 계획서의 후보 B/C/E를 절충한다.

| 항목 | 선택 |
|------|------|
| PR run | restore-only |
| trusted seed | `workflow_dispatch` on `devel/main`에서만 허용 |
| cargo path | 현행 `~/.cargo/registry`, `~/.cargo/git`, `target` 유지 |
| npm cache | 제거 |
| push trigger | 추가하지 않음 |
| `Swatinem/rust-cache` | 도입하지 않음 |

`target` path를 이번에 제거하지 않는 이유는, after에서 trusted manual seed hit 가능성을 먼저 확인하기 위해서다.
반대로 npm cache는 최신 표본에서 항상 miss였고 `npm ci` 자체가 6-7초라 유지할 실익이 낮다.

## 구현 내용

### 1. Render Diff cargo cache restore/save 분리

기존 단일 step:

```yaml
- name: Cache cargo registry & build
  uses: actions/cache@v5
```

변경 후:

```yaml
- name: Restore cargo registry & build cache
  id: render_diff_cargo_cache_restore
  uses: actions/cache/restore@v5
```

save step:

```yaml
- name: Save cargo registry & build cache
  if: ${{ github.event_name == 'workflow_dispatch' && (github.ref == 'refs/heads/devel' || github.ref == 'refs/heads/main') && steps.render_diff_cargo_cache_restore.outputs.cache-hit != 'true' }}
  uses: actions/cache/save@v5
```

결과:

- `pull_request`: restore-only, save skipped
- `workflow_dispatch` on `devel/main`: miss 또는 fallback이면 save 허용
- exact hit: save skipped

### 2. Render Diff npm cache 제거

`actions/setup-node@v4`에서 다음 설정을 제거한다.

```yaml
cache: npm
cache-dependency-path: rhwp-studio/package-lock.json
```

이유:

- before 표본에서 npm cache는 20/20 miss였다.
- npm save는 20/20 reservation failure를 남겼다.
- `Install Studio dependencies`는 cache miss 상태에서도 P50 6s / P90 7s였다.
- PR ref별 약 47 MB npm cache를 유지할 실익이 낮다.

## 포함 범위

- `.github/workflows/render-diff.yml`
- `mydocs/plans/task_m100_1667_impl_v2.md`
- `mydocs/working/task_m100_1667_stage2.md`
- `mydocs/orders/20260703.md`

## 제외 범위

- `mydocs/report/task_m100_1667_measurement.md`
- `mydocs/report/task_m100_1668_ci_pipeline_tracking.md`
- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- cache 삭제 또는 cleanup 자동화
- `Swatinem/rust-cache` 도입
- branch protection / required check 변경
- 회귀 가드 파일, `tests/**`, `tests/golden_svg/**` 변경

measurement/tracking 문서는 Render Diff code PR merge와 after 관측이 끝난 뒤 별도 최종 문서 PR에서 한 번에 반영한다.

## after 측정 기준

PR run:

- `Render Diff` workflow 완료 시간
- `Canvas visual diff` job 시간
- 주요 step 시간
  - Restore cargo registry & build cache
  - Build WASM package
  - Build native CLI for PDF report
  - Setup Node.js
  - Install Studio dependencies
  - Install Chromium
  - Check render diff script syntax
  - Run canvas visual diff and PDF report
- cargo cache restore hit/miss
- cargo cache save skipped 여부
- npm cache 관련 restore/save 로그가 사라졌는지
- cache reservation/read-only/save failure 경고가 사라졌는지
- `refs/pull/<PR>/merge` 신규 `Linux-render-diff-cargo-*` / `node-cache-*` 생성 여부
- branch protection / required check 변경 없음
- 회귀 가드 1:1 추적성 보존

trusted manual seed run:

- `workflow_dispatch` on `devel` 또는 `main`에서 cargo save가 성공하는지
- seed cache 크기
- 후속 PR run에서 seed cache가 restore 가능한지

## 롤백 기준

- PR full Render Diff `Canvas visual diff` job P90이 before 3m57s 대비 과도하게 악화되는 경우
- `Build WASM package` 또는 `Build native CLI for PDF report`가 반복적으로 실패하는 경우
- PR에서 cargo save가 다시 실행되는 경우
- required check 이름 또는 branch protection 표면이 바뀌는 경우

롤백 시 `.github/workflows/render-diff.yml`의 cargo cache step을 기존 단일 `actions/cache@v5`로 되돌리고,
`setup-node` npm cache 설정도 복구한다.
