# Task M100 #1664 Stage 2 완료 보고서

## 단계 목표

후속 코드 PR #1702에서 trusted branch만 cargo cache save를 허용하고, PR에서는 cache save를 차단한다.

이 Stage 보고서는 문서 PR #1701에 포함되는 작업 기록이다. 실제 `.github/workflows/ci.yml` 변경은 후속 코드
PR #1702에만 포함되며, #1701 자체에는 workflow 변경이 없다.

## 변경 내용

후속 코드 PR #1702 수정 파일:

- `.github/workflows/ci.yml`

추가한 step:

- `Save cargo registry & build cache`
- action: `actions/cache/save@v5`
- 위치: `Clippy` step 이후

save 조건:

```yaml
if: ${{ github.event_name == 'push' && (github.ref == 'refs/heads/devel' || github.ref == 'refs/heads/main') && steps.cargo_cache_restore.outputs.cache-hit != 'true' }}
```

조건 의미:

- `push` 이벤트에서만 save한다.
- `refs/heads/devel` 또는 `refs/heads/main`에서만 save한다.
- exact cache hit이면 동일 key 저장을 다시 시도하지 않는다.
- `pull_request`, tag, `workflow_dispatch`에서는 save step이 실행되지 않는다.

유지한 항목:

- cache path:
  - `~/.cargo/registry`
  - `~/.cargo/git`
  - `target`
- cache key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`
- `Build & Test` job 이름
- build/test/clippy command
- profile 정책
- job 구조
- 회귀 가드 구조

## 검증

| 항목 | 결과 | 비고 |
|------|------|------|
| `git diff --check` | 통과 | whitespace 문제 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 | save 조건식과 workflow 문법 오류 없음 |

## #1664 정책 충족 상태

- PR restore-only: 조건상 `pull_request`에서는 save step이 실행되지 않는다.
- trusted branch save: `push`의 `devel` / `main`에서만 save step이 실행된다.
- tag/manual run save 차단: tag는 `refs/tags/*`, manual run은 `workflow_dispatch`라 조건에서 제외된다.
- 현행 `actions/cache` 기반 유지: third-party action을 도입하지 않았다.
- profile/job/test 구조 변경 없음: #1666/#1665 범위를 침범하지 않았다.

## 남은 확인 항목

Stage 3에서 다음을 정리한다.

- 전체 변경 diff와 범위 확인
- 최종 검증 결과 정리
- before/after 측정 기준을 최종 보고서에 포함할 수 있도록 점검
- required check 표면이 유지되는지 확인
- 회귀 가드 162개가 PR마다 실행되는 전제가 보존되는지 문서화

## 다음 단계 승인 요청

Stage 3 진행 승인 후 최종 검증과 보고서 정리를 진행한다.
