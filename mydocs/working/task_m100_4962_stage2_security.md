# Task M100 #4962 W3 Stage 2-S — 무제한 성공 집계의 자원 고갈 방어

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/archives/task_m100_4962.md)
- **기능 기준선**: `830ddfe12`
- **보안 구현 commit**: `cc319822c`, `23af57b8f`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 2-S 완료, Stage 3 메인테이너 승인 대기

## 1. 판정 정정

Stage 2의 “page 문자 상한 없음”은 정상 입력을 4,096자에서 잘라 성공으로 가장하지 않는 정확성
계약이다. 그러나 최초 구현은 이를 자원 예산과 분리하지 못했다. parser의 압축 해제 상한이 있어도
collector가 문단·CharShapeRef 조합을 반복 검색하고 여러 문자 배열과 key 문자열을 만들면 CPU·heap
고갈이 별도로 가능했다.

따라서 Stage 2를 **기능 완료**로만 보존하고, 보안 완료 조건을 이 절편에서 다시 닫았다. 현재 계약은
다음과 같다.

```text
successful aggregate = 모든 대상 문자를 전수 집계하고 truncation 0
resource-limit        = 부분 aggregate를 폐기한 문서 전체 실패
cancelled             = 부분 aggregate를 폐기한 문서 전체 실패
```

성공 문자 수에는 임의 상한이 없다. 유한한 시스템 자원을 넘는 입력은 더 작은 성공 결과가 아니라 명시적
실패가 되며, corpus 성공 분모에 섞이지 않는다.

## 2. 발견한 공격면

### 2.1 CharShape 경계 CPU 증폭

최초 POC walker는 각 CharShape 구간마다 문단의 전체 `chars`를 `filter`했다. 문자 `N`, 경계 `R`인
입력에서 최악 `O(N×R)`가 되어, 작은 파일 안에서도 경계를 교차 배치하면 CPU를 증폭할 수 있었다.

현재 walker는 정렬된 문자 offset과 CharShapeRef에 monotonic cursor 하나를 사용한다. cursor는 뒤로
가지 않으므로 구간 분리는 `O(N+R)`이다.

### 2.2 메모리·allocator 증폭

decision key의 face·relation·source 문자열을 문자마다 새 `String`으로 만들던 경로를 `Arc<str>`과
`&'static str` 차원으로 바꿨다. face·layout·metric 이름은 language group에서 한 번 만들고 문자 key는
참조만 복제한다. 문단과 현재 language group에 필요한 vector는 남지만 문서 전체 raw decision record를
보존하지 않는다.

### 2.3 깊은 객체 재귀와 큰 aggregate

표·도형·캡션·주석 재귀에 중첩 깊이 예산을 추가했다. aggregate row와 JSON output byte도 별도 예산으로
제한한다. 각 dimension 문자열은 4,096 bytes를 넘을 수 없다. hash 전 report 복제가 output 예산보다
먼저 무한히 커지지 않도록 기본 row 예산도 문서당 20,000개로 낮췄다.

## 3. 기본 자원 정책

| 옵션 | 기본값 | 허용 최대 | 의미 |
| --- | ---: | ---: | --- |
| `maxWorkUnits` | 10,000,000 | 2,000,000,000 | decoded byte·문자·경계·decision·control의 결정적 작업량 |
| `deadlineMillis` | 60,000 | 3,600,000 | cooperative wall-clock deadline |
| `maxNestingDepth` | 128 | 4,096 | 표·도형·캡션·주석 재귀 깊이 |
| `maxAggregateRows` | 20,000 | 200,000 | legacy+decision retained row 합계 |
| `maxOutputBytes` | 32 MiB | 128 MiB | 최종 UTF-8 JSON 봉투 크기 |

옵션은 `deny_unknown_fields`로 해석한다. 0, 범위 밖 값과 알 수 없는 key는 실행 전에 실패한다. caller가
큰 정상 문서를 의도적으로 처리할 때 예산을 올릴 수 있지만, 입력 문서가 이 정책을 바꿀 수는 없다.

외부 supervisor는 `AtomicBool`을 전달하는
`get_font_metric_coverage_analysis_with_cancel_native()`로 cooperative cancellation을 요청할 수 있다.
budget check는 문단, 문자 구간, decision, control과 재귀 경계에서 반복된다.

