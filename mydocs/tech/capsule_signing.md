---
kind: guide
status: active
canonical: mydocs/tech/capsule_signing.md
last_verified: 2026-08-10
---

# 캡슐 서명 — 귀속(4년 축)의 형식·명령·위협 모델

- 좌표: 설계서 `mydocs/tech/agent_roadmap/horizon_year4_signing.md` ([지평]) 의
  **구현 1호** — 이슈 [#4509](https://github.com/edwardkim/rhwp/issues/4509),
  선행 조건(계보 devel 통합) 충족 확인 후 착공.
- 검증 원장: `tests/signing_contract.rs` (이 문서의 모든 단언은 그 테스트가
  실측으로 고정한다 — 문서와 테스트가 다르면 테스트가 맞다).

## 0. 한 장 요약

작업 캡슐(영수증+계획)은 "무엇이 어떻게 만들어졌나"를 증명하지만 **"누가
봉인했나"** 는 증명하지 못했다. 이 축은 세 파일 형식(키·키 등록부·분리 서명)과
두 명령(`keygen`·`verify-signature`), 두 플래그(`replay --sign-key`·`lineage
--keyring`)로 그 공백을 메운다:

```text
rhwp keygen --key-id org.example/agent-7#2026 --out agent7.key.json
rhwp replay --plan-json … --capsule work.capsule.json --sign-key agent7.key.json
          → work.capsule.json + work.capsule.json.sig.json (분리 서명)
rhwp verify-signature work.capsule.json --keyring keyring.json --json
          → verdict: valid | invalid | unknownKey | revoked | malformed
rhwp lineage head.capsule.json --keyring keyring.json --json
          → 링크마다 signerOk / keyId 판정 축 추가 (opt-in)
```

판정은 언제나 봉투 데이터이고, valid 가 아니면 exit 3 이다 (#2707 규약).

## 1. 설계 결정 4가지와 그 근거

### 1.1 서명 대상 = 캡슐 파일 바이트 (정규화 금지)

| 선택지 | 채택 | 근거 |
|---|---|---|
| canonical JSON 서명 | ✗ | 정규화 규칙 자체가 공격면·구현 분기 — JSON 정규화의 역사는 상호운용 사고의 역사다 |
| **파일 바이트 서명** | ✓ | 계보 축이 이미 파일 바이트 해시(부모 링크) 위에 서 있다 — 같은 대상을 봉인해야 두 체계가 어긋나지 않는다 |

따라서 **캡슐 파일 재직렬화는 금지**다. 이것은 새 제약이 아니다 — "캡슐은 발급
후 불변"은 계보(부모 해시 대조)의 기존 전제다. 포맷터·에디터로 캡슐을 열어
저장하는 순간 부모 해시도 서명도 함께 깨지고, 그것이 정확히 의도된 동작이다.

### 1.2 분리 서명(sidecar), 내장 아님

서명을 캡슐 안에 넣으면 "서명 필드를 제외한 바이트에 서명한다"는 정규화 문제가
되돌아온다. `<캡슐>.sig.json` 분리 파일이면: 캡슐 바이트 불변 → 부모 해시
대조(계보)·파일 해시 회계(감사)와의 정합이 공짜이고, 서명 없는 소비자는 사이드카를
무시하면 된다(하위호환 0비용).

### 1.3 Ed25519, 그리고 결정론

Ed25519 는 **결정론 서명**이다 — 같은 키로 같은 바이트에 서명하면 언제나 같은
서명이 나온다. 이 저장소의 검증 문화(같은 계획 → 같은 바이트 → 같은 해시)와
정확히 정합하며, `tests/signing_contract.rs` 의
`sign_verify_roundtrip_and_deterministic_signature` 가 이를 실측으로 고정한다:
같은 계획의 캡슐 재발급(캡슐 결정론) → 같은 캡슐 바이트 → **같은 서명 문자열**.
결정론 사슬이 계획→산출→캡슐→서명까지 이어진 것이다.

의존성은 `ed25519-dalek 2`(순수 Rust — wasm 타깃 그대로 컴파일) 하나이고, 키
생성 엔트로피는 이미 의존성 트리에 있던 `getrandom` 을 직접 쓴다. PEM/pkcs8
파서를 피하려고 키 파일을 자체 JSON 형식으로 정한 것도 의존성 절제의 일부다.

### 1.4 폐기 > 서명 유효 (판정 우선순위)

폐기된 키의 서명은 암호학적으로 유효해도 판정은 `revoked` 다. 단, 봉투의
`signatureOk` 는 암호학적 사실을 그대로 보고한다(true) — **판정과 사실을 섞지
않는다**. 소비자는 "서명은 진짜지만 키가 폐기됐다"는 두 정보를 다 받는다.

## 2. 파일 형식 명세

### 2.1 키 파일 (`keygen --out`)

```json
{
  "schemaVersion": "1.0",
  "kind": "ed25519Key",
  "keyId": "org.example/agent-7#2026",
  "alg": "ed25519",
  "secret": "<base64 32B>",
  "publicKey": "<base64 32B>"
}
```

- **비밀키가 담긴다.** 캡슐·봉투·저장소에 절대 인라인하지 않는다. `keygen` 은
  기존 파일 덮어쓰기를 거부한다(exit 2) — 잃어버린 키는 재발급하면 되지만
  덮어쓴 키는 복구 불능이기 때문이다.
- `keyId` 관례: `소유 주체/용도#세대`. 키 회전 시 세대만 올린다
  (`#2026` → `#2027`).

### 2.2 키 등록부 (keyring.json)

```json
{
  "schemaVersion": "1.0",
  "kind": "keyring",
  "keys": [
    { "keyId": "org.example/agent-7#2026", "publicKey": "<base64 32B>",
      "revoked": null },
    { "keyId": "org.example/agent-3#2025", "publicKey": "<base64 32B>",
      "revoked": { "at": "2026-07-01", "reason": "유출 신고" } }
  ]
}
```

- **폐기는 삭제가 아니라 기록이다.** 항목을 지우면 그 키의 과거 서명이
  `unknownKey` 로 격하되어 이력 추적이 끊긴다 — `revoked` 를 채워 `revoked`
  판정으로 남기는 것이 옳다.
- 등록부의 신뢰 뿌리(이 파일을 왜 믿는가)는 도구 밖 거버넌스다. 도구는 "주어진
  등록부 기준 판정"까지만 한다.

### 2.3 분리 서명 (`<캡슐>.sig.json`)

```json
{
  "schemaVersion": "1.0",
  "kind": "capsuleSignature",
  "capsuleSha256": "<서명 시점 캡슐 파일 SHA-256>",
  "alg": "ed25519",
  "keyId": "org.example/agent-7#2026",
  "signature": "<base64 64B>",
  "signedAt": "2026-08-10T12:00:00Z"
}
```

- `capsuleSha256` 은 자기서술이다 — 검증 시 실물과 대조되어
  `capsuleShaMatches` 로 보고된다. 다른 캡슐의 사이드카를 가져다 붙이면 여기서
  드러난다.
- **`signedAt` 은 주장이지 증명이 아니다.** 서명 시점의 증명은 5년 축(앵커)의
  몫이며, 이 경계가 4년/5년 축의 설계 분리선이다.

## 3. 명령 계약

### 3.1 `keygen`

```text
rhwp keygen --key-id <소유/용도#세대> --out <키.json> [--json]
```

봉투: `{schemaVersion, keyId, publicKey, keyFile}` — publicKey 를 그대로 키
등록부에 옮겨 적으면 된다. 실패 규약: 인자 누락·기존 파일 = exit 2.

### 3.2 `replay --sign-key`

```text
rhwp replay --plan-json <json> --capsule <캡슐> --sign-key <키.json> [--json]
```

캡슐 저장 **성공 후** 방금 쓴 파일 바이트를 서명해 사이드카를 만든다.
`--sign-key` 는 `--capsule` 없이는 사용법 오류다(서명할 대상이 없다).
`--parent` 와 자유롭게 조합된다 — 서명된 계보 체인이 그렇게 만들어진다.

### 3.3 `verify-signature`

```text
rhwp verify-signature <캡슐> --keyring <키링.json> [--sig <서명.json>] [--json]
```

봉투 필드 (지식지도 §2-2 사전 등재):

| 필드 | 뜻 |
|---|---|
| `capsuleSha256` | 실물 캡슐 파일 바이트의 SHA-256 |
| `capsuleShaMatches` | 사이드카 기록 해시와 실물의 일치 |
| `signatureOk` | 암호학적 검증 결과 (키를 몰라 검증 불가면 `null`) |
| `keyKnown` / `revoked` | 등록부 조회 결과 |
| `verdict` | `valid` · `invalid` · `unknownKey` · `revoked` · `malformed` |

서명 파일 **파싱 실패는 IO 오류가 아니라 판정 데이터**(`malformed`)다 —
위조·손상 서명을 오류 뒤에 숨기지 않는다. IO(캡슐/서명/등록부 부재)는 exit 1,
사용법 exit 2, valid 아님 exit 3.

### 3.4 `lineage --keyring`

체인 걷기에 4번째 판정 축이 붙는다:

| `signerOk` | 뜻 | 체인 판정 |
|---|---|---|
| (필드 없음) | `--keyring` 을 주지 않음 — 축 자체가 꺼짐 | 영향 없음 (**opt-in 무파손**) |
| `null` | 사이드카 없음 (미서명) | 깨지지 않음 — 서명 강제는 게이트(6년 축)의 몫 |
| `true` | verdict `valid` | — |
| `false` | invalid·unknownKey·revoked·malformed | **깨진 계보** (exit 3, brokenAt) |

미서명(null)이 체인을 깨지 않는 것은 결정이다: lineage 는 사실을 보고하고,
"미서명 반입 금지" 같은 정책 판정은 6년 축(admissionPolicy 의 signerIn 조건)의
직무다 — 축마다 한 가지 일.

## 4. 위협 모델 → 구현 매핑 (설계서 S1~S5 실측 대조)

| # | 공격 | 설계서 판정 | 구현 실측 (signing_contract.rs) |
|---|---|---|---|
| S1 | 캡슐 변조 후 서명 유지 | signature 검증 실패 | `tampered_…`: 1바이트 변조 → `invalid`·exit 3 ✓ |
| S2 | 위장 발급 (남의 작업인 척) | 등록부 기준 미등록 | `unknownKey`·exit 3 ✓ — 강제는 6년 축 정책에서 |
| S3 | 키 유출 후 위조 서명 | **못 잡는다** (폐기 후 신규 검증만) | `revoked` 판정 + `signatureOk:true` 병기 실측 ✓ — 소급 판정은 5년 축 결합 전 불가 명시 |
| S4 | signedAt 거짓 기재 | **못 잡는다** (주장 필드) | 형식 명세·주석에 명시, 판정에 불사용 ✓ |
| S5 | keyring 바꿔치기 | 도구 밖 | 신뢰 뿌리 거버넌스 문서화 ✓ — 등록부 파일 자체의 서명·앵커는 5·6년 축 |

사이드카 바꿔치기(다른 캡슐의 서명 이식)는 S1 의 변형으로: 서명 검증이 현재
캡슐 바이트에 대해 실패(`invalid`)하고 `capsuleShaMatches:false` 가 어느 파일의
서명이었는지 단서를 준다 — 테스트 ① 이 정확히 이 경로다.

## 5. 운영 수칙

1. **키 파일 보관** — 비밀키다. 저장소에 커밋 금지(.gitignore 대상), 공유
   드라이브 금지. Windows 는 0600 이 없으므로 폴더 ACL 로 대신한다.
2. **회전** — 세대 표기(`#2026`)로 새 키를 발급하고, 옛 키는 등록부에서
   `revoked` 로 남긴다(삭제 금지 — §2.2).
3. **유출 대응** — 등록부에 `revoked:{at, reason}` 기입이 전부다. 이후 그 키의
   모든 검증은 `revoked` 로 떨어진다. 유출~폐기 사이 발급분의 소급 판정은 앵커
   축 도입 전까지 불가능함을 조직이 알고 있어야 한다.
4. **등록부 배포** — 등록부 파일의 이동 경로가 신뢰 뿌리다. 사내 저장소 커밋
   (변경 이력이 남는 곳)을 권하고, 연합(7년 축) 전까지는 도메인 밖 배포를
   하지 않는다.

## 6. FAQ

**Q. 캡슐을 열어보고 싶은데 재직렬화가 금지라니?**
읽기는 자유다 — 금지는 "열어서 다시 저장"이다. 수정이 필요하면 새 캡슐을
발급하라(replay 재실행). 캡슐은 문서가 아니라 증거물이다.

**Q. 서명 없이 쓰던 파이프라인이 깨지나?**
아니다. 사이드카는 무시하면 되고, `lineage` 는 `--keyring` 없이는 signerOk
축 자체를 싣지 않는다(실측: `lineage_signer_axis_and_optin_compat`).

**Q. 왜 서명을 MCP 세션 도구가 아니라 무상태 도구로만 냈나?**
서명·검증은 문서 세션(IR 보유)과 무관한 파일 연산이다. 세션 표면에 넣으면
프로필 경계(읽기 전용 세션)가 복잡해질 뿐 이득이 없다.

**Q. 다음 단계는?**
Y4-M2 잔여(등록부 자체의 서명), 그리고 5년 축(앵커 로그 — signedAt 을 주장에서
증명으로). 착수는 규약대로 이 구현의 머지가 결정한다.
