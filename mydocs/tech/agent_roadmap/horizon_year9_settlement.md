---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/horizon_year9_settlement.md
last_verified: 2026-08-10
---

# 9년 선행 축 — 정산 증빙: 검증된 노동의 회계 원장

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) →
  8년 공개([#4451](https://github.com/edwardkim/rhwp/issues/4451)) → **이 문서**
- 이슈: [#4452](https://github.com/edwardkim/rhwp/issues/4452)
- 등급: **[지평]** — 코드 0줄.
- 착수 조건: 8년 축 머지.
- **범위 경계(가장 먼저)**: 이 축은 돈을 움직이지 않는다. 지불·송금·잔액은
  전부 외부 시스템의 몫이고, 이 축의 산출물은 **"지불 근거가 되는, 제3자
  검증 가능한 증빙"** 뿐이다. 금융 실행 기능은 설계에도 구현에도 넣지 않는다.

## 1. 질문 — 검증 통과가 지불 조건이 되는 날

에이전트 노동이 외주·시장의 형태를 갖추면(7·8년 축이 그 유통을 깔면) 정산
분쟁의 구조가 보인다: "일을 했다/안 했다", "요구 품질이다/아니다", "이미
청구했다/중복이다". 사람 노동은 이 분쟁을 계약서·검수서·세금계산서로 다루지만
전부 **자기 신고 문서**다. 에이전트 노동은 더 나은 것을 가질 수 있다 — 작업
자체가 재현 검증 가능한 캡슐이므로, **검수를 기계 판정으로, 청구를 해시
고정으로** 만들 수 있다.

한 줄 정의: **정산 축은 (요구 명세 → 캡슐 → 게이트 판정 → 원장 기입)을 전부
검증 가능한 데이터로 잇는다.** 새 검증 메커니즘은 하나도 발명하지 않는다 —
6년 게이트가 검수이고, 3년 계보가 납품 이력이고, 해시 체인 원장은 5년 로그의
동형 재사용이다. 이 축의 기여는 **연결 형식**이다.

## 2. 설계

### 2.1 작업 명세서(workorder) — 요구를 선언하는 쪽

```json
{
  "schemaVersion": "1.0",
  "kind": "workorder",
  "workorderId": "wo-2026-0142",
  "title": "표기용",
  "acceptancePolicy": { "…": "6년 축 admissionPolicy 인라인 — 검수 기준 그 자체" },
  "unitPrice": { "amount": "50000", "currency": "KRW", "per": "capsule" },
  "deadline": "2026-09-01T00:00:00Z",
  "orderer": { "keyId": "buyer.example/po#2026" }
}
```

- **검수 기준 = 정책 파일.** "요구 품질"을 산문이 아니라 6년 축 정책으로
  선언하므로, 검수 분쟁은 "게이트를 통과했는가"라는 기계 판정으로 환원된다.
- workorder 는 발주자가 4년 축으로 서명한다 — 사후 "그런 조건 아니었다"를
  막는 것은 서명된 명세서다.
- 금액 필드는 문자열이다(부동소수점 금액 금지 — 회계 상식) — 도구는 이 값을
  계산하지 않고 **운반**만 한다(범위 경계).

### 2.2 정산 증빙(settlement claim) — 청구하는 쪽

```json
{
  "schemaVersion": "1.0",
  "kind": "settlementClaim",
  "workorderSha256": "…",
  "capsuleSha256": "…",
  "gateEnvelopeSha256": "…",
  "claimant": { "keyId": "vendor.example/agent-7#2026" },
  "claimedAt": "…"
}
```

`rhwp settle propose` 가 발급하고 청구자가 서명한다. 세 해시가 급소다:
**명세서·캡슐·게이트 봉투를 전부 해시로 고정**하므로, 청구 후에 산출물을
바꾸거나(캡슐 해시 불일치), 다른 명세서에 갖다 붙이거나(명세서 해시 불일치),
게이트 판정을 위조(봉투 해시 불일치)할 수 없다.

### 2.3 원장(ledger) — 5년 로그의 동형 재사용

```json
{"seq":0,"kind":"settlementLedger","claimSha256":"…","verdict":"accepted","prevEntryHash":null,"at":"…"}
```

- append-only ndjson 해시 체인 — 5년 축 anchor 로그와 **같은 형식, 같은 검증
  코드 경로**. verdict 는 accepted/rejected(+ 게이트 violations 참조).
- **이중 청구 검사**: 같은 capsuleSha256 의 accepted 가 이미 있으면 신규
  기입은 exit 3 (duplicate). 같은 캡슐을 다른 workorder 로 청구하는 것도
  잡힌다 — 원장 전역에서 capsuleSha256 는 한 번만 accepted 된다.

### 2.4 검증 명령

```text
rhwp settle propose --workorder wo.json --capsule c.json --gate-envelope g.json -o claim.json
rhwp settle verify claim.json --ledger ledger.ndjson --keyring k.json --json
  → {workorderOk, capsuleOk, gateOk, signerOk, duplicate: false, verdict}
```

검증자는 세 해시 대조 + 서명 2건(명세서·청구) + 게이트 봉투의 verdict 재확인 +
원장 중복 검사를 밟는다. 전부 오프라인 파일 검증 — 이 사다리의 결 그대로.

## 3. 위협 모델

| # | 공격 | 판정 |
|---|---|---|
| P1 | 청구 후 산출물 바꿔치기 | capsuleSha256 고정 |
| P2 | 게이트 봉투 위조 | gateEnvelopeSha256 + 검증자의 게이트 재실행 옵션(--deep 재검수) |
| P3 | 이중 청구 | 원장 전역 중복 검사 (§2.3) |
| P4 | 명세서 사후 변경 | workorder 서명 + 해시 고정 |
| P5 | 원장 재작성 | 5년 축과 동일 — 체크포인트 공표가 방어선 (동형 재사용의 이득) |
| P6 | 실제 지불과 원장의 괴리 | **범위 밖** — 원장은 근거이지 잔액이 아니다. 지불 시스템과의 대사는 회계 운영의 몫임을 명시 |

## 4. 단계와 DoD

| 단계 | 내용 | DoD |
|---|---|---|
| Y9-M1 | workorder·claim 형식+propose/verify | 3해시 고정 왕복 + P1/P4 변조 검출 |
| Y9-M2 | 원장+이중 청구 | P3 픽스처(같은 캡슐 2회 청구 → 두 번째 exit 3) |
| Y9-M3 | 재검수(--deep)+등재 | 게이트 재실행 판정 일치 + 7표면 green |

## 5. 정직 조항

- 이 축의 전제(에이전트 노동 시장의 형성)는 아직 실측이 아니라 전망이다 —
  사다리에서 가장 투기적인 축이며, 그래서 9년이다. 착수는 수요 실측이 결정한다.
- 가격·세무·회계 규정 준수는 도구 밖이다. 도구는 "무엇이 언제 검수를 통과해
  청구되었나"의 사실만 고정한다.
- 지불 실행을 넣지 않는 것은 기술 한계가 아니라 **설계 원칙**이다 — 검증
  도구와 금융 도구의 결합은 실패 시 피해 반경이 다르다.
