---
kind: guide
status: active
canonical: mydocs/tech/policy_gate_guide.md
last_verified: 2026-08-11
---

# 게이트 운영 대전서 — 반입 정책의 작성·배치·감사

- 좌표: 설계서 `mydocs/tech/agent_roadmap/horizon_year6_policy.md`([지평]) 의
  구현 1호 — 이슈 [#4545](https://github.com/edwardkim/rhwp/issues/4545).
- 검증 원장: `tests/gate_contract.rs` — 이 문서의 모든 단언은 그 테스트가
  실측 고정한다. 문서와 테스트가 다르면 테스트가 맞다.

## 0. 한 문단 요약

1~5년 축이 만든 판정 데이터(재현·계보·서명·앵커)를 사람이 매번 읽으면
반입이 스케일하지 않는다. `rhwp gate` 는 **정책 파일(데이터)** 을 캡슐에
적용해 allow/deny 를 기계로 판정한다. 원칙 셋: 판정 재료는 자기 신고가
아니라 **재계산**, 정책 언어는 **연산자 4개로 고정**(감사 가능성 우선),
모르는 것은 통과시키지 않는다(**deny 기본·unavailable 위반**).

## 1. 정책 파일 작성법

```json
{
  "kind": "admissionPolicy",
  "name": "재무팀 반입 정책 v1",
  "defaultVerdict": "deny",
  "rules": [
    { "id": "R1-재현",  "require": { "reproduced":   { "eq": true } } },
    { "id": "R2-계보",  "require": { "lineageValid": { "eq": true },
                                    "lineageDepth": { "lte": 50 } } },
    { "id": "R3-서명",  "require": { "signerVerdict": { "eq": "valid" },
                                    "signerKeyId":  { "in": ["org.example/agent-7#2026"] } } },
    { "id": "R4-앵커",  "require": { "anchoredOk":   { "eq": true } } }
  ]
}
```

- **연산자는 `eq`·`in`·`gte`·`lte` 4개뿐이다.** 정규식·산술·스크립트는
  넣지 않는다 — 정책은 감사 대상 문서이고, 튜링 완전한 정책은 감사 불가능한
  정책이다. 부족한 표현력은 규칙을 늘려 해결한다.
- **미지 판정 키·연산자는 로드 시점 exit 2 다.** 오타(`reproducd`)를 조용히
  넘기면 그 규칙은 영원히 평가되지 않는 항상-참 구멍이 된다 — 게이트가 뚫린
  줄도 모르게. 실측: `unknown_key_and_operator_are_load_errors`.
- **deny 기본**: `rules` 가 비어 있으면 통과가 아니라 거부다(실측:
  `deny_default_…`). `defaultVerdict:"allow"` 는 가능하지만 봉투에 그대로
  드러나므로 감사에서 설명할 준비를 하라.

## 2. 판정 키 사전 (전체)

| 키 | 타입 | 재계산 경로 | 필요한 재료 |
|---|---|---|---|
| `reproduced` | bool | 캡슐 계획 재실행 산출 해시 == 영수증 해시 | `--deep` (없으면 unavailable — **신고를 읽지 않는다**) |
| `lineageValid` | bool | 머리→뿌리 체인 걷기(부모 파일 해시·산출=입력) | 없음 (캡슐 파일들) |
| `lineageDepth` | number | 위 걷기의 링크 수 | 없음 |
| `signerVerdict` | string | 사이드카를 캡슐 바이트·키 등록부와 대조 — valid·invalid·unknownKey·revoked·malformed·**unsigned**(사이드카 없음) | `--keyring` |
| `signerKeyId` | string\|null | 사이드카의 키 식별자 | `--keyring` |
| `anchoredOk` | bool | 캡슐 파일 해시가 앵커 로그에 등재됐는가(로그 자기 무결 선검사) | `--anchor-log` |

재료 없이 그 키를 참조하면 위반의 `actual` 에
`unavailable(판정 재료 미지정 …)` 이 실린다 — **모르는 것은 통과가 아니다**
(실측: `deny_default_and_unavailable_judgment`).

## 3. 판정 봉투 읽는 법

```json
{
  "policy": "재무팀 반입 정책 v1",
  "policySigned": null,
  "target": "work.capsule.json",
  "targetSha256": "9f2c…",
  "verdict": "deny",
  "evaluated": 6,
  "violations": [
    { "rule": "R3-서명", "key": "signerVerdict", "op": "eq",
      "expected": "valid", "actual": "invalid" }
  ]
}
```

- `verdict:"deny"` 는 exit 3 — 오류가 아니라 데이터다(#2707 규약).
- `violations[]` 가 감사 기록이다: 어느 규칙의 어느 조건이 무엇을 기대했고
  실측이 무엇이었는지. 이 배열을 그대로 보고서에 옮겨 적을 수 있게 설계했다.
- `evaluated` 는 (키, 연산자) 조건 수 — 규칙 수가 아니다.

## 4. TOCTOU — 판정 시점과 소비 시점

게이트 통과 후 파일이 바뀌면? 봉투의 `targetSha256` 이 판정 시점 해시다.
소비 직전에 **1해시 재대조**만 하면 된다(재검증 불필요):

```bash
rhwp gate work.capsule.json --policy p.json --json > verdict.json
# … 시간 경과 …
sha256sum work.capsule.json   # verdict.json 의 targetSha256 과 대조 후 소비
```

최강 모드는 소비 직전 `--deep` 게이트 재실행이다 — 비용은 재실행 1회.

## 5. 정책 자체의 신뢰 (M3 — 4년 축 재사용)

정책 파일이 바꿔치기되면 게이트는 무력하다. 새 메커니즘 대신 재사용한다:

```bash
# 정책을 서명해 배포한다 (정책 파일 바이트 대상 — 캡슐 서명과 같은 규약).
rhwp keygen --key-id org.example/policy#2026 --out policy.key.json
#   (sidecar 발급은 서명 도구 재사용 — policy.json.sig.json)
rhwp gate work.capsule.json --policy p.json --policy-keyring keyring.json --json
#   → policySigned: true|false  (미지정이면 null — 판정 축 꺼짐)
```

v1 에서 `policySigned:false` 는 **보고 필드**다(강제 아님) — 강제하려면
반입 스크립트가 이 필드를 조건으로 걸어라. 강제 플래그는 수요 실측 후
후속으로 판단한다(과설계 금지).

## 6. 배치 전략 — G2(우회) 방지

게이트는 반입 결정을 **표현**하는 도구지 **강제**하는 도구가 아니다. 강제는
배치의 문제다:

1. **단일 반입 지점**: 산출물이 파이프라인에 들어오는 길목을 하나로 모으고
   (수신 폴더·CI 잡·MCP 프로필), 그 길목에서만 게이트를 돈다.
2. **CI 게이트**: PR/잡의 필수 단계로 `rhwp gate … || exit 1`. verdict 봉투를
   아티팩트로 남기면 감사 기록이 공짜다.
3. **하네스 결합**: `harness status` 로 작업장 건강을, `gate` 로 개별 캡슐
   반입을 — 지속 판정과 반입 판정의 분업.
4. **프로필 경계**: MCP 호스트에는 품질검증 프로필로 게이트 도구를 열고,
   반입 우회 도구(직접 파일 조작)는 그 프로필에 넣지 않는다.

## 7. 반입 시나리오 6종 (정책 조각 모음)

| 시나리오 | 규칙 조각 |
|---|---|
| 사내 표준 (재현+계보) | `reproduced eq true` + `lineageValid eq true` |
| 서명 필수 조직 | + `signerVerdict eq "valid"` |
| 특정 에이전트만 | + `signerKeyId in ["org/agent-a#2026", "org/agent-b#2026"]` |
| 감사 대비 (앵커 강제) | + `anchoredOk eq true` |
| 외주 수신 (엄격) | 위 전부 + `lineageDepth lte 20` |
| 빠른 선별 (재실행 생략) | 서명·앵커·계보만 — reproduced 규칙을 빼서 --deep 비용 회피 (**재현 미보장을 알고 쓰는 것**) |

## 8. FAQ

**Q. verdict 가 deny 인데 어느 축 문제인지 어떻게 아나?**
`violations[].key` 가 축이다 — signerVerdict 면 4년 축, anchoredOk 면 5년 축.
그 축의 단건 명령(verify-signature·anchor verify·lineage)으로 내려가 원인을
좁혀라(판단 트리: 대전 50장).

**Q. 폴더 전체를 게이트할 수 없나?**
v1 은 단건이다. 폴더 통합 판정은 `harness status` 가 이미 하므로, 폴더
게이트(M2)는 status 재사용으로 후속 — 축마다 한 일.

**Q. 정책을 여러 개 겹치면?**
게이트 호출을 정책 수만큼 — 전부 allow 여야 반입. 정책 합성 문법을 넣지
않는 것도 감사 가능성 결정이다.

**Q. unsigned 를 허용하고 싶다.**
`signerVerdict in ["valid", "unsigned"]` — 명시적으로. 기본이 관대해지는
일은 없다.
