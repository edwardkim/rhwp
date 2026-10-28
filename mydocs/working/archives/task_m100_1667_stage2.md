# Task M100 #1667 Stage 2 완료 보고서

## 단계 목표

Render Diff workflow의 PR ref cache save 표면을 제거한다. before 관측에서 Render Diff cargo/npm cache는 최신 PR
run에 재사용되지 않았고, post-save 단계에서 reservation/read-only 실패 로그만 남겼다.

## 변경 파일

| 파일 | 변경 |
|------|------|
| `.github/workflows/render-diff.yml` | Render Diff cargo cache restore/save 분리, npm cache 제거 |
| `mydocs/plans/task_m100_1667_impl_v2.md` | Render Diff cache 구현계획서 추가 |
| `mydocs/working/task_m100_1667_stage2.md` | Stage 2 완료 보고서 추가 |
| `mydocs/orders/20260703.md` | #1667 진행 상태 갱신 |

## 구현 내용

### cargo cache

- 기존 `actions/cache@v5` 단일 step을 `actions/cache/restore@v5`와 `actions/cache/save@v5`로 분리했다.
- restore step id는 `render_diff_cargo_cache_restore`로 지정했다.
- save step은 아래 조건에서만 실행된다.
  - `workflow_dispatch`
  - `refs/heads/devel` 또는 `refs/heads/main`
  - exact cache hit가 아님
- PR run에서는 cargo cache save가 실행되지 않는다.

### npm cache

- `actions/setup-node@v4`의 `cache: npm` 설정을 제거했다.
- before 표본에서 npm cache는 20/20 miss였고, save는 20/20 reservation failure였다.
- `npm ci`는 cache miss 상태에서도 P50 6s / P90 7s였으므로 PR ref별 npm cache를 유지하지 않는다.

## 유지한 사항

- Render Diff workflow 이름 유지
- `Canvas visual diff` job 이름 유지
- Render Diff preflight / fast-pass 로직 유지
- cargo cache key namespace `Linux-render-diff-cargo-*` 유지
- cargo cache path `~/.cargo/registry`, `~/.cargo/git`, `target` 유지
- `wasm-pack build --target web --dev` 유지
- `cargo build --bin rhwp` 유지

## 제외한 사항

- Build & Test cache 변경 없음
- CodeQL cache 변경 없음
- `Swatinem/rust-cache` 도입 없음
- cache cleanup 또는 삭제 없음
- cleanup 자동화 없음
- measurement/tracking 문서 변경 없음
- 테스트/회귀 가드 변경 없음

## 로컬 검증

| 검증 | 결과 |
|------|------|
| `git diff --check` | 통과 |
| `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/render-diff.yml")'` | 통과 |
| `actionlint .github/workflows/render-diff.yml` | 통과 |

Ruby YAML parse 실행 중 로컬 gem `ffi-1.13.1` 확장 경고가 출력됐지만, workflow parse 자체는 `yaml ok`로
완료됐다.

## PR CI 관측 항목

- Render Diff preflight 결과
- full `Canvas visual diff` 실행 여부
- `Restore cargo registry & build cache` hit/miss
- `Save cargo registry & build cache` skipped 여부
- npm cache restore/save 로그가 사라졌는지
- cache reservation/read-only/save failure 경고가 사라졌는지
- `Build WASM package`, `Build native CLI for PDF report`, `Install Studio dependencies`, `Run canvas visual diff and PDF report` 시간
- PR ref 신규 `Linux-render-diff-cargo-*` / `node-cache-*` 생성 여부

## 후속

PR CI와 merge 후 after 관측이 끝나면 최종 measurement/tracking 문서 PR을 별도로 열어 before/after를 한 번에 정리한다.
