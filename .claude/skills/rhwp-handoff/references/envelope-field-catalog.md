# 봉투 필드 사전

세션 핸드오프가 **읽는** 필드만 적는다. 오케스트레이터와 replay 가 내는
키를 재명명하지 않는다.

## orchestrator 최종 봉투 (`operation: agent.handoff`)

| 필드 | 형 | 읽는 이유 |
|---|---|---|
| `protocol` | `"DAP/1.0"` | 다른 봉투와 섞지 않기 |
| `operation` | `"agent.handoff"` | last result 판별 |
| `tool` | `"rhwp-handoff-orchestrator"` | 도구 고정 |
| `handoffVersion` | `"1.0"` | 버전 |
| `taskId` | 문자열 | working doc 과 대조 |
| `taskSha256` | 64hex | 같은 task 인지 |
| `status` | ok/error/verdict | DAP 3분류 |
| `code` | 0/1000/3000/4000 | DATP 대역 |
| `outcome` | accepted/handoff/rejected | 세션 분기 |
| `agentsTried` | 문자열 배열 | 누가 시도됐나 |
| `acceptedAgent` | 문자열\|null | 수용된 라벨 |
| `attempts[]` | 객체 | category·findings·nextAction |
| `result` | HandoffResult\|null | untrusted |
| `capabilities[]` | 객체 | untrusted |
| `collectedOutputs[]` | `{path,sha256}` | 수거 대조 |
| `nextAction.action` | consume/retry/fallback/selfExecute | 다음 동작 |
| `nextAction.why` | 문자열 | 이유 (데이터) |
| `untrustedContent` | bool | 수용 시 true |
| `untrustedFields` | 문자열 배열 | 지시로 읽지 말 키 |
| `journal` | 경로 | `--verify-journal` |

## verifyJournal 봉투

| 필드 | 형 |
|---|---|
| `operation` | `"agent.handoff.verifyJournal"` |
| `entries` | 정수 |
| `chainValid` | bool |
| `brokenAt` | 정수\|null |

이 봉투는 last result 가 아니다.

## 세션 예외 봉투 (`fixtures/envelopes/`)

공통 `_skillMeta`:

| 필드 | 값 |
|---|---|
| `exit` | 0/1/2/3/4 |
| `command` | `python` / `rhwp` / `git` / `read` |
| `trigger` | 세 트리거 또는 null |
| `exception` | 네 갈래 또는 null |
| `stdoutSilentOnFail` | bool |

발명 명령 이름(`handoff`, `receipt`, `session-resume`)을 `command` 에 쓰지 않는다.

## workCapsule (포인터)

`kind`, `parent`, `plan`, `planText`, `receipt.{inputSha256,planSha256,outputSha256,toolVersion,steps,mode}`
— work-receipt 와 같다. 세션 필드(`handoffTrigger`)를 캡슐에 넣지 않는다.
트리거는 working doc 과 `_skillMeta` 에만 둔다.
