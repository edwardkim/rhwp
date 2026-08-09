---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/horizon_year8_disclosure.md
last_verified: 2026-08-10
---

# 8년 선행 축 — 선택적 공개: 증명은 공개, 내용은 비밀

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) →
  7년 연합([#4450](https://github.com/edwardkim/rhwp/issues/4450)) → **이 문서**
- 이슈: [#4451](https://github.com/edwardkim/rhwp/issues/4451)
- 등급: **[지평]** — 코드 0줄.
- 착수 조건: 7년 축 머지.

## 1. 질문 — 계보를 공개하면 내용이 샌다

연합(7년)으로 캡슐이 도메인을 넘기 시작하면 즉시 부딪히는 현실: 기업 문서의
**내용은 비밀**이다. 그런데 현행 캡슐은 검증 가능성을 위해 `plan` 을 통째로
싣고, 계획서에는 편집 문자열이 평문으로 들어 있다 — `replace_text` 의
find/replace, `fill_fields` 의 값들, 파일 경로(조직 구조 누설)까지. "계보를
증명해 달라"는 요구와 "내용을 보이지 말라"는 요구가 정면 충돌한다.

관찰: 사다리의 판정 축들은 이미 대부분 **해시만으로** 동작한다 — parentOk·
lineageOk·anchoredOk·signerOk 는 내용을 안 본다. 내용이 필요한 것은 `--deep`
재실행뿐이다. 즉 구조적으로 "공개용 검증"과 "내용 보유자용 검증"을 분리할 수
있다.

한 줄 정의: **선택적 공개 축은 캡슐을 '커밋(해시)'과 '개봉(내용+salt)'으로
분리해, 공개 범위를 필드 단위로 발급자가 선택하게 만든다.**

## 2. 설계

### 2.1 가림 캡슐(redacted capsule) — 필드별 salt 커밋

발급 시 `--redact-plan` 을 주면 plan 의 지정 필드가 커밋으로 치환된다:

```json
{
  "schemaVersion": "1.2",
  "kind": "workCapsule",
  "planRedacted": true,
  "plan": {
    "planVersion": "1.0",
    "input": { "committed": "sha256(경로바이트‖salt₁)" },
    "output": { "committed": "…" },
    "steps": [ { "action": "replace_text",
                 "find": { "committed": "sha256(값‖salt₂)" },
                 "replace": { "committed": "…" } } ]
  },
  "receipt": { "…": "해시 3종은 원래부터 공개 안전" }
}
```

별도 파일 `<캡슐>.opening.json` (비공개 보관):

```json
{ "kind": "capsuleOpening", "capsuleSha256": "…",
  "openings": { "/plan/steps/0/find": { "value": "원문", "salt": "32바이트 hex" } } }
```

- **salt 는 필드마다 독립·필수**다. 커밋이 `sha256(값)` 뿐이면 저엔트로피
  필드(경로·짧은 문구)는 사전 공격으로 전수 복원된다 — salt 가 유일한 방어이고,
  도구가 생성을 강제한다(수동 salt 금지).
- 구조(steps 개수·action 이름)는 공개로 남긴다 — "무슨 종류의 작업 몇 건"은
  계보의 골격이라 가리면 검증 의미가 사라진다. 골격 공개조차 곤란한 시나리오는
  범위 밖으로 명시(§5).

### 2.2 부분 개봉 — 필드 단위 증명

분쟁·감사에서 특정 필드만 증명한다: `/plan/steps/0/find` 의 (값, salt) 를
건네면 상대는 `sha256(값‖salt) == committed` 한 번으로 그 필드만 확인한다.
나머지 필드는 계속 비밀이다. 개봉 검증 명령:

```text
rhwp disclose verify <가림캡슐> --opening partial-opening.json --json
  → {verifiedFields: ["/plan/steps/0/find"], mismatched: [], unopened: 7}
```

### 2.3 검증 의미론의 분리 (정확하게)

| 검증 | 공개 번들만으로 | 개봉 보유 시 |
|---|---|---|
| 무결·계보·서명·앵커 (해시 축) | **가능** — 원래 해시만 쓴다 | 가능 |
| `--deep` 재실행 재현 | **불가능** — 계획 원문이 없다 | 가능 (전체 opening 으로 plan 복원 → 기존 replay 그대로) |

가림 캡슐의 `planSha256` 은 **가려진 plan 바이트**의 해시다(공개 검증의 대상).
전체 opening 으로 복원한 원문 plan 의 해시는 `originalPlanSha256` 로 별도
기재 — 두 해시의 역할 분리가 이 설계의 급소이며, 혼동하면 재현 검증이 영영
안 맞는 캡슐이 나온다.

## 3. 위협 모델

| # | 공격 | 판정 |
|---|---|---|
| D1 | 사전·무차별 대입 (커밋에서 원문 복원) | 필드별 강제 salt (§2.1) |
| D2 | 개봉 위조 (다른 값+짜맞춘 salt) | sha256 충돌 저항 — 계산적으로 불가 |
| D3 | 선택 개봉의 오도 (유리한 필드만 공개) | **못 막는다 — 설계의 본질이다.** 개봉 범위가 협상 대상임을 봉투(unopened 계수)가 항상 드러낸다는 것까지가 도구의 몫 |
| D4 | opening 파일 유출 | 유출 = 그 필드 공개와 동치. 보관 책임은 발급자(운영) — 도구는 opening 을 캡슐과 분리 발급하는 것까지 |
| D5 | 구조 자체의 누설 (steps 개수 등) | 범위 밖 명시(§5) |

## 4. 단계와 DoD

| 단계 | 내용 | DoD |
|---|---|---|
| Y8-M1 | 가림 발급+개봉 검증 | 왕복 + 잘못된 salt 검출 + salt 재사용 거부 테스트 |
| Y8-M2 | 전체 개봉 → deep 재현 | 가림 캡슐 + 전체 opening 으로 reproduced:true 왕복 (originalPlanSha256 정합) |
| Y8-M3 | 번들 결합(7년 축) | 공개 번들(opening 없음)이 해시 축 전건 통과 + deep 은 "판정 불가"로 정직 보고 |

## 5. 정직 조항

- ZK(영지식) 증명 — "내용도 구조도 안 보이면서 재현까지 증명" — 은 명시적으로
  이 축의 범위 밖이다. 필요 기술(범용 ZK-VM)의 성숙을 기다리는 것이 정직하고,
  salt 커밋+부분 개봉은 그때까지의 실용 해법이다.
- 구조 골격(§2.1)은 공개된다. 골격조차 비밀인 요구는 이 설계로 풀 수 없다.
- D3 은 결함이 아니라 속성이다 — 선택 공개는 언제나 발급자에게 유리한 공개다.
  수신자의 방어는 "더 열어라"고 요구하는 것이고, 그 협상을 지원하는 것(무엇이
  안 열렸는지 셈해 보이는 것)까지가 도구의 정직한 경계다.
