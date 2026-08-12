---
kind: canonical
status: active
canonical: mydocs/tech/agent_runtime/version_policy.md
last_verified: 2026-08-11
---

# 버전 정책 — 릴리스 semver × 스키마 4축 (R67, #4329)

**이 문서가 버전 정책의 canonical 이다.** 정책의 기계 적용점은
[`src/schema_registry.rs`](../../../src/schema_registry.rs)(R83 단일 출처
레지스트리)이고, 정책↔실물의 드리프트는
[`tests/schema_registry_contract.rs`](../../../tests/schema_registry_contract.rs)
가 잡는다. 여기 적힌 규칙 중 "제안" 표지가 붙은 것은 메인테이너 채택 판단이
남아 있는 항목이다 — 채택·개정 권한은 전부 메인테이너에게 있다.

착수 근거: R67 트랙 문서가 실측한 "축마다 따로 굴러감" + #4327 실조사 §3 의
소비자 스테일 고정 2건(외부 소비자가 v0.7.x 스냅샷의 표면 인식에 고정된 채
갈라져 나간 실물 마찰). 버전이 흩어져 있으면 정책을 세울 자리가 없고, 정책이
없으면 소비자는 추종할 방법이 없다 — 그래서 R67(정책)과 R83(레지스트리)은 한
PR 로 함께 착지한다(두 트랙 문서의 착수 게이트가 서로를 가리키는 이유).

## 1. 축의 정의 — 무엇이 버전을 갖는가

| 축 | 상수 (schema_registry) | 노출 표면 | 1차 소비자 |
|---|---|---|---|
| **봉투** | `ENVELOPE_SCHEMA_VERSION` = 1.0 | 모든 `--json` 봉투 최상위 `schemaVersion` | 봉투 파싱 스크립트·외부 소비자 |
| **IR** | `IR_SCHEMA_VERSION` = 1.0 | `export-ir-schema` 봉투의 `irSchemaVersion` | IR 소비 코드·외부 코드 생성기 |
| **capabilities** | `CAPABILITIES_SCHEMA_VERSION` = 1.3 | `export-capabilities-schema` 봉투의 `capabilitiesSchemaVersion` | MCP 클라이언트·외부 명령 래퍼 |
| **계획(plan)** | `PLAN_SCHEMA_VERSION` = 1.1 | `export-plan-schema` 봉투의 `planSchemaVersion` | `run` 계획서 작성기·검증기 |
| **릴리스 semver** | `crate_version()` (= `CARGO_PKG_VERSION`) | `rhwp --version`, 모든 봉투의 `version`, 릴리스 태그 `v*` | GitHub Release·기존 npm/확장 배포 |

값의 유일한 정의처는 `src/schema_registry.rs` 다. `ir_schema`·
`capabilities_schema`·`plan_schema` 모듈의 동명 상수는 **재수출**이고(호출부
경로 보존), 봉투 리터럴·`$id` 버전 조각은 전부 상수 참조로 파생된다. #4329
이전에는 봉투 리터럴만 8개 파일 ~67사이트에 흩어져 있었다 — 그 상태로의 회귀는
`no_version_literals_outside_registry` 소스 스캔이 red 로 만든다.

### 1.1 축이 아닌 인접 상수 (혼동 주의)

- `REQUIRED_PLAN_VERSION`(src/plan_schema.rs) — 계획 **파일**이 선언해야 하는
  `planVersion` 문법 수용 게이트. 계획 스키마(`planSchemaVersion`)와
  **의도적으로 독립**이다(모듈 doc 이 명시). 레지스트리에 편입하지 않는다.
## 2. 판올림 규약 — 언제 어느 숫자가 오르는가

네 스키마 축 공통(종전 각 모듈 머리말의 규약을 단일 서술로 승계):

- **minor (x.y → x.y+1)** — 필드·항목 **추가**. 하위호환이다. 소비자는 모르는
  필드를 무시하면 된다(봉투는 추가-전용 계약 — `jsonContract.schemaPolicy` 와
  JSON Schema 의 `additionalProperties: true` 가 같은 말을 한다).
- **major (x.y → x+1.0)** — 기존 필드의 **의미 변경·개명·삭제**. 파괴적이다.
  **분기 회고 승인 없이 금지**(종전 규약 승계). major 를 올리는 PR 은
  마이그레이션 노트(무엇이 깨지고 소비자가 무엇을 바꿔야 하는가)를 동반한다.
- **축 추가** — `schemaRegistry.axes` 집합 자체가 늘어나는 것은 capabilities
  봉투의 계약 확장이므로 **capabilities minor** 로 계상한다(1.3 이 바로 그
  사례다 — `schemaRegistry` 필드 추가).

판올림은 값 변경 + 이력 주석(레지스트리 상수의 doc comment) + 관련 가드 갱신이
한 커밋에 함께 간다. 값만 바꾸고 이력을 안 남기면 다음 사람이 "왜 올랐는가"를
잃는다.

## 3. semver 연동 — 스키마가 오르면 릴리스는 (제안)

