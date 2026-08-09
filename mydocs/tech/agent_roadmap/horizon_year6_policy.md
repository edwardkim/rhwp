---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/horizon_year6_policy.md
last_verified: 2026-08-10
---

# 6년 선행 축 — 정책 게이트: 판정의 기계화

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) →
  5년 앵커([#4448](https://github.com/edwardkim/rhwp/issues/4448)) → **이 문서**
- 이슈: [#4449](https://github.com/edwardkim/rhwp/issues/4449)
- 등급: **[지평]** — 코드 0줄.
- 착수 조건: 5년 축 머지(anchored 조건을 규칙에 쓰려면) — 최소 4년 축 머지.

## 1. 질문 — 데이터는 쌓였는데 판정은 수동이다

1~5년 축이 완성되면 캡슐 하나에 다음 판정 데이터가 붙는다: 재현 여부
(reproduced), 재현율(reproducedRate), 계보 유효(valid), 링크별
parentOk/lineageOk, 서명(signerOk·keyId), 앵커(anchoredOk). 그러나 "이
산출물을 우리 파이프라인에 **반입해도 되는가**"는 여전히 사람이 봉투를 읽고
정하는 일이다. 반입 건수가 하루 수백이 되는 순간 이 구조는 무너진다 — 읽지
않고 통과시키거나, 병목이 되거나.

한 줄 정의: **게이트 축은 "무엇을 받아들일 것인가"를 정책 파일(데이터)로
선언하고, 판정을 기계에 넘긴다.** 검증 축들이 만든 데이터의 소비처를 만드는
축이며, 사다리에서 처음으로 "증명"이 아니라 "결정"을 다룬다.

## 2. 설계

### 2.1 정책 파일 — 선언형, 조건은 화이트리스트 연산만

```json
{
  "schemaVersion": "1.0",
  "kind": "admissionPolicy",
  "name": "재무팀 반입 정책 v3",
  "defaultVerdict": "deny",
  "rules": [
    { "id": "R1-재현강제", "require": { "reproduced": { "eq": true } } },
    { "id": "R2-계보유효", "require": { "lineageValid": { "eq": true } } },
    { "id": "R3-서명원", "require": { "signerKeyId": { "in": ["org.example/agent-7#2026"] } } },
    { "id": "R4-재현율", "require": { "reproducedRate": { "gte": 0.98 } } },
    { "id": "R5-깊이", "require": { "lineageDepth": { "lte": 50 } } },
    { "id": "R6-앵커", "require": { "anchoredOk": { "eq": true } } }
  ]
}
```

- 연산자는 `eq`·`in`·`gte`·`lte` **4개로 고정**한다. 표현식 언어(스크립트·정규식
  ·산술)를 넣지 않는 것이 설계 결정이다: 정책은 감사 대상 문서이고, 튜링 완전한
  정책은 감사 불가능한 정책이다. 부족한 표현력은 규칙을 늘려 해결한다.
- `defaultVerdict: "deny"` — 명시 허용 없으면 거부가 기본값. allow 기본은
  선택 가능하되 봉투에 경고 표기.
- 조건의 좌변(judgment key)은 1~5년 축 봉투 필드의 **고정 사전**에서만 고른다
  (미지 키는 정책 로드 시점에 exit 2 — 오타가 조용히 항상-참이 되는 사고를
  스키마가 막는다).

### 2.2 게이트 명령

```text
rhwp gate <캡슐.json | 캡슐 폴더> --policy policy.json
          [--keyring k.json] [--anchor-log a.ndjson] [--deep] [--json]
```

봉투:

```json
{
  "schemaVersion": "1.0",
  "policy": "재무팀 반입 정책 v3",
  "target": "d.capsule.json",
  "verdict": "deny",
  "violations": [
    { "rule": "R4-재현율", "expected": { "gte": 0.98 }, "actual": 0.95 }
  ],
  "evaluated": 6
}
```

- **판정 재료는 게이트가 직접 재계산한다** — 캡슐에 적힌 자기 신고를 읽지
  않고 lineage/audit/서명/앵커 검증을 호출해 얻은 값만 쓴다. 신고 기반
  게이트는 게이트가 아니다.
- exit: 통과 0 / IO 1 / 정책·사용법 오류 2 / 거부 3. 거부는 오류가 아니라
  데이터다(violations[]) — 이 저장소의 exit 3 규약 그대로.
- 폴더 대상이면 전건 판정 + 요약(passed/denied 계수) — audit 와 동형.

### 2.3 정책 자체의 신뢰 — 4년 축 재사용

정책 파일이 바꿔치기되면 게이트는 무력하다. 해결은 새 메커니즘이 아니라
재사용이다: 정책 파일을 4년 축 서명 대상으로 삼고(`policy.json.sig.json`),
게이트에 `--policy-keyring` 을 주면 **정책 서명 검증을 선행**한다. 미서명
정책은 봉투에 `policySigned: false` 경고를 싣는다.

### 2.4 TOCTOU — 검증 시점과 사용 시점

게이트 통과 후 산출물이 바뀌면? 게이트 봉투에 `targetSha256` 을 고정 기재한다
— 소비 측은 반입 직전 해시 재대조만 하면 된다(1해시 비교, 재검증 불필요).
`--deep` 옵션은 게이트 시점에 재실행 재현까지 다시 밟는 최강 모드다(비용
회계는 3년 축과 동일 — 고유 planSha256 캐시).

## 3. 위협 모델

| # | 공격 | 판정 |
|---|---|---|
| G1 | 정책 파일 바꿔치기 | 정책 서명(§2.3) — 미서명 정책은 경고 표기 |
| G2 | 게이트 우회(직접 반입) | **도구 밖** — 게이트를 반입 경로에 강제 배치하는 것은 운영·CI 구성의 몫임을 명시 |
| G3 | 통과 후 바꿔치기 | targetSha256 재대조(§2.4) |
| G4 | 자기 신고 캡슐로 기만 | 게이트는 신고를 읽지 않는다(§2.2 재계산 원칙) |
| G5 | 정책 오타로 전건 통과 | 미지 키 exit 2 + deny 기본값 — 이중 방어 |

## 4. 단계와 DoD

| 단계 | 내용 | DoD |
|---|---|---|
| Y6-M1 | 정책 스키마+단건 게이트 | 통과/거부/violations 3형 테스트 + 미지 키 exit 2 + deny 기본 |
| Y6-M2 | 폴더 게이트+재계산 원칙 | 신고 위조 캡슐(내용은 거짓, 재계산은 진실) 픽스처가 재계산 값으로 판정됨을 고정 |
| Y6-M3 | 정책 서명+등재 7표면 | 정책 변조 검출 + 3년 축 동일 게이트 전건 green |

## 5. 정직 조항

- 게이트는 반입 결정을 **표현**하는 도구지 **강제**하는 도구가 아니다 — 강제는
  배치(G2)의 문제고, 그것은 조직 운영의 영역이다.
- 연산자 4개 고정은 표현력의 손해를 감수한 감사 가능성 우선 결정이다. 실수요가
  이를 깨는 날(예: 시간 조건) 확장은 새 연산자 추가가 아니라 판정 키 추가로
  먼저 시도한다.
