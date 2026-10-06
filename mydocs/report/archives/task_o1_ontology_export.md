# O1 — `export-ontology` (스키마 모드): 자기서술 유도 JSON-LD 온톨로지

- 이슈: #3907 (로드맵 — 온톨로지 자동 유도 층)
- 브랜치: `task/ontology-export-impl` (upstream/devel `dc7d7adcc` 기준)
- 스코프: **O1 스키마 모드만** — rhwp 라는 도구 자신(IR 타입·명령 표면·신뢰 경계)의
  온톨로지. 특정 문서 한 부를 개체로 서술하는 문서 인스턴스 모드(O2)는 후속이다.

## 무엇

`rhwp export-ontology [--bare] [-o 파일] [--json]` — 인자 없는 자기서술 계열
(`export-ir-schema` 관례)의 새 명령. JSON-LD 를 낸다:

- `@context` — 자체 어휘 접두어(`rhwp:` = `https://github.com/edwardkim/rhwp/ontology#`)
  + 표준 어휘(`rdf`/`rdfs`/`xsd`/`schema`). schema.org 대응은 실제로 성립하는 것만:
  명령·MCP 도구 = `schema:Action`. IR 타입을 특정 schema.org 타입에 강제 매핑하지 않는다.
- `@graph` —
  1. **IR 타입 41정의 → 클래스 노드**(`rdfs:Class`). 계층은 IR 스키마의 구조에서
     유도되는 만큼만: 순수 `oneOf` 유니온(`Control`)의 변형 15종 →
     `rdfs:subClassOf rhwp:ir/Control`. 억지 계층 없음.
  2. **IR 필드 → 속성 노드**(`rdf:Property`) 191건 — 도메인 = 소속 타입, 레인지 =
     필드 타입(`$ref`→클래스, 원시형→`xsd:*`, 익명 객체→`rdfs:Resource`),
     배열은 `rhwp:multiValued`, `[X,null]` 유니온은 `rhwp:nullable`, 필수 필드는
     `rhwp:required`. 사람용 의미는 스키마의 `description` 이 있으면 `rdfs:comment` 로
     싣고 **없으면 생략** — 지어내지 않는다.
  3. **명령 66건 + MCP 도구 44건 → 행위 노드**(`rhwp:Action`+`schema:Action`) —
     category·json·batch·flags·recordFields·subcommands 를 선언 그대로, 도구는
     `rhwp:implementsCommand` 로 내려가는 명령 노드에 연결하고
     inputProperties/requiredInputs/outputFields 를 싣는다.
     `annotations`(readOnly/destructive/idempotent, #4226)는 **있으면 통째로 싣고
     없으면 생략** — #4226 미머지 devel 에서도 유도가 성립한다(모양 무가정).
  4. **출처 지도 → 신뢰 술어** — `provenance::MAP` 의 명령별 untrusted 경로가
     `rhwp:untrustedFields` 로, note 가 `rhwp:provenanceNote` 로 실린다
     (비어 있지 않은 명령 18건, 경로 전수).

봉투: `schemaVersion`·`ontology`·`classCount`·`propertyCount`·`actionCount`
(+ 출처 표지 `untrustedContent:false`/`untrustedFields:[]`). `source` 없음 —
문서를 열지 않는 자기서술이다. exit 0/1/2 (`-o` 쓰기 실패 = 1, 사용법 오류 = 2·stdout 0B).

실측 유도 통계 (이 브랜치 빌드): **클래스 41 · 속성 191 · 행위 110**(명령 66 +
도구 44) · subClassOf 간선 15 · 신뢰 술어 비어 있지 않은 명령 18 · 그래프 노드 343.

## 판단

- **전부 실행 시점 유도, 손 나열 상수 0.** 유도 원천은 같은 크레이트의 단일 출처
  함수 4개다: `ir_schema()`·`capabilities_value()`·`mcp_tool_definitions()`·
  `provenance::MAP`. 프로세스 재호출 없음. 손으로 쓰는 Ontology-as-Code 는 낡지만,
  살아있는 자기서술에서 유도하면 원천이 바뀔 때 온톨로지가 함께 바뀐다 —
  드리프트가 구조적으로 불가능하다. (자기 포섭: `export-ontology` 명령과
  `hwp_export_ontology` 도구 자신도 유도 결과에 나타난다.)
- **조립 위치.** 유도 코어는 lib(`src/ontology.rs`) — capabilities·MCP 도구 목록은
  bin 의 단일 출처 함수 산출을 **값으로** 받는다(lib 이 bin 을 알 수 없으므로).
  이렇게 하면 유도 로직이 단위 테스트 가능하고, wasm 빌드에도 안전하다.
- **의미 사전.** IR 필드의 사람용 의미는 지식지도 사전이 아니라 **바이너리 안의
  기계 가독 원천**(ir_schema 의 description)에서 온다 — 지식지도 §2-2 는 봉투
  필드 사전이라 IR 필드와 축이 다르고, 마크다운을 실행 시점에 읽으면 그 자체가
  드리프트 원천이 된다. 없으면 생략(지어내기 금지) 원칙은 동일하게 지켰다.
- **프로필 등재.** `hwp_export_ontology` 는 업무 직무 도구가 아니라 자기서술
  계열이므로 선례(#3762/#3776/#3787/#3828/#3719 §6-4)를 따라
  `agent_profile_router_contract` 의 `meta_only_by_design` 에 사유와 함께 등재 —
  `개발통합`(필터 없음)으로만 접근한다.

## red → green

유도 코드(`push_ir_nodes`)에서 타입 하나(`TableCell`)를 누락시키는 변이 →
계약 ① `every_ir_definition_appears_as_a_class` 가 누락 이름을 열거하며 red:

```
IR 스키마 정의인데 온톨로지 클래스에 없는 것: ["TableCell"]
유도(src/ontology.rs push_ir_nodes)가 $defs 전수를 돌지 않고 있습니다.
```

변이 복원 후 13/13 green. (스냅샷 비교가 아니라 원천 대조라, 원천이 늘어도
테스트 수정 없이 전수성이 유지된다.)

## 검증 (이 PC, 2026-08-08)

| 게이트 | 결과 |
|---|---|
| `tests/ontology_contract.rs` (신규, 계약 ①~⑤ + 표면 배선) | 13/13 green |
| `cargo test --lib ontology` (단위 8) | 8/8 green |
| `capabilities_schema_contract` | 17/17 green |
| `capabilities_subcommands_contract` | 4/4 green |
| `agent_profile_router_contract` | 8/8 green |
| `mcp_server_contract` | 22/22 green |
| `provenance_contract` (SWEEP_EXEMPT·호출표 동반 갱신) | 9/9 green |
| `cli_json_contract` (help↔capabilities 드리프트 가드 포함) | 31/31 green |
| `cargo clippy --bin rhwp --lib --tests -- -D warnings` | green |
| `rustfmt --check` (변경 파일 7) | 실 diff 0 (CRLF 소음은 이 PC 전역 현상) |
| Node vitest (`bindings/node`, 패리티 포함) | 448/448 green |
| Node `npm run gen:check` (envelopes 재생성 후) | green — `ExportOntologyEnvelope` 편입 |
| Python pytest (`bindings/python`) | 260 passed, **1 failed(기존 red — #4196 범위**: explain·export-agent-manifest·export-plan-schema 래퍼 부재. export-ontology 는 아님) |

## 바뀐 파일

| 파일 | 내용 |
|---|---|
| `src/ontology.rs` (신규, 유도 코어 + 단위 8) | 클래스·속성·행위·신뢰 술어 유도, 봉투 조립 |
| `src/lib.rs` | `pub mod ontology` |
| `src/main.rs` | 디스패치·`cmd_export_ontology`·capabilities `cmd_json` 등재·MCP `hwp_export_ontology`(optionalArgs 규약)·help |
| `src/provenance.rs` | MAP 에 `export-ontology`(빈 목록 + 사유) |
| `tests/ontology_contract.rs` (신규) | 계약 ①전수 포섭 ②유도 정합 ③신뢰 술어 전수 ④JSON-LD 형식 ⑤실패 규약 + 표면 배선 2본 |
| `tests/provenance_contract.rs` | SWEEP_EXEMPT + 표지 존재 검사 호출표 |
| `tests/agent_profile_router_contract.rs` | `meta_only_by_design` 등재(사유 포함) |
| `bindings/node/src/commands.ts`·`index.ts` | `exportOntology` 래퍼·재수출 |
| `bindings/node/src/envelopes.ts` | gen:types 재생성 (봉투 34→35) |
| `bindings/node/test/commands.test.ts` | argv 회귀 2본 + 최소 호출 목록 편입 |
| `bindings/python/src/rhwp/commands.py`·`__init__.py` | `export_ontology` 래퍼·수출 |
| `bindings/python/tests/test_commands.py` | argv 회귀 |
| `bindings/python/tests/test_integration.py` | 패리티 가드 exported 집합에 `export-ontology` |

## 남긴 것 (후속)

- **O2 문서 인스턴스 모드** — `export-ontology <문서>` 로 특정 문서를 개체 그래프로.
- MCP resources 축(#3627 R74 계열)에 온톨로지 생성기 등재 — 이번 표면 계약 밖.
- #4226(annotations) 머지 시 자동 편입 — 유도가 모양 무가정으로 통째로 싣게 돼
  있어 후속 수정 없음(단위 `tool_annotations_are_carried_when_present` 가 고정).
