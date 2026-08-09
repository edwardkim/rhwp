---
kind: guide
status: active
canonical: mydocs/tech/agent_roadmap/horizon_year5_anchor.md
last_verified: 2026-08-10
---

# 5년 선행 축 — 앵커(투명성 로그): 역사 전체 재작성을 잡는다

- 좌표: 조망 [#3907](https://github.com/edwardkim/rhwp/issues/3907) →
  4년 귀속([#4447](https://github.com/edwardkim/rhwp/issues/4447)) → **이 문서**
- 이슈: [#4448](https://github.com/edwardkim/rhwp/issues/4448)
- 등급: **[지평]** — 코드 0줄.
- 착수 조건: 4년 축 머지.

## 1. 질문 — T7 은 체인 안에서 못 잡는다

합류 설계서([#4407](https://github.com/edwardkim/rhwp/issues/4407))의 위협 표가
정직하게 남긴 구멍이 T7(역사 전체 재작성)이다: 키 소유자 본인이 뿌리부터 전부
재발급·재서명하면, 해시 체인도 서명도 전부 유효하다. **체인 내부의 어떤
검증도 "이것이 원래의 역사"임은 증명하지 못한다** — git 이 강제 푸시에 갖는
한계와 정확히 같고, git 의 방어(원격 사본·서명 태그·공표된 릴리스)와 같은
원리 — **외부 시점 고정** — 이 필요하다.

한 줄 정의: **앵커 축은 "이 캡슐이 이 시점 이전에 존재했다"를 append-only
로그와 그 로그의 외부 공표로 증명 가능하게 만든다.**

## 2. 설계

### 2.1 계보 로그 — append-only ndjson 해시 체인

```json
{"seq":0,"kind":"anchorLog","capsuleSha256":"9f2c…","prevEntryHash":null,"loggedAt":"2026-08-10T12:00:00Z"}
{"seq":1,"kind":"anchorLog","capsuleSha256":"a01b…","prevEntryHash":"e4d7…","loggedAt":"…"}
```

- 한 줄 = 한 등재. `prevEntryHash` = 직전 줄의 파일 바이트 SHA-256 — **계보
  캡슐 체인과 동형**이다. 이 저장소의 검증 자산(해시 체인 걷기·변조 검출
  테스트 패턴)을 그대로 재사용한다는 뜻이고, 새 발명이 아니라는 뜻이다.
- 로그는 캡슐 내용을 모른다 — 해시만 등재한다. 8년 축(선택적 공개)과의 정합:
  로그 공개가 내용 공개를 강제하지 않는다.

### 2.2 머클 체크포인트

주기적으로(운영이 정한 간격) 로그 전체의 머클 루트를 계산해 체크포인트를 만든다:

```json
{ "kind": "anchorCheckpoint", "upToSeq": 1023, "merkleRoot": "c3f9…",
  "createdAt": "…", "publishedTo": ["표기용 — 증명 아님"] }
```

- **도구의 경계**: rhwp 는 merkleRoot 계산과 검증까지만 한다. 체크포인트를
  어디에 공표하는가(사내 게시판·git 커밋·타임스탬프 기관·제3자 로그)는 운영
  절차다. `publishedTo` 는 표기용 주장 필드다 — 도구가 공표를 증명하는 척하지
  않는다.
- 머클 트리인 이유: 로그가 커져도 "캡슐 X 가 체크포인트 C 에 포함됨"을
  경로 증명(log₂N 해시)만으로 검증할 수 있다 — 7년 축(연합 번들)이 이 경로
  증명을 교환 형식에 싣는다.

### 2.3 CLI 설계

```text
rhwp anchor add <캡슐.json> --log anchor.ndjson        # 등재 (append 전용)
rhwp anchor checkpoint --log anchor.ndjson             # 머클 루트 산출
rhwp anchor verify <캡슐.json> --log anchor.ndjson [--checkpoint cp.json]
  → 봉투 {logged: bool, seq, logChainOk: bool, inCheckpoint: bool|null, merklePath: […]}
rhwp lineage head.json --anchor-log anchor.ndjson
  → 링크마다 5번째 판정 축 anchoredOk (opt-in — 4년 signerOk 와 같은 규약)
```

- `anchor add` 는 로그 파일 끝에만 쓴다. 기존 줄 수정이 감지되면(체인 해시
  불일치) exit 3 — 로그 자체가 자기 무결을 판정한다.
- exit 규약 불변: 0/1/2/3.

### 2.4 시점 증명의 의미론 (정확하게)

앵커가 증명하는 것: **"캡슐 해시 H 는 체크포인트 C 공표 시점 이전에 로그에
있었다."** 공표가 외부에 있었다면(제3자가 C 를 봤다면) 이후의 역사 재작성은
C 와 충돌한다 — T7 이 잡힌다.

앵커가 증명하지 않는 것: 작업이 실제로 그 시각에 **수행**됐다는 것(로그 등재
시점 ≠ 작업 시점), 그리고 공표 자체(도구 밖). 이 두 한계를 문서 머리에 두는
것이 이 로드맵의 방식이다.

## 3. 위협 모델

| # | 공격 | 판정 |
|---|---|---|
| A1 | 로그 중간 수정 | prevEntryHash 불일치 → exit 3 (자기 무결) |
| A2 | 로그 꼬리 절단 | 체크포인트 upToSeq 와 대조 → 절단 검출. 마지막 체크포인트 이후 꼬리는 **무방비** — 체크포인트 주기가 방어선임을 명시 |
| A3 | 로그 전체 재작성 | 공표된 체크포인트와 충돌 → 검출. 공표가 없었다면 **못 잡는다** (공표는 운영) |
| A4 | 이중 로그 (분기 뷰 — 상대마다 다른 로그 제시) | 단일 조직 안에서는 체크포인트 비교로, 조직 간에는 7년 축(연합의 상호 로그 검증)으로 — 이 축 단독 한계 명시 |
| A5 | 선등재 (작업 전에 해시만 미리? ) | 캡슐 해시는 산출물이 있어야 계산되므로 선등재는 곧 산출물 선존재 — 공격이 성립하지 않음 (해시의 성질) |

## 4. 단계와 DoD

| 단계 | 내용 | DoD |
|---|---|---|
| Y5-M1 | 로그 add/자기 무결 검증 | append 왕복 + 중간 1바이트 변조 exit 3 테스트 |
| Y5-M2 | 머클 체크포인트+경로 증명 | 1024건 로그에서 경로 증명 검증 + 절단(A2) 검출 테스트 |
| Y5-M3 | lineage anchoredOk 통합 | opt-in 무파손 + 미등재 캡슐 anchoredOk:false |
| Y5-M4 | 공표 어댑터(선택) | git-커밋 공표 예시 1종 — 도구 경계 밖임을 문서로 |

## 5. 정직 조항

- 시각(`loggedAt`)은 로컬 시계의 주장이다. 신뢰 가능한 시각은 외부
  공표(운영)나 타임스탬프 기관 연동(선택 확장)에서만 나온다.
- 체크포인트 공표 주기·장소는 조직 결정이다 — 도구가 대신 정하지 않는다.
- 4년 축과의 결합(서명된 로그 항목)은 Y5 범위에 넣지 않았다 — 로그 파일
  자체를 서명 대상으로 삼으면 되므로 새 메커니즘이 필요 없다(동형 재사용).
