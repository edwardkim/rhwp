# Issue #4961 — Font Decision Trace

## 1. 목적

이 디렉터리는 실제 문서의 한 문자가 어떤 font name·metric·paint·backend 결정을 거쳤는지 설명하는
W2 Font Decision Trace의 계약과 공개 fixture 경계를 보존한다.

이 trace는 읽기 전용 진단이다. fallback target, metric 값, paint face, font 공급과 기본 renderer
결과를 선택하는 authority가 아니다.

## 2. 권위와 선행 자료

- 수행 범위와 승인 게이트: [`task_m100_4961.md`](../../../plans/task_m100_4961.md)
- W1 원장과 candidate identity: [`issue-4939`](../issue-4939/README.md)
- 보호 불변식: [`font_metrics_fallback_causal_lineage_20260816.md`](../../../report/font_metrics_fallback_causal_lineage_20260816.md)
- machine-readable trace 계약: [`font_decision_trace.schema.json`](font_decision_trace.schema.json)

W1의 `font_rule_ledger.json`은 historical investigation snapshot이다. W2는 실제 결정이 끝난 뒤
candidate identity로 evidence link를 계산하고 원장과 교차 검사한다. 원장을 runtime font 선택 표로
import하거나 W7 registry projection을 선행하지 않는다.

## 3. Stage 1 산출물

| 파일 | 역할 |
| --- | --- |
| `font_decision_trace.schema.json` | v1 봉투·문자 record·backend certainty·상한 계약 |
| `font_decision_identity_vectors.json` | Rust/TypeScript가 공유할 W1 candidate identity hash golden |
| `public_fixtures.json` | Stage 4에서 사용할 repository-tracked HWP/HWPX 후보와 digest |
| `scripts/font_decision_trace_contract.mjs` | identity, ledger join, 상한, hash와 민감정보 검사 |
| `scripts/tests/font_decision_trace_contract.test.mjs` | 정상·실패·결정론 계약 회귀 |

Stage 1은 production renderer·resolver·Studio source를 변경하지 않는다. Stage 2 이후 구현은 이 계약을
소비해야 하며, 계약에 없는 값을 조용히 채워 넣지 않는다.

## 4. identity와 ledger 연결

candidate identity의 필드는 W1 collector와 같다.

```text
sourceBoundaryId
candidateKind
sourceFace
targetOrPolicy
conditions
order
```

object key를 재귀 정렬한 pretty JSON과 끝의 LF 한 개를 SHA-256으로 계산한다. 앞 20 hex를 사용해
`candidate.<hash>`와 `rule.<sourceOwner>.<hash>`를 만든다. 이 ID는 이미 내려진 결정에 provenance를
붙이는 용도이며 선택 입력이 아니다.

원장에 같은 `ruleId`, owner와 candidate evidence anchor가 없으면 trace는 `ruleId: null`과
`ledgerRuleMissing`을 반환한다. unknown relation/evidence는 그대로 유지한다.

## 5. 상한과 fail-closed

- page 한 개, 기본 1,024문자, hard maximum 4,096문자
- `0`, 음수, 비정수와 maximum 초과는 clamp하지 않고 거부
- source 순서 수집, 초과 시 `truncated`와 최초 누락 좌표
- unsupported·not-observed·join 실패·source drift를 빈 성공으로 바꾸지 않음
- trace가 font fetch, 권한 요청, document reload 또는 repaint를 시작하지 않음
- 절대 host path, 사용자명, access token과 exception stack을 trace에 넣지 않음

## 6. hash

`layoutHash`는 source·document·layout name·layout metric과 provenance의 portable 부분만 포함한다.
`normalizedHash`는 현재 backend capability 결과도 포함한다. timestamp, elapsed time, 절대 path,
error stack과 hash 필드 자신은 제외한다.

object key와 의미상 unordered인 capability·failure·known limitation은 정규화하지만, source record와
fallback candidate chain처럼 순서가 정책인 배열은 보존한다.

## 7. 공개 fixture 경계

[`public_fixtures.json`](public_fixtures.json)은 이미 repository에 추적된 HWP/HWPX 한 쌍만 가리킨다.
private 10k corpus의 문서·파일명·식별 목록·절대 경로는 사용하지 않는다. fixture digest가 달라지거나
추적 상태가 사라지면 검사는 실패한다.
