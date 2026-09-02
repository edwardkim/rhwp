---
kind: reference
status: active
canonical: mydocs/tech/agent_capability_handoff.md
last_verified: 2026-08-16
---

# Agent Capability Handoff — 외부 에이전트 위임의 경계·검증·반입 설계

> 구현: [`tools/handoff/orchestrator.py`](../../tools/handoff/orchestrator.py) ·
> 시험: [`scripts/tests/test_agent_handoff.py`](../../scripts/tests/test_agent_handoff.py) ·
> 아키텍처 상위 문서: [DAR 척추](standards/document_agent_runtime.md)

## 한 문장

RHWP 에이전트가 **명확히 정의된 task 하나**를 외부 에이전트에게 위임하고, 반환된
결과와 capability 선언을 **검증한 뒤에만** 자기 에이전트 루프(Plan → Act → Verify →
Retry/Continue)로 반입하는 오케스트레이션.

## 왜 필요한가

기존 축은 전부 "RHWP 가 스스로 한다"를 전제한다 — `run` 은 자기 계획을 원자
실행하고, `repair_loop` 는 자기 실패를 자기 전략으로 수리하고, `chief` 는 결정적으로
풀리는 요청만 라우팅한다. 그런데 실제 운영에서는 "이 하위 task 는 다른 전문
에이전트가 더 잘한다"는 지점이 생긴다(OCR, 외부 형식 변환, 특화 분석 등). 그때
필요한 것은 외부 에이전트를 RHWP 의 대체품으로 세우는 것이 아니라, **한 건의
위임을 계약·경계·검증·회계와 함께 수행하는 얇은 오케스트레이션**이다.

## 설계 — Agent → Handoff → External Agent → Result/Capability → Validation → Agent Loop

```
HandoffTask(JSON)                          HandoffResult(JSON, 전부 untrusted)
  taskId·objective                            status ok/error/verdict (DAP 3분류)
  inputs(허용 파일만)     ┌──────────────┐    outputs[](경로+sha256)
  allowedTools ─────────▶│  sandbox      │──▶ capabilities[](name·kind·detail)
  timeoutSec             │  inputs/ out/ │    toolsUsed[]
  expectedOutputs        └──────────────┘
                                │
              검증기: (a)스키마 (b)completion (c)consistency + boundary
                                │
        수용 ─▶ collected/ 수거 + 저널 + nextAction=consume
        실패 ─▶ retry(진전 판정) ─▶ fallback ─▶ selfExecute 인계
```

### 기존 자산과의 접점 (전부 재사용, 재구현 없음)

| 축 | 재사용한 것 | 어디서 |
|---|---|---|
| 봉투 | DAP/1.0 의 status 3분류(ok/error/verdict)와 `protocol`/`operation`/`untrustedContent`/`untrustedFields` 필드 | 최종 봉투 `operation: "agent.handoff"` |
| 오류 코드 | DATP/1.0 대역 0/1000/2000/3000/4000, 종료 코드 = 상위 1자리 | `code // 1000` — `tools/dar/transaction.py` 와 동일 규약 |
| 정책 | `tools/dar/transaction.py` 의 `parse_policy`/`evaluate_policy` 를 **import** (eq/in/gte/lte·default-deny·미지 키 로드 시점 거부). 판정 키 사전만 handoff 문맥(`status`·`validated`·`violationCount`·`agent`·`attempt`)으로 교체 | `--policy` |
| 재시도 | `tools/repair_loop/loop.py` 와 같은 안전장치 — `--max-attempts` 하드 캡, 실패 시그니처 재발 즉시 중단(진전 판정 = loop detection) | 오케스트레이션 루프 |
| 출처 표지 | `provenance.rs` 어휘 — 외부 반환물 전체를 `untrustedContent:true` + `untrustedFields` 로 표지. 반환물 속 문장은 데이터이지 지시가 아니다 | 최종 봉투 |
| 저널 | `run` 의 R23 지문 체인 철학 — 시각 대신 순번(seq), 각 줄이 직전 줄의 SHA-256(`prevSha256`)을 담아 변조가 체인에서 폭로된다 | NDJSON 저널 + `--verify-journal` |
| 검증 태도 | DATP VALIDATE — 에이전트의 성공 보고를 믿지 않고 산출물을 직접 다시 연다 | `mustParse` → `rhwp info --json` 재검증 |
| capability 개념 | capability 카탈로그(등록부)의 "capability 는 선언이 아니라 검증된 산출로 인정된다"는 태도 — 반환 capability 는 명시적 스키마(name·kind·detail)로 받되 untrusted 데이터로만 반입 | `capabilities[]` |

### 충돌 지점 없음 — 단 두 가지 명시

