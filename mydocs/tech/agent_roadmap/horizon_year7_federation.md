---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/horizon_year7_federation.md
last_verified: 2026-08-10
---

# 7년 선행 축 — 연합: 조직 간 검증 가능한 노동 교환

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) →
  6년 게이트([#4449](https://github.com/edwardkim/rhwp/issues/4449)) → **이 문서**
- 이슈: [#4450](https://github.com/edwardkim/rhwp/issues/4450)
- 등급: **[지평]** — 코드 0줄.
- 착수 조건: 6년 축 머지.

## 1. 질문 — 검증은 조직 안에서만 닫힌다

1~6년 축의 검증은 전부 **한 신뢰 도메인 안**의 이야기다: 내 keyring, 내 로그,
내 정책. 에이전트 노동이 조직 경계를 넘는 순간(외주 에이전트의 산출물 반입,
계열사 간 문서 파이프라인, 공개 생태계의 재사용) 세 가지가 한꺼번에 열린다:

1. 상대 도메인의 **키**를 어떻게 아나 (keyring 교환)
2. 상대 도메인의 **로그**를 어떻게 검증하나 (앵커 증명의 이동)
3. 상대가 준 캡슐의 **조상 전체**를 어떻게 받나 (계보는 그래프다 — 머리만
   받으면 검증이 끊긴다)

한 줄 정의: **연합 축은 "한 도메인의 검증 가능성"을 "도메인 사이를 이동해도
깨지지 않는 검증 가능성"으로 확장한다.** 핵심 산출물은 프로토콜이 아니라
**교환 파일 형식**이다 — 오프라인 파일 하나로 전건 검증이 닫혀야 한다는 것이
이 저장소의 결이다(서버 없는 검증, 신뢰 기관 없는 해시).

## 2. 설계

### 2.1 신뢰 도메인 선언

```json
{
  "schemaVersion": "1.0",
  "kind": "trustDomain",
  "domain": "example-corp",
  "keyring": { "…": "4년 축 keyring 인라인 또는 파일 해시 참조" },
  "checkpoints": [ { "upToSeq": 1023, "merkleRoot": "c3f9…", "createdAt": "…" } ],
  "declaredPolicies": [ "감사 표준 L3 준수(주장)" ]
}
```

- 도메인 파일은 상대에게 건네는 **자기 소개서**다. 이것을 믿을지는 수신
  도메인의 결정이고(도메인 파일 수령 경로의 신뢰 — DNS·대면·계약), 도구는
  "이 도메인 파일 기준으로 검증"까지만 한다. 신뢰 뿌리의 외부화를 4년 축과
  같은 문장으로 반복하는 이유: 연합에서 이 경계가 가장 자주 뭉개진다.

### 2.2 교환 번들 — `.lineage-bundle`

zip 컨테이너 규약(새 포맷 발명 아님 — HWPX 가 zip 이듯):

```text
bundle/
  manifest.json      # {kind:"lineageBundle", head, files:[{path, sha256}], domain}
  capsules/          # 머리 + 조상 폐쇄집합 전체 (*.capsule.json)
  signatures/        # 각 캡슐의 sig.json (4년 축)
  anchor/            # 해당 캡슐들의 머클 경로 증명 + 체크포인트 (5년 축)
  domain.json        # 발신 도메인 선언 (§2.1)
```

- **조상 폐쇄집합(closure)**: 머리에서 도달 가능한 모든 부모 캡슐이 빠짐없이
  들어야 한다. `bundle export` 가 계보를 걷어 자동 수집하고, `bundle verify` 가
  완전성(모든 parent 참조가 번들 안에서 해소됨)을 1차 판정한다 — 부분 누락은
  검증 실패이지 경고가 아니다.
- manifest 의 files[] 해시로 번들 내 모든 파일이 고정된다. manifest 자체를
  4년 축으로 서명하면 번들 전체가 한 서명에 봉인된다.

### 2.3 검증 명령

```text
rhwp bundle export <머리캡슐> -o work.lineage-bundle
     [--anchor-log a.ndjson] [--sign-key k.pem]
rhwp bundle verify work.lineage-bundle --trust-domain their-domain.json
     [--policy my-policy.json] [--deep] [--json]
```

`bundle verify` 의 판정 순서(각 단계가 이전 단계 위에):

1. manifest 해시 전건 대조 (컨테이너 무결)
2. 조상 폐쇄집합 완전성 (§2.2)
3. 계보 걷기 — 3년/합류 축 그대로 (parentOk·lineageOk·[--deep] reproduced)
4. 서명 — 번들 동봉 keyring 이 아니라 **--trust-domain 의 keyring** 으로
   (동봉 keyring 만 믿으면 위장 도메인이 자기 키로 전부 통과시킨다 — F2)
5. 앵커 — 머클 경로를 도메인 선언의 체크포인트와 대조
6. [--policy] 수신측 정책 게이트 (6년 축) — **수신 정책이 항상 우선**이다.
   발신 도메인의 "우리는 L3 준수" 주장은 참고 표기일 뿐 판정에 안 쓴다.

봉투: `{containerOk, closureOk, lineageValid, signerOk, anchoredOk,
gateVerdict, violations[]}` — 단계별 판정 전부 데이터. exit 0/1/2/3.

## 3. 위협 모델

| # | 공격 | 판정 |
|---|---|---|
| F1 | 번들 내 파일 바꿔치기 | manifest 해시 대조 (1단계) |
| F2 | 위장 도메인 (자체 키로 전부 서명한 가짜 역사) | 수신측이 **자기 경로로 받은** trust-domain 기준 검증 (4단계) — 도메인 파일 수령 경로가 신뢰 뿌리임을 재명시 |
| F3 | 조상 일부 은닉 (불리한 부모 누락) | 폐쇄집합 완전성 (2단계) — parent 참조 미해소 = 실패 |
| F4 | 이중 뷰 (상대마다 다른 로그) | 체크포인트 교차 대조 — 두 수신자가 받은 체크포인트가 다르면 폭로. 단일 수신자 단독으로는 **못 잡는다** (정직: 수신자 간 대사(照合)는 운영 절차) |
| F5 | 재현 불가 환경 차이 (버전 다른 rhwp) | 영수증의 toolVersion 대조 — 불일치는 reproduced 판정 불가로 보고(거짓 실패 방지) |

## 4. 단계와 DoD

| 단계 | 내용 | DoD |
|---|---|---|
| Y7-M1 | 번들 형식+export/verify(1~3단계) | 왕복 + 파일 1개 변조(F1)·부모 1개 누락(F3) 검출 테스트 |
| Y7-M2 | 도메인 신뢰(4~5단계) | 위장 keyring 번들이 trust-domain 기준으로 거부되는 픽스처 |
| Y7-M3 | 정책 결합(6단계)+등재 | 수신 정책 우선 테스트 + 7표면 등재 green |

## 5. 정직 조항

- 이 축은 전송 프로토콜을 정의하지 않는다 — 파일이 어떻게 이동하든(메일·
  드라이브·API) 검증이 동일하다는 것이 설계의 요점이다.
- F4(이중 뷰)의 완전한 해결은 수신자 간 대사 또는 공용 투명성 로그를 요구한다
  — 전자는 운영, 후자는 이 사다리 밖의 공공 인프라 문제로 명시한다.
- toolVersion 이 다른 도메인 간 재현(F5)은 "판정 불가"가 정직한 답이다 —
  버전 고정 재현 환경의 표준화는 10년 축(감사 표준)의 소재다.
