# Task M100 #3064 구현 보고서

## 목표

PR #3054에서 한 필수 CI gate가 실패한 뒤에도 다른 고비용 테스트가 계속 실행된 문제를 해결한다.
8개 기본 테스트 shard의 정상 병렬성은 유지하고, 실패가 발생한 시점부터 남은 작업만 중단한다.

## 원인

- `lint`, `frontend-package-gates`, `native-skia-tests`, `build-test-archive`가 각각 `preflight`만
  의존해 sibling job으로 동시에 시작했다.
- GitHub Actions는 한 sibling job 실패만으로 이미 시작한 다른 sibling job을 자동 취소하지 않는다.
- `test-shard.strategy.fail-fast`가 명시적으로 `false`여서 shard 실패도 나머지 matrix job에
  전파되지 않았다.

## 변경

필수 worker의 선행 성공 조건을 다음과 같이 구성했다.

```text
preflight
  → lint
  → frontend-package-gates (frontend 비대상이면 skipped)
  → native-skia-tests
  → build-test-archive
  → test-shard 1/8 ... 8/8 (병렬)
  → Build & Test 집계
```

- frontend 비대상인 정상 `skipped`와 frontend 실패·취소를 구분해, 비대상일 때만 native-skia 이후
  체인이 진행된다.
- 8개 shard의 matrix 배열과 병렬 실행은 유지하고 `fail-fast: true`로 바꿨다.
- shard 실패로 count artifact가 완성되지 않은 경우 최종 집계에서 다운로드/합계 검사를 생략하고,
  worker 결과 검사에서 원래 실패 상태를 보고하도록 했다.
- 같은 집계 script의 기존 `ls` 카운트를 `find`로 바꿔 `actionlint` SC2012 진단을 제거했다.

## 실패 전파 계약

| 실패 위치 | 실행하지 않는 후속 worker |
|---|---|
| lint | frontend, native-skia, archive, shard |
| frontend | native-skia, archive, shard |
| native-skia | archive, shard |
| archive | shard |
| shard 1개 | 실행 중이거나 대기 중인 나머지 shard 취소 |

최종 `Build & Test`는 테스트 worker가 아니라 required check 집계이므로 항상 실행해 실패 원인을 표시한다.
CodeQL과 Render Diff는 별도 workflow라 이 의존성 체인의 범위가 아니다.

## 검증

| 항목 | 결과 |
|---|---|
| Ruby YAML parse | PASS (`yaml ok`) |
| `actionlint .github/workflows/ci.yml` | PASS |
| job `needs` 의존성 계약 검사 | PASS |
| matrix shard `[1,2,3,4,5,6,7,8]` 보존 | PASS |
| `test-shard.strategy.fail-fast == true` | PASS |
| `git diff --check` | PASS |

제품 Rust·TypeScript 코드와 실제 테스트 명령은 변경하지 않았다. CI workflow 변경의 최종 동작과 전체
회귀는 PR 최신 head의 GitHub Actions에서 확인한다.

## 후속 확인

- 정상 경로에서 8개 shard가 병렬 시작하고 모두 성공하는지 확인
- 실패 전파는 후속 실패 재현 또는 다음 실제 gate 실패 run에서 job skip/cancel 상태 확인
- PR CI 전체 성공 뒤 merge하고 #3064를 완료 처리

## PR #3077 도구 참조 보강

작업 중 merge된 `scripts/pr_triage.sh`의 사용처가 정식 절차 문서에 없다는 작업지시자 지적을 반영했다.
`pr_review_workflow.md` 2.0절에 대량 PR 유입 시의 실행 명령, 조회 상한, 전수 누락 확인, 사실 수집과
사람의 최종 판정 경계를 추가했다. 개별 review 단계의 reviewer 선행 assign 규칙은 그대로 유지한다.
