# 단계 완료보고서 — #3268 Stage 1 Frontend·Lint 병렬화

- **이슈**: [edwardkim/rhwp#3268](https://github.com/edwardkim/rhwp/issues/3268)
- **브랜치**: `task/3268-frontend-lint-parallel`
- **기준**: `upstream/devel@204c56528`

## 구현 결과

`frontend-package-gates`의 `needs`를 `[preflight, lint]`에서 `preflight`로 축소했다.
Frontend의 조건식은 preflight 성공, fast-pass 제외, frontend 영향 조건을 그대로 유지한다.

```text
preflight
  ├─ Lint ────────────────┐
  └─ Frontend package gates ─┴→ { Native Skia, Build test archive } → shards → Build & Test
```

`Native Skia tests`와 `Build test archive`의 `needs: [preflight, lint,
frontend-package-gates]` 및 두 worker 성공 조건은 변경하지 않았다. 따라서 Lint 또는 Frontend가
실패하면 두 downstream worker와 shard는 시작하지 않는다. Lint 실패 뒤 이미 시작한 Frontend가 끝날
수 있는 runner 비용은, 기존 fast-pass와 실패 전파를 보존하기 위해 허용한 trade-off다.

## Cargo lock 안전성

- Lint와 Frontend는 각각 `runs-on: ubuntu-latest`의 독립 hosted runner에서 실행한다.
- job 간 workspace와 `target/` 디렉터리를 공유하지 않으므로 Cargo 파일 lock이 생기지 않는다.
- Frontend cache는 restore-only이고 PR에서는 save하지 않는다. Lint의 `rust-cache`와 Frontend WASM
  cache는 key 정책도 다르다.
- 이 작업에서는 Cargo 명령을 실행하지 않았다. Cargo 명령을 병렬로 실행하지 않는 규칙도 유지한다.

## 변경 범위

| 파일 | 변경 |
| --- | --- |
| `.github/workflows/ci.yml` | Frontend의 Lint 의존성 제거, 병렬 경로·downstream gate 주석 보강 |
| `mydocs/plans/task_m100_3268.md` | 목표, Cargo lock 안전 조건, 검증 계획 |
| 본 문서 | 구현 및 검증 결과 |

## 검증

| 검증 | 결과 |
| --- | --- |
| `git diff --check` | PASS |
| `actionlint .github/workflows/ci.yml` | PASS |
| Ruby YAML·DAG 계약 검사 | PASS — Lint/Frontend preflight 병렬, downstream dual-success gate, shard 의존성, 독립 hosted runner 확인 |

이 작업은 CI workflow 변경이므로 제품 Cargo 테스트를 로컬에서 실행하지 않았다. 최신 PR head의
GitHub Actions에서 frontend 영향 run이 실제로 병렬 시작하고 required checks가 모두 통과하는지 확인하는
것이 최종 검증이다.
