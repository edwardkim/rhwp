# Task M100 #1664 Stage 1 완료 보고서

## 단계 목표

후속 코드 PR #1702에서 `Build & Test` job의 cargo cache step을 restore/save 분리 구조의 첫 단계로 전환한다.

이번 단계에서는 기존 `actions/cache@v5` 단일 step을 `actions/cache/restore@v5` restore step으로 바꾸고,
후속 Stage 2에서 trusted branch save 조건을 추가할 수 있도록 step id를 부여했다.

이 Stage 보고서는 문서 PR #1701에 포함되는 작업 기록이다. 실제 `.github/workflows/ci.yml` 변경은 후속 코드
PR #1702에만 포함되며, #1701 자체에는 workflow 변경이 없다.

## 변경 내용

후속 코드 PR #1702 수정 파일:

- `.github/workflows/ci.yml`

변경 사항:

- `Cache cargo registry & build` step을 `Restore cargo registry & build cache`로 변경했다.
- action을 `actions/cache@v5`에서 `actions/cache/restore@v5`로 변경했다.
- step id `cargo_cache_restore`를 추가했다.
- cache path, primary key, restore key는 기존 값을 그대로 유지했다.

유지한 항목:

- `Build & Test` job 이름
- build/test/clippy command
- cargo cache key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- cargo restore key: `${{ runner.os }}-cargo-`
- 통합 테스트 파일 구조와 회귀 가드 구조

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| `git diff --check` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | workflow 문법 오류 없음 |

## 관찰

- 이 단계만으로는 cache save step이 아직 없다.
- PR save 차단과 `devel` / `main` save 허용은 Stage 2에서 `actions/cache/save@v5` 조건부 step으로 추가해야 한다.
- 현재 상태는 restore-only 중간 상태이므로 Stage 2까지 이어서 완료해야 #1664 정책이 완성된다.

## 다음 단계 승인 요청

Stage 2 진행 승인 후 다음을 수행한다.

- `actions/cache/save@v5` step을 test/build/clippy 단계 이후에 추가한다.
- save 조건을 `refs/heads/devel` 또는 `refs/heads/main`으로 제한한다.
- exact cache hit가 아닌 경우에만 save를 시도하도록 조건을 구성한다.
- PR, tag, `workflow_dispatch`에서는 cargo cache save가 실행되지 않게 한다.