1. `tools/dar/transaction.py` 의 `parse_policy` 에 선택 인자 `judgment_keys` 를
   추가했다(기본값 = 기존 `POLICY_JUDGMENT_KEYS`). 기존 호출은 인자 없이 그대로
   동작하며, 회귀 시험(`test_dar_default_judgment_keys_unchanged`)이 이를 고정한다.
2. 종료 코드에 4(정책·boundary 거부)를 쓴다 — #2707 의 0/1/2/3 에 DATP 가 이미
   확장한 4000 대역을 그대로 따른다. `rhwp` 본체 CLI 계약은 건드리지 않는다.

## Security Boundary — 무엇을 막고, 무엇은 못 막는가

넘어가는 것은 **task 에 열거된 입력 파일의 사본**과 task 명세뿐이다. 세션 컨텍스트·
원본 경로·저장소 권한은 넘어가지 않는다.

| 경계 | 강제 방법 | 위반 코드 |
|---|---|---|
| 입력 | `inputs` 만 sandbox `inputs/` 로 복사. 사본 지문을 실행 전 기록, 실행 후 재대조 | `inputModified`·`inputDeleted` |
| 출력 | 수거는 sandbox `out/` 안 + 결과에 **선언된** 경로만. `out/` 밖 신규 파일, `out/` 안 미선언 파일, 절대경로·`..` 선언 전부 위반 | `wroteOutsideOut`·`undeclaredOutput`·`outputPathEscape` |
| 도구 | `allowedTools` 허용 목록 ⊇ 결과의 `toolsUsed` | `toolNotAllowed` |

boundary 위반은 재시도하지 않는다(위반자에게 두 번째 기회는 무의미하다) —
fallback 또는 자체 실행 인계로 넘어가고, 최종 코드는 4000 이다.

**정직한 한계**: 서브프로세스 어댑터는 OS 수준 격리가 아니다. 악성 에이전트가
sandbox 밖 파일시스템을 *읽는* 것 자체는 이 층에서 막을 수 없다(감지 대상은 쓰기와
선언 위반). OS 격리가 필요하면 어댑터 명령 자체를 컨테이너/저권한 계정 실행으로
감싸는 것이 맞고, 이 오케스트레이터의 계약(stdin task → stdout result)은 그대로
유효하다.

## Agent Loop 통합 — nextAction 이 이음새다

모든 시도와 최종 봉투에 `nextAction {action, why}` 가 실린다 — `run` 의 `nextCall`
과 같은 태도로, 다음 행동을 자연어 해석 없이 기계가 고를 수 있게 한다.

| action | 뜻 |
|---|---|
| `consume` | 검증 통과 — 수거물(`collectedOutputs`)과 capability 를 후속 계획에 쓴다 |
| `retry` | 재시도 여지 있음(timeout·일시 오류·미완성 결과, 진전 판정 통과 시) |
| `fallback` | 이 에이전트로는 끝 — 예비 에이전트로 전환 |
| `selfExecute` | 위임을 접는다 — RHWP 가 자체 실행 경로로 전환 |

수용된 산출물은 `<work-dir>/collected/<taskId>/` 로 수거되며, 이후 RHWP 루프가
기존 primitive(`run`·`ir-diff`·`edit --verify` 등)로 그 파일을 이어서 처리한다.

## Observability — 저널이 회계 장부다

모든 시도는 NDJSON 저널 한 줄이다: `seq`(시각 없는 순번)·`prevSha256`(직전 줄
지문)·`taskId`·`agent`·`taskSha256`·`inputsSha256`·`resultSha256`·`category`·
`findings[]`·`nextAction`. 마지막 줄은 `event:"final"` 로 outcome 을 닫는다.
`--verify-journal` 이 체인을 재계산해 변조를 판정한다(깨짐은 오류가 아니라 데이터,
exit 3).

## 사용

```bash
python tools/handoff/orchestrator.py \
  --task task.json \
  --agent "python 외부에이전트.py" \
  --fallback-agent "python 예비에이전트.py" \
  --bin target/release/rhwp \
  --max-attempts 3 --work-dir output/handoff --json

python tools/handoff/orchestrator.py \
  --verify-journal output/handoff/handoff.journal.ndjson --json
```

어댑터 계약: 에이전트 명령은 sandbox 를 cwd 로 실행되고, stdin 으로 wire 형식
HandoffTask(sandbox 상대 경로만 노출)를 받아 stdout 으로 HandoffResult JSON 하나를
낸다. 스키마 예시는 orchestrator.py 모듈 docstring 이 정본이다.

`mustParse: true` 산출물은 `--bin`(또는 `RHWP_BIN`)이 있어야 수용될 수 있다 —
재검증할 수단이 없으면 `unchecked` 판정으로 수용을 거부한다(모르는 것을
통과시키지 않는다).