crate 는 현재 0.x 다. semver 관례상 0.x 에서는 **minor 가 파괴적 변경의
신호**이고 patch 가 비파괴 수정이다. 이를 스키마 축과 다음처럼 잇는다:

| 스키마 변화 | 0.x 릴리스 요건 (제안) | 1.0 이후 (제안) |
|---|---|---|
| 어느 축이든 minor ↑ | 다음 **minor** 릴리스에 실어 출하 (patch 릴리스에 스키마 변화를 싣지 않는다) | 다음 minor |
| 어느 축이든 major ↑ | **minor ↑ + 마이그레이션 노트 필수** (0.x 의 minor 가 breaking 신호) | **major ↑** |
| 스키마 무변화 | patch 자유 | patch 자유 |

이유: 외부 소비자와 기존 npm 패키지 사용자는 버전 범위로 업데이트를 받는다.
스키마 변화가 patch 에 섞이면 "안전한 업데이트" 신호가 거짓이 된다. 이 표는
제안이며, 채택 시 릴리스 절차 문서에 편입한다 — 그 전까지 구속력은 없다.

## 4. 외부 소비자 추종 규칙

공식 Python·Node 바인딩은 v0.8.4에서 철회됐다(#4655). 외부 소비자는
`rhwp capabilities`의 `schemaRegistry`와 각 `export-*-schema` 결과를 기계 대조한다.
상위 호환 minor는 모르는 필드를 무시하고, major 차이는 실행을 중단한 뒤 마이그레이션
노트를 확인한다. 특정 언어 래퍼의 지원 범위와 생성물 드리프트는 해당 다운스트림이
책임진다.

## 5. 외부 소비자 대사 절차 (#4327 U2)

외부 소비자(래퍼·스킬·바인딩 제작자)가 상류 버전 진화를 추종하는 표준 절차:

```bash
rhwp capabilities | jq '.schemaRegistry'
```

- `crateVersion` — 지금 실행 중인 rhwp 릴리스.
- `axes[]` — 축별 현재 버전·노출 표면·판올림 규약. 소비자는 자기가 지원하는
  버전과 **기계로** 대조하고, minor 차이는 수용(추가-전용), major 차이는 중단
  후 마이그레이션 노트를 확인한다.
- `policy` — 이 문서의 저장소 경로(경로 실물성은 계약 테스트가 고정).

이 표면이 없던 시기의 실측 결과가 #4327 §3 이다: 외부 소비자가 특정 시점
스냅샷(v0.7.3·0.7.7)의 표면 인식에 고정된 채 "상류에는 편집이 없다"는 낡은
전제로 자체 우회를 만들었다. 대조할 자리가 생겼으므로, 이후의 스테일 고정은
소비자 선택의 문제가 되고 상류 구조의 문제가 아니게 된다 — R86(온보딩
사례집)이 이 절차의 채택 여부를 사례로 환류한다.

## 6. 릴리스 태그·서명과의 연결 (R38 게이트 ① 의 정책 몫)

- 현재 실물: `v*` 태그 push → `release-binary.yml`(#612)이 4플랫폼 바이너리 +
  `SHA256SUMS.txt` 를 릴리스에 첨부한다. SHA256SUMS 는 **무결성**(전송 중 변조
  검출)이지 **진본성**(누가 만들었는가)이 아니다 — R38 트랙 문서가 게이트 ① 을
  유지하는 이유.
- 제안: 진본성은 태그 시점 아티팩트 서명으로 얹는다(도구 선택 — minisign 류
  경량 서명 또는 Sigstore 계열 — 은 메인테이너 결정 사항; 어느 쪽이든
  SHA256SUMS 와 병행).
- 버전 정합: 태그 `vX.Y.Z` = `CARGO_PKG_VERSION` = 봉투 `version`이어야 한다.

## 7. 가드 지도 — 이 정책은 어디서 기계로 지켜지는가

| 가드 | 무엇을 고정하나 |
|---|---|
| `tests/schema_registry_contract.rs` · `no_version_literals_outside_registry` | 소스 산개 금지(봉투 리터럴·상수 재정의·`$id` 조각) |
| 〃 · `capabilities_schema_registry_matches_constants` | `schemaRegistry` ↔ 상수 일치, 축 집합 고정, 정책 경로 실물성 |
| 〃 · `export_schema_envelopes_derive_from_registry` | 각 `export-*-schema` 봉투·`$id` 의 축 버전 파생 |
| 〃 · `capabilities_schema_declares_schema_registry` | 자기서술 스키마에 새 표면 등재(코드 생성기 시야) |
| `src/schema_registry.rs` 단위 2본 | 조립 자체의 축 집합·경로 실물성 |

## 8. 미결정 사항 (메인테이너 판단 대기)

1. §3 semver 연동표의 채택 여부(채택 시 릴리스 절차 문서에 편입).
2. §6 서명 도구 선택과 도입 시점(R38·R63 착수의 선행 판단).
3. 축 추가 시 이 문서·레지스트리·capabilities 스키마 3곳 동반 갱신을 묶는
   추가 가드의 필요 여부(현재는 계약 테스트의 축 집합 고정이 그 역할).
