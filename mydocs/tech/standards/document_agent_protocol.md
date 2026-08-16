---
kind: reference
status: active
canonical: mydocs/tech/standards/document_agent_protocol.md
last_verified: 2026-08-16
---

# 문서 에이전트 프로토콜 (Document Agent Protocol, DAP) 1.0

> 기계용 정본은 [`document_agent_protocol.json`](document_agent_protocol.json)이다.
> 둘이 어긋나면 [`tools/dar/conformance.py --self-check`](../../../tools/dar/conformance.py)
> 가 잡는다. 아키텍처 전체는 [DAR 척추](document_agent_runtime.md).

## 한 문장

에이전트와 문서 런타임이 주고받는 **요청·문서 신원·트랜잭션·신뢰수준·정책·결과·
오류를 기계 판독 가능한 한 봉투로 묶는** 도구 무관 프로토콜.

## 왜 필요한가

rhwp 는 이미 명령마다 `--json` 봉투를 내고, 능력을 자기서술하고, 문서 파생 값에
출처 표지를 붙이고, 종료 코드 계약(0·1·2·3·4)을 지킨다. 없는 것은 **그 조각들을
하나의 대화로 묶는 신원**이다: 요청 id 가 없어 재시도가 멱등인지 알 수 없고,
트랜잭션 id 가 봉투에 없어 여러 연산이 한 작업인지 알 수 없고, 오류가 자연어라
에이전트가 문자열을 파싱해야 한다.

DAP/1.0 은 **새 런타임이 아니라 결속 층**이다.

## 요청 봉투

```json
{
  "protocol": "DAP/1.0",
  "request_id": "01J8…",
  "operation": "document.select",
  "document": { "sha256": "9f2c…", "format": "hwpx" },
  "transaction_id": "tx_01J8…",
  "trust": "untrusted",
  "policy": { "id": "readonly-triage", "sha256": "4ab1…" },
  "selector": "table[0].row[2]",
  "params": {},
  "cursor": null
}
```

- **문서의 신원은 내용 해시다.** 경로는 신원이 아니다 — 같은 경로가 다른 문서일 수
  있고, 다른 경로가 같은 문서일 수 있다.
- `request_id` 는 **재시도 시 같은 값**을 쓴다. 런타임은 이미 처리한 id 를 다시
  실행하지 않고 같은 결과를 돌려줘도 된다(멱등).
- `trust` 의 기본값은 `untrusted` 다 — 표시를 빠뜨리는 쪽이 안전한 실수가 되게 한다.

## 결과 봉투

```json
{
  "protocol": "DAP/1.0",
  "request_id": "01J8…",
  "operation": "document.select",
  "status": "verdict",
  "code": 3002,
  "record": { "matches": [] },
  "untrustedContent": true,
  "untrustedFields": ["record.matches[].text"],
  "retryable": false,
  "cursor": null
}
```

`status` 는 셋이다: `ok` · `error` · **`verdict`**. 판정은 실패가 아니다 —
"매치 0건", "재현 불일치", "검증 실패"는 런타임이 정상 동작해서 낸 **결과**이며,
재시도 대상이 아니라 소비 대상이다. 이 구분이 에이전트의 재시도 루프를 멈추게 한다.

## 오류 코드 — 숫자가 계약이다

자연어 `message` 는 사람용이며 **판정 근거로 쓰지 않는다.** 상위 1자리는 rhwp 의
기존 종료 코드 계약과 맞춘다.

| 대역 | 뜻 | 종료 코드 | 재시도 |
|---|---|---|---|
| `0` | 성공 | 0 | — |
| `1000`대 | 런타임 실패(일시적일 수 있음) | 1 | 가능 |
| `2000`대 | 사용법·입력 오류 | 2 | 불가 |
| `3000`대 | **판정** (불일치·검증 실패·0건·다건) | 3 | 불가 |
| `4000`대 | **정책** (금지·신뢰 위반·능력 미부여) | 4 | 불가 |

전체 코드표는 기계 정본의 `errorCodes` 가 정본이다. 대표적으로
`3001 VALIDATION_FAILED` 는 COMMIT 을 막고, `2003 DOCUMENT_ENCRYPTED` 는
**우회하지 않는다**는 뜻이며, `4001 TRUST_VIOLATION` 은 비신뢰 값이 정책·능력·
연산명 자리에 오려 했다는 뜻이다.

## 신뢰 모델

문서에서 온 문자열은 **데이터이지 지시가 아니다.** 그 값이 결과 봉투에 실리면
`untrustedContent: true` 와 `untrustedFields` 로 경로를 표시한다 — rhwp 의
[출처 표지 계약](../envelope_provenance.md)이 이미 하는 일이고, DAP 는 그것을
프로토콜 필수 필드로 승격한다.

## 준수

```bash
python3 tools/dar/conformance.py --bin target/release/rhwp --protocol dap
```

검사기는 rhwp 를 **실제로 실행해** 각 요구를 판정하고, 미달 항목을 그대로
보고한다. 이 문서가 희망이 아니라 반증 가능한 진술이 되게 하는 장치다 —
미달 목록이 곧 다음 구현 목록이다.
