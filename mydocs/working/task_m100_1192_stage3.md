# Task #1192 Stage 3 완료 보고서 — render-diff.yml (F) + 전체 검증

- **이슈**: #1192
- **브랜치**: `local/task1192`
- **단계**: Stage 3 / 3
- **대상**: `.github/workflows/render-diff.yml` + 전체 워크플로우 검증

## 적용 내용

### F — concurrency 취소 그룹 (render-diff.yml)
- 최상위(`env:` 다음)에 추가:
  ```yaml
  concurrency:
    group: render-diff-${{ github.workflow }}-${{ github.head_ref || github.ref }}
    cancel-in-progress: ${{ github.event_name == 'pull_request' }}
  ```
- render-diff 는 pull_request 트리거 위주 → PR 이벤트 한정 cancel.

## 전체 검증 (3개 워크플로우 정적 확인)

| 확인 | 결과 |
|------|------|
| YAML 유효성 (ci/codeql/render-diff) | ✅ 3개 모두 VALID, concurrency=yes |
| ci: Canvas parity 제거 | ✅ 0 |
| ci: Native Skia tests 유지 | ✅ 1 |
| ci: 느린 disk 항목(apt/docker/ghc/CodeQL) 제거 | ✅ 0 |
| codeql: rust cargo 캐시 step | ✅ 1 |
| codeql/render-diff: concurrency | ✅ 각 1 |

> actionlint 는 로컬 미설치 → python `yaml.safe_load` 로 구문 검증 대체.

## 런타임 검증 (devel push CI 관찰)

워크플로우 변경은 PR CI 에 즉시 반영되지 않을 수 있어, `local/task1192` → devel 머지 + push
직후 실행되는 devel CI run 을 최종 검증으로 삼는다. 관찰 항목:

- A: `Run tests` 로그에 canvas_layer_tree_matches_legacy + native-skia 각 1회 실행.
- B: CodeQL rust 로그 cache restore + 빌드 시간 단축.
- C: `df -h` 출력 디스크 여유(빌드 성공).
- 전체 green + Build & Test / CodeQL rust 벽시계 단축.
- (F 는 후속 PR 의 연속 push 에서 cancelled 전환 관찰 — 본 push 단독으로는 확인 제한)

> 런타임 결과는 최종 보고서 `task_m100_1192_report.md` 에 기록.
