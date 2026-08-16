---
kind: reference
status: active
canonical: mydocs/tech/standards/document_transaction_protocol.md
last_verified: 2026-08-16
---

# 문서 트랜잭션 프로토콜 (Document Agent Transaction Protocol, DATP) 1.0

> 기계용 정본은 [`document_transaction_protocol.json`](document_transaction_protocol.json)이다.
> 봉투·오류 코드는 [DAP/1.0](document_agent_protocol.md), 아키텍처 전체는
> [DAR 척추](document_agent_runtime.md).

## 한 문장

**원자성·결정적 재실행·불변 영수증**을 갖는 문서 변경 상태기계.

## rhwp 는 이것을 이미 하고 있다 — 이름이 없었을 뿐

이 프로토콜은 발명이 아니라 형식화다. 실측:

- `run` — 선언적 편집 계획을 **정적 선검증 → 원자 실행 → 저널**로 처리한다.
- `replay` — 계획을 재실행해 **입력·계획·산출 SHA-256 3해시** 영수증을 발급하고,
  `--expect-output-sha256` 으로 타인의 작업 주장을 재현 검증한다. **불일치는 exit 3.**
- `--dry-run`·`--verify` — 변경 전 판정과 산출 자기검증.
- `lineage`·`anchor`·`audit`·`conformance`·`settle` — 계보·봉인·감사·정산
  ([AWS/1.0](agent_work_standard.md) AW-L1~L5).

DATP/1.0 은 이것들에 **상태기계와 불변식**을 부여해, 에이전트가 순서를 지켰는지
기계로 판정할 수 있게 한다.

## 상태기계

```
BEGIN ──▶ READ ─┐
  │             ├──▶ SELECT ──▶ PROPOSE ──▶ MODIFY ──▶ VALIDATE ──▶ DIFF ──▶ COMMIT ──▶ 영수증
  │             │                  ▲                      │           │
  └─────────────┴──────────────────┴──────────────────────┘           │
                                   (검증 실패 시 되돌아감)             │
  ROLLBACK ◀──────────────────────────────────────────────────────────┘ (언제든)
```

### 불변식 (이 넷이 프로토콜의 전부다)

1. **COMMIT 은 직전에 성공한 VALIDATE 가 있을 때만 허용된다.** MODIFY 직후
   COMMIT 은 프로토콜 위반이다.
2. **VALIDATE 가 `3001` 을 내면 그 트랜잭션은 COMMIT 으로 갈 수 없다.** PROPOSE 로
   돌아가거나 ROLLBACK 한다.
3. **MODIFY 는 원본을 바꾸지 않는다** — 산출은 항상 분리된다. 그래서 ROLLBACK 이
   값싸다(되돌릴 것이 없다).
4. 한 트랜잭션의 모든 연산은 **같은 `transaction_id` 와 같은 입력 문서 해시**를
   참조한다.

## 연산과 rhwp 대응

| 연산 | 변경? | rhwp 표면 |
|---|---|---|
| `BEGIN` | — | 계획서의 `input` 고정 |
| `READ` | — | `info` · `export-*` |
| `SELECT` | — | `search` · `fields` · `export-tables` (좌표를 준다) |
| `PROPOSE` | — | 계획(plan) 작성 |
| `MODIFY` | ✓ | `edit` 계열 `-o` 산출 분리 |
| `VALIDATE` | — | `--verify` · `--dry-run` · 정적 선검증 |
| `DIFF` | — | `ir-diff` · `render-diff` |
| `COMMIT` | ✓ | `run` 원자 실행 + `replay --capsule` |
| `ROLLBACK` | — | 산출 폐기 |
| `REPLAY` | — | `replay --expect-output-sha256` |
| `VERIFY` | — | `verify-signature` · `lineage` · `audit` · `conformance` |

`SELECT` 가 0건이면 `3002`, 하나를 요구했는데 여럿이면 `3003` 이다 — **둘 다
오류가 아니라 판정이다.**

## 영수증

필수: `transactionId` · `parentTransactionId` · `inputSha256` · `operationSha256` ·
`outputSha256` · `policySha256` · `toolVersion` · `agentIdentity` · `timestamp`.
선택: `signature` · `anchorEpoch`.

- **불변이다.** 정정은 새 트랜잭션으로 하고 `parentTransactionId` 로 잇는다.
- `operationSha256` 은 계획의 해시다 — 계획이 다르면 다른 트랜잭션이다.
- `policySha256` 은 COMMIT 시점 정책의 해시다. 정책이 바뀌어도 **과거 COMMIT 의
  근거를 소급 변경할 수 없다.**
- **결정적 재실행**: 같은 `input`+`operation`+`toolVersion` 이면 같은
  `outputSha256` 이어야 한다. 아니면 REPLAY 가 `3000` 을 낸다.

## 원자성

- COMMIT 은 전부 적용되거나 전부 적용되지 않는다.
- 실패한 트랜잭션의 임시 산출은 **지운다** — "성공처럼 보이는 미완성 산출물"을
  남기지 않는다.
- 여러 변경을 한 트랜잭션으로 묶으려면 PROPOSE 에 전부 담는다. MODIFY 를 나눠
  부르지 않는다.

## 준수

```bash
python3 tools/dar/conformance.py --bin target/release/rhwp --protocol datp
```