## 4. 오류와 corpus 분모

- work, deadline, depth, row, dimension 또는 output 초과:
  `[RESOURCE_LIMIT_EXCEEDED]`
- cancellation 요청:
  `[ANALYSIS_CANCELLED]`

두 오류 모두 JSON `status=complete`나 부분 usage row를 반환하지 않는다. contract의 document failure
inventory에는 `resource-limit`과 `cancelled`를 추가했고, 성공 aggregate도 모든 failure key를 0으로
명시한다.

Stage 3 corpus supervisor는 오류 code만 비식별 failure count로 변환해야 한다. 오류 문자열, 파일명,
경로와 stack은 aggregate에 넣지 않는다.

## 5. process 격리 경계

cooperative budget은 library 호출 하나가 정상적으로 unwind하도록 만드는 1차 방어다. allocator abort,
native dependency hang 또는 예산 check 사이의 peak RSS까지 같은 process 안에서 완전히 격리하지는 못한다.

따라서 contract의 `corpusWorkerIsolation`은 `required`다. 아직 corpus runner 자체가 없는 현재 단계에서는
private 문서를 실행하지 않는다. Stage 3은 다음 순서가 아니면 pilot에 진입할 수 없다.

1. 문서 한 개당 별도 worker process
2. parent가 강제하는 wall-clock timeout
3. OS가 강제하는 RSS/address-space limit
4. timeout·signal·OOM을 `resource-limit`으로 비식별 집계
5. worker 실패 후 다음 문서를 계속할 수 있음을 공개 fixture로 검증

즉 process 격리는 “향후 권고”가 아니라 Stage 3 runner의 첫 번째 hard gate다. 이 runner가 없으므로 이번
절편에서 private pilot이나 전수 계측은 실행하지 않았다.

## 6. 공격형 integration 검증

일반 checkout에는 generated suite를 만들지 않았다. commit `23af57b8f`의 detached review worktree에서
`tests/cases/issue_4962_font_metric_coverage.rs`를 `regression_suite_019`로 준비해 실행했다.

```bash
cargo test --test regression_suite_019 issue_4962_font_metric_coverage -- --nocapture
```

결과: **5 passed, 0 failed**.

- 정상 공개 fixture의 반복 JSON·hash와 모든 분모 일치
- 5,000자 정상 long-page의 전수 성공과 truncation 0
- work 1, row 1, output 1 KiB 예산의 명시적 전체 실패
- 사전 cancellation의 `[ANALYSIS_CANCELLED]` 전체 실패
- unknown/0 option 거부
- 10,000자와 10,000 CharShape 경계를 100,000 work units·10초 query deadline 안에서 전수 대사
- collector 호출 전후 W2 trace byte equality

공격 fixture의 전체 test 시간은 문서 편집·파생 상태 재구축을 포함해 40.08초였다. collector 자체는
10초 deadline 안에서 완료했다. 이 값은 Stage 3 처리량 추정치가 아니라 복잡도 퇴행 방지 증거다.

같은 최종 commit에서 기존 W2 integration도 재실행했다.

```bash
cargo test --test regression_suite_003 issue_4961_font_decision_trace -- --nocapture
```

결과: **4 passed, 0 failed**.

추가 검증:

- `node --test scripts/tests/font_metric_coverage_contract.test.mjs`: 10 passed
- `cargo check --lib`: 통과
- `node scripts/rust-unit-test-tiers.mjs --check`: 4,225 tests 기준 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

## 7. 남은 경계

완료된 것은 collector 내부의 안전한 실패와 Stage 3 격리 계약이다. 아직 수행하지 않은 것은 다음과 같다.

- corpus supervisor 구현과 process 강제 종료 검증
- private 소규모 pilot
- peak RSS·처리량 측정
- full 10k delta pass
- 원격 push와 PR

따라서 다음 승인은 Stage 3 전체가 아니라 먼저 **격리 supervisor 구현·공개 fixture 검증 절편**에만 적용할
수 있다. 그 절편이 통과하기 전에는 private corpus를 worker에 전달하지 않는다.
