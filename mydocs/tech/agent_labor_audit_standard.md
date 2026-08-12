---
kind: guide
status: active
canonical: mydocs/tech/agent_labor_audit_standard.md
last_verified: 2026-08-11
---

# 에이전트 노동 감사 표준 (초안 v0.1) — 보고 형식·리콜 절차·적합성 등급

- 좌표: [#4558](https://github.com/edwardkim/rhwp/issues/4558) (10년 축 착공) ·
  설계서 [`horizon_year10_standard.md`](agent_roadmap/horizon_year10_standard.md)
- 참조 구현: `rhwp audit-report` · `rhwp recall-scope` · `rhwp conformance`
  (계약: `tests/audit_standard_contract.rs`)
- 지위: **초안** — 외부 표준 제출은 이 문서의 범위 밖이다(채택 전략은 가설이며
  생태계 실측 없이 사실로 승격하지 않는다 — 설계서 §5).

## 0. 이 문서가 답하는 질문

"이 산출물은 어떤 에이전트가 어떤 과정으로 만들었고, 그것을 **제3자가 기계로**
확인할 수 있는가." 규제·조달·감사가 이 질문을 요구하는 날, 필요한 것은 새
이론이 아니라 **보고 형식·리콜 절차·등급 언어**와 그것을 채울 수 있는 참조
구현이다. 이 문서는 rhwp 검증 사다리(1~9년 축)를 감사인이 읽는 언어로 묶는다.

## 1. 용어 사전 (규범적)

표준의 절반은 낱말이다. 각 용어의 정의·불변식·해당 산출물을 고정한다.

| 용어 | kind | 정의 | 핵심 불변식 |
|---|---|---|---|
| 영수증 (receipt) | (캡슐 내장) | 작업 1건의 3해시 기록 — 입력·산출·계획 | `inputSha256`·`outputSha256`·`planSha256` 은 파일/텍스트 **바이트** 기준 |
| 작업 캡슐 (work capsule) | `workCapsule` | 계획 원문(planText)과 영수증·부모 링크를 담은 재현 가능 단위 | `plan == parse(planText)` · `sha256(planText) == receipt.planSha256` |
| 계보 (lineage) | (캡슐 `parent`) | 캡슐의 부모 사슬 — 납품 이력 | 부모 파일 해시 일치 + **부모 산출 == 자식 입력** |
| 서명 (signature) | `capsuleSignature` | 캡슐 **파일 바이트**에 대한 Ed25519 분리 사이드카 | 정규화 없음 — 바이트가 서명 대상 |
| 키 등록부 (keyring) | `keyring` | keyId → 공개키·폐기 기록 | 수신 경로가 신뢰 뿌리(동봉 keyring 불신) |
| 앵커 로그 (anchor log) | `anchorLog` | append-only 줄 해시 체인 등재부 | `prevEntryHash` = 직전 **줄 바이트** 해시 · seq 연번 |
| 체크포인트 (checkpoint) | `anchorCheckpoint` | 로그 머클 루트의 외부 공표물 | 공표 이후의 역사 재작성은 루트와 충돌 |
| 게이트 판정 (gate verdict) | (봉투) | 반입 정책(admissionPolicy)에 대한 기계 검수 | 판정 재료 미지정은 통과가 아니라 위반(deny 기본) |
| 연합 번들 (lineage bundle) | `lineageBundle` | 조상 폐쇄집합+서명+증명의 오프라인 교환 컨테이너 | 서명 판정 기준은 수신자 보유 trust-domain 뿐 |
| 개봉 (opening) | `capsuleOpening` | 가림 캡슐의 값·salt·원문 보관물(비밀) | 전체 개봉 복원은 **바이트 동일** — 원본 서명이 그대로 valid |
| 원장 (ledger) | `settlementLedger` | 정산 청구의 append-only 등재부 (앵커 로그 동형) | capsuleSha256 는 전역에서 한 번만 accepted |
| 감사 보고서 | `agentLaborAuditReport` | 아래 §2 의 표준 보고 | **보고서를 감사할 수 있다** — 전 수치 재계산 가능 + 보고서 자체 서명 |

## 2. 감사 보고 표준 — `agentLaborAuditReport`

### 2.1 스키마 (기계)

```json
{
  "schemaVersion": "1.0",
  "kind": "agentLaborAuditReport",
  "scope": { "root": "<감사 대상 폴더>", "capsules": 412 },
  "reproduction": { "attempted": 412, "reproduced": 409, "rate": 0.9927,
                    "failures": [ { "capsule": "…", "reason": "…" } ] },
  "lineage": { "graphs": 37, "heads": 41, "valid": 40,
               "broken": [ { "head": "…", "brokenAt": "…" } ] },
  "attribution": { "signed": 400, "unsigned": 12,
                   "validSignatures": 400, "revokedKeyUses": 0 },
  "anchoring": { "anchored": 412, "unanchored": 0 },
  "gate": { "policySha256": "…", "passed": 405, "denied": 7 },
  "toolVersions": { "rhwp": ["0.8.2"], "mixed": false },
  "auditor": { "keyId": "auditor.example#2026" }
}
```

### 2.2 절별 산출 규정 (규범적)

| 절 | 산출 방법 | opt-in 재료 | `null` 의 뜻 |
|---|---|---|---|
| `scope` | `*.capsule.json` 비재귀 수집 | — | — |
| `reproduction` | 캡슐마다 계획 재실행 → 3해시 대조 (replay 코어와 동일 경로) | `--deep` | 재현 미수행(비용 사유) — **수치 부재를 0 이나 100% 로 위장하지 않는다** |
| `lineage` | 머리(자식 없는 노드)별 조상 사슬 걷기 · graphs = 뿌리 수 | — | — |
| `attribution` | 캡슐 파일 바이트 × 사이드카 × keyring 판정 | `--keyring` | 귀속 축 미검사 |
| `anchoring` | 캡슐 파일 해시 ∈ 로그 등재 집합 | `--anchor-log` | 앵커 축 미검사 |
| `gate` | 캡슐별 판정 재료(위 절들의 재사용)로 정책 평가 | `--policy` | 게이트 축 미검사 |
| `toolVersions` | 영수증 기록 합산 — 미기록은 `"미기록"` | — | — |

**요건 R1 (재계산 가능성)**: 보고서의 모든 수치는 대상 폴더와 같은 재료(키링·
로그·정책)만으로 제3자가 재계산해 일치를 확인할 수 있어야 한다.

**요건 R2 (보고서의 감사 가능성)**: 보고서 파일 자체가 4년 축 분리 서명의
대상이다. 보고서 1바이트 변조는 `verify-signature` 가 invalid 로 폭로한다.

**요건 R3 (부재의 정직)**: 검사하지 않은 축은 `null` 로 보고한다 — 미검사를
"이상 없음"으로 위장하는 보고서는 이 표준의 부적합이다.

### 2.3 인간 서식 (권고)

보고서 상단에 다음 네 줄을 산문으로 붙인다: ① 감사 대상과 기간, ② 재현율과
실패 목록 요지, ③ 귀속·앵커·게이트의 미달 건수와 사유, ④ 감사인 서명 키와
보고서 해시. 산문과 기계 절이 다르면 **기계 절이 정본**이다.

## 3. 오염 리콜 절차 — recall-scope

### 3.1 절차 (규범적)

1. **오염 확정**: 오염 노드를 캡슐 경로 또는 파일 sha256 으로 지목한다
   (해시가 정체성 — 경로 표기가 달라도 같은 파일이면 같은 노드다).
2. **범위 계산**: `rhwp recall-scope --contaminated <노드> --among <폴더>` —
   후손 폐쇄집합(오염 노드 자신 포함 — 회수 1호)과 미영향 계수를 얻는다.
   각 영향 캡슐에는 오염 노드부터 자신까지의 경로가 붙는다(왜 회수인가의 증명).
3. **회계 연결**: 원장이 있으면 `--ledger` 로 영향 캡슐의 정산 청구 좌표
   (seq·claimSha256·verdict)까지 짚는다 — 환불·재작업의 회계 근거.
4. **재작업**: 오염 노드의 입력을 교정한 뒤 후손을 같은 계획으로 재실행한다
   (캡슐의 planText 가 재작업 지시서다). 재작업 산출은 **새 캡슐**이다 —
   기존 캡슐의 정정·삭제가 아니다(append-only 원칙).

### 3.2 리콜 보고 서식 (권고)

무엇이(오염 노드 해시) · 왜(오염 사유) · 어디까지(affected 목록+경로) ·
조치(재작업 캡슐 목록 + 원장 조치). affected 의 근거는 항상 기계 산출이다.

## 4. 적합성 등급 — L1~L5

| 등급 | 요건 (누적) | 기계 검사 | 대응 축 |
|---|---|---|---|
| L1 | 산출물마다 영수증 | 전 캡슐 receipt 3해시 존재 | 1년 |
| L2 | 감사 가능 + 계보 기록 | 계획 정합(`plan==parse(planText)`) + 조상 사슬 전건 유효 (+`--deep` 재현) | 2·3년 |
| L3 | 서명 귀속 + 앵커 운영 | 전 캡슐 서명 valid + 로그 등재 | 4·5년 |
| L4 | 게이트 상시 배치 | 전 캡슐 정책 통과 (판정 재료는 L2·L3 재사용) | 6년 |
| L5 | 공개 운영 + 정산 원장 | 원장 체인 무결·기입 존재 · **공개 "운영"은 기계 판정 밖(수동 확인 항목으로 명시)** | 8·9년 |

- 등급이 요구하는 재료(keyring·로그·정책·원장)를 주지 않으면 판정이 아니라
  **사용법 오류**다 — "재료 없이 통과"라는 우회로를 봉쇄한다.
- 검사 항목별 판정은 `checks[{id, ok, detail}]` 로 보고한다. `ok: null` 은
  기계 판정 밖(수동 확인) 의 정직 표기다.
- 참조 구현: `rhwp conformance <폴더> --level L1..L5` — 미달 exit 3.

## 5. 이 표준이 하지 않는 것 (정직 조항)

1. **지불·회계 규정 준수 판정** — 원장은 근거이지 잔액이 아니다(9년 축 범위
   경계 그대로).
2. **오염의 판정** — 무엇이 오염인가(저작권 침해·훼손·잘못된 원본)는 사람의
   판단이다. 표준은 "오염이 확정된 뒤의 범위 계산"만 기계화한다.
3. **조직 절차의 검사** — L5 의 "선택적 공개 운영" 같은 절차 요건은 기계 판정
   밖임을 명시하고 수동 확인 항목으로 남긴다.
4. **외부 표준과의 정합** — SBOM/AIBOM 계열(SPDX·CycloneDX)에의 확장 제안은
   생태계 실측 후의 일이다. 이 문서는 그 실측 전의 참조 구현 기술서다.

## 6. 부록 — 최소 실행 예

```bash
# 감사 보고 (전 축)
rhwp audit-report capsules/ -o report.json \
  --deep --keyring keyring.json --anchor-log anchor.ndjson --policy policy.json \
  --sign-key auditor.key.json

# 보고서의 감사
rhwp verify-signature report.json --keyring keyring.json

# 리콜 범위 (회계 연결 포함)
rhwp recall-scope --contaminated bad.capsule.json --among capsules/ --ledger ledger.ndjson

# 적합성 자가진단
rhwp conformance capsules/ --level L3 --keyring keyring.json --anchor-log anchor.ndjson
```
