---
kind: investigation
status: active
canonical: mydocs/tech/agent_architecture/ontology_from_code.md
last_verified: 2026-08-08
---

# 코드에서 유도하는 온톨로지 — 손으로 쓰는 OaC 를 넘어서 (제안)

**질문**: rhwp 의 자기서술(IR 스키마·capabilities·출처 지도)을 **하나의 형식
온톨로지**로 기계 유도할 수 있는가 — 그리고 그것이 손으로 쓰는
"Ontology-as-Code" 보다 나은가.

이 문서는 **제안**이다. 어떤 항목도 로드맵 등급을 올리지 않으며, 채택·번호
부여·트랙 편입은 전부 메인테이너 몫이다(동향 조사 PR #4221 과 같은 성격).
조사 시점은 2026-08-08 이고, 출처 접속일도 별도 표기가 없으면 같은 날이다.
경쟁 제품·플랫폼은 실명 없이 **원리로만** 서술한다 — 근거 URL 은 문서 끝
출처 절에 있다.

**실측 기준** — 이 문서의 코드 인용·수치는 전부 `upstream/devel`
(`dc7d7adcc`, 2026-08-08 fetch) 워크트리에서 얻었다. 측정 명령을 함께 적었으니
믿기 전에 다시 찍어 보면 된다. 측정하지 않은 것은 §7에 "확인되지 않음"으로
적었다.

관련: 로드맵 전수 지도 [roadmap_atlas.md](roadmap_atlas.md) · 층 모델
[layer_model.md](layer_model.md) · 표준 트랙
[track_i_standards.md](../agent_roadmap/track_i_standards.md) · 발견 트랙
[track_d_discovery.md](../agent_roadmap/track_d_discovery.md) · refs #3907.

---

## 1. 논지 — 손으로 쓰는 온톨로지는 낡는다

### 1.1 바깥 동향

세 갈래가 뚜렷하다(출처는 문서 끝).

1. **선언적 온톨로지의 부상.** 지식그래프·GraphRAG 도구들이 온톨로지를
   "코드처럼" — 버전 관리되고, CI 로 검증되고, 선언 하나에서 하류 산출물이
   파생되는 1급 아티팩트로 — 다루는 흐름이 2025~26 에 뚜렷하다. Git 기반
   온톨로지 공학(모듈화·자동 구문/의미 검사·문서 생성·게시 파이프라인)이
   방법론으로 정리됐고, SHACL 검증을 CI 에 거는 도구가 여럿 나왔다.
2. **에이전트가 소비자다.** LLM 에이전트가 구조화 선언(스키마 마크업·지식
   그래프)을 직접 읽고 판단 근거로 쓰는 추세가 확립됐다. 온톨로지의 1차
   독자가 사람 온톨로지스트에서 **기계**로 옮겨 가고 있다.
3. **추출 자동화.** 코퍼스·스키마에서 온톨로지를 자동 구축하는 연구가
   활발하다 — LLM 기반 온톨로지 학습, 스키마 유도(schema induction),
   온톨로지 유도 지식그래프 구축. 공통 문제의식은 하나다: **손으로 만든
   온톨로지는 만들자마자 대상과 어긋나기 시작한다.**

### 1.2 손 OaC 의 구조적 문제 = 드리프트

"온톨로지를 코드로 관리한다"는 선언만으로는 정합이 생기지 않는다. 온톨로지
파일이 Git 에 있어도, 그것이 서술하는 실물(타입·명령·계약)과 **별도 파일**인
한 둘은 독립적으로 진화한다. 이 저장소가 문서 축에서 반복 확인한 패턴
그대로다:

- 같은 목록이 두 곳에 있으면 6개월 뒤 서로 다른 말을 한다
  (`src/provenance.rs:31` 모듈 주석이 이 원칙의 코드 쪽 진술이다).
- 검증할 수 없는 선언은 다음 사람이 지운다
  ([envelope_provenance.md](../envelope_provenance.md) §2.3).

바깥 생태계의 처방은 온톨로지 저장소에 CI 검증을 붙이는 것까지 왔다. 그러나
그 검증은 대부분 **온톨로지 내부 정합**(구문·제약 위반)이지, **온톨로지 ↔
실물 코드 정합**이 아니다. 실물과의 대사(對査)가 없는 온톨로지는 잘 검증된
채로 낡는다.

### 1.3 rhwp 의 위치 — 온톨로지의 원료가 이미 살아 있다

rhwp 는 "서술이 실물에서 나온다"는 원칙을 이미 여러 축에서 계약 테스트로
강제한다. 즉 **온톨로지를 손으로 쓸 필요가 없는 드문 위치**에 있다:

| 축 | 원료(단일 출처) | 규모(실측) | 정합 강제 |
|---|---|---|---|
| **타입** | `src/ir_schema.rs` `ir_schema()` — 공개 IR 의 JSON Schema | `$defs` **41개** (`src/ir_schema.rs:803-845`) | `every_ref_resolves_to_a_definition`·`definitions_are_reachable_from_root` (`src/ir_schema.rs:892·906`) |
| **필드 의미** | 각 `$defs` 정의의 `description` 문자열(코드 내장) + 봉투 필드 사전([agent_knowledge_map.md](../../manual/agent_knowledge_map.md) §2 — 표 198행 실측, §2-2 전수 사전 148필드) | IR 쪽은 전 필드 서술, 봉투 쪽은 §3.5 참조 | `tests/cli_json_contract.rs` (봉투 필드 추가-전용) |
| **행위(명령)** | `capabilities` 자기서술 — `cmd`/`cmd_json`/`cmd_gated` 선언 (`src/main.rs:1559-1590`) | `--json` 계약 명령 **34개**(`cmd_json` 호출부 실측) + 비계약 명령·게이트 명령 | `capabilities_covers_every_help_command` (`tests/cli_json_contract.rs`) |
| **행위(도구)** | `capabilities --mcp` 매니페스트 + `mcp-serve` `tools/list` | 무상태+세션 도구(v0.8.2 실측 51, devel 은 증가 — §7) | `tests/capabilities_schema_contract.rs` 외 |
| **신뢰 경계** | `src/provenance.rs::MAP` — 명령별 문서 파생 필드 + `origin` 근거 | 명령 **34개** 등재(`grep -c 'command: "' src/provenance.rs`) | `tests/provenance_contract.rs` — 실문서 토큰 출현으로 누락 판정 |
| **행위 성격** | MCP `annotations` 4필드 — 기존 선언에서 유도(PR #4226, **리뷰 중**) | 전 도구 4필드 | `tests/mcp_tool_annotations_contract.rs` (해당 PR) |
| **묶음 선례** | `export-agent-manifest` — 4축(capabilities·irSchema·provenanceMap·planSchema)을 왕복 1회로 조립 (`src/main.rs:13961`) | `missingAxes` 규약 포함 | 조립만 하고 로직 중복 없음(주석 명시) |

**공백은 두 가지다.** ① 이 원료들이 각자 자기 어휘(JSON Schema·자기서술
JSON·출처 지도)로 흩어져 있고, 이를 **하나의 형식 온톨로지**로 묶는 유도
층이 없다. ② 스키마(타입·행위) 수준만 있고, **개별 문서를 그 온톨로지의
인스턴스 그래프**로 내는 표면이 없다.

### 1.4 논지 요약

손 OaC: 온톨로지 파일이 실물과 별도로 존재 → 드리프트는 시간 문제 →
CI 는 내부 정합만 지킨다.

**유도 온톨로지**: 온톨로지가 파일이 아니라 **함수**다 — 매 호출 때 살아있는
선언(§1.3 원료)에서 다시 유도된다. 선언이 바뀌면 온톨로지는 다음 호출에서
이미 바뀌어 있다. 남는 위험은 "유도기 자체가 원료를 빠뜨리는 것"뿐이고,
그것은 전수 계약 테스트(§5)가 잡는다. 드리프트가 **줄어드는** 게 아니라
**성립할 자리가 없다.** 이는 rhwp 가 봉투 표지(`provenance::marked`,
`src/provenance.rs:507`)와 자기서술에서 이미 검증한 설계 원칙의 재적용이다.

---

## 2. 어휘 선택 — JSON-LD 1.1 `@context` 기반

### 2.1 후보 대조

| 후보 | 성격 | 판정 |
|---|---|---|
| **JSON-LD 1.1** | RDF 의 JSON 직렬화. W3C Recommendation(2020-07, 2025 워킹그룹 재차터로 유지보수 계속) | **채택** — §2.2 |
| OWL 2 (Turtle/RDF-XML) | 추론용 공리 어휘. 열린 세계 가정 | 1차 비채택, **매핑 여지 유지** — §2.3 |
| SHACL | 그래프 제약 검증. 닫힌 세계 | 비채택 — 제약 검증은 이미 JSON Schema + 계약 테스트가 수행. 필요 시 파생 산출로 |
| schema.org 어휘 | 웹 문서 마크업용 공용 어휘 | 부분 — 자체 어휘(`rhwp:`)를 1차로 하고, 대응 가능한 술어만 주석으로 병기 검토 |
| 자체 JSON(현행 유지) | capabilities·ir-schema 그대로 | 이미 있음 — 형식 온톨로지가 아니라서 외부 그래프 도구·표준 소비자와 접점이 없다 |

### 2.2 JSON-LD 를 고르는 근거

1. **소비자가 에이전트다.** rhwp 봉투는 전부 JSON 이고, 소비자(에이전트·
   바인딩·MCP 호스트)는 JSON 파서만 갖고 있다. JSON-LD 는 **유효한 JSON
   그대로** RDF 그래프가 된다 — 그래프 도구가 없는 소비자는 평범한 JSON
   으로 읽고, 그래프 도구가 있는 소비자는 트리플로 올린다. 두 소비층을
   한 산출물로 만족시키는 직렬화는 이것뿐이다. 2025~26 의 "구조화 데이터를
   LLM/에이전트가 직접 소비" 동향의 사실상 표준 직렬화이기도 하다(출처 절).
2. **자기완결이 가능하다.** `@context` 를 봉투 안에 **내장**하면 네트워크
   해석 없이 완결된다 — 외부 컨텍스트 URL 해석을 요구하지 않는 것은
   의존성 0·오프라인 원칙과 정합한다.
3. **기존 계약과 겹치지 않는다.** JSON Schema(구조 검증)와 JSON-LD(의미
   선언)는 역할이 다르고 같은 JSON 문서에 공존 가능하다. `ir_schema()` 를
   대체하는 게 아니라 **그 위에 의미 층을 유도**한다.

### 2.3 OWL·SHACL 은 매핑 여지로만

- 클래스·속성 술어는 RDFS 어휘(`rdfs:Class`·`rdf:Property`·`rdfs:domain`·
  `rdfs:range`·`rdfs:comment`)로 충분하다. OWL 공리(제약·추론)는 1차
  소비자(에이전트)에게 당장 필요가 없고, OWL 의 열린 세계 가정과 검증의
  닫힌 세계 가정 사이 부정합은 알려진 함정이다(출처 절). 다만 유도 규칙이
  RDFS 부분집합에 머무는 한 `rdfs:Class → owl:Class` 승격은 기계적이므로,
  외부 요구가 실증되면 `--owl` 파생 모드를 추가할 여지를 막지 않는다.
- SHACL shape 역시 JSON Schema 에서 기계 파생이 가능하지만, 검증 축은 이미
  JSON Schema + 계약 테스트가 지키므로 **요구가 실증되기 전에는 만들지
  않는다**(제안 절제 원칙, PR #4221 선별 기준 승계).

### 2.4 네임스페이스

IR 스키마 `$id`(`https://github.com/edwardkim/rhwp/schema/ir/1.0`,
`src/ir_schema.rs:852`)와 같은 계열로:

```
"@context": {
  "rhwp": "https://github.com/edwardkim/rhwp/ontology/1.0#",
  "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
  "xsd":  "http://www.w3.org/2001/XMLSchema#", …
}
```

이 IRI 는 **식별자이지 참조 의무가 아니다** — 해석하지 않아도 봉투만으로
완결된다(§2.2-2).

---

## 3. 유도 규칙 — 전부 기존 선언에서, 손 서술 0

원칙 하나: **온톨로지 전용으로 손으로 쓰는 서술은 0 이다.** 아래 규칙은
전부 §1.3 의 원료를 기계 변환한다. 원료에 없는 의미는 온톨로지에도 없다 —
공백은 §3.5 처럼 공백으로 명시한다.

### 3.1 타입 축 — IR 타입 → 클래스

`ir_schema()` 의 `$defs` 항목 하나가 클래스 하나다.

| JSON Schema 원료 | 온톨로지 요소 |
|---|---|
| `$defs` 키(예: `TableCell`) | `rhwp:TableCell`, `@type: rdfs:Class` |
| 정의의 `description` | `rdfs:comment` (한국어 원문 그대로) |
| `Document` 루트(`$ref`) | `rhwp:Document` 에 `rhwp:root: true` |

### 3.2 필드 축 — 필드 → 속성 (타입·의미는 사전에서)

각 정의의 `properties` 항목 하나가 속성 하나다. 타입과 의미는 **선언에 이미
실려 있는 사전** — IR 쪽은 `ir_schema.rs` 의 `description` 문자열, 봉투 쪽은
`recordFields`(§3.5) — 에서 온다.

| JSON Schema 원료 | 온톨로지 요소 |
|---|---|
| 속성 키(예: `rowSpan`) | `rhwp:TableCell-rowSpan`, `@type: rdf:Property`, `rdfs:domain: rhwp:TableCell` |
| `type: integer, minimum: 0` (`uint`, `src/ir_schema.rs:39`) | `rdfs:range: xsd:nonNegativeInteger` |
| `type: string`/`number`/`boolean` | `xsd:string`/`xsd:double`/`xsd:boolean` |
| `$ref: #/$defs/X` | `rdfs:range: rhwp:X` (객체 속성) |
| `array` + items | `rdfs:range` 는 원소 타입, `rhwp:container: "set"` |
| `enum_of` 값·뜻 목록 (`src/ir_schema.rs:65`) | `rhwp:enumValue` 목록(값+뜻 쌍) |
| `required` 목록 | `rhwp:required: true` |
| 속성 `description` | `rdfs:comment` |

유도 예시(발췌 — 실제 산출은 유도기가 만든다):

```json
{ "@id": "rhwp:TableCell-rowSpan", "@type": "rdf:Property",
  "rdfs:domain": { "@id": "rhwp:TableCell" },
  "rdfs:range": { "@id": "xsd:nonNegativeInteger" },
  "rhwp:required": true,
  "rdfs:comment": "세로 병합 칸 수 (1 = 병합 없음)" }
```

### 3.3 행위 축 — 명령·도구 → 행위 클래스

`capabilities` 자기서술의 명령 항목 하나가 행위 노드 하나다.

| 자기서술 원료 | 온톨로지 요소 |
|---|---|
| 명령 이름 | `rhwp:cmd-search`, `@type: rhwp:Command` |
| `category`·`summary` | `rhwp:category` · `rdfs:comment` |
| `json`·`batch`·`flags` | `rhwp:jsonContract`·`rhwp:batch`·`rhwp:flag` |
| `recordFields` | `rhwp:emits` → 봉투 필드 항 (§3.5) |
| `requiresFeature`·`available` (`cmd_gated`) | `rhwp:requiresFeature`·`rhwp:available` |
| MCP 도구(`capabilities --mcp`) | `rhwp:tool-hwp_search`, `@type: rhwp:Tool`, `rhwp:wraps` → 대응 `rhwp:Command` |
| edit·inspect 하위 명령 (`EDIT_SUBCOMMANDS` 등) | `rhwp:subcommandOf` 로 부모에 연결 |

**annotations 가 성격 술어다** — PR #4226 이 유도해 둔 MCP `annotations`
4필드(`readOnlyHint`·`destructiveHint`·`idempotentHint`·`openWorldHint`)를
`rhwp:readOnly`·`rhwp:destructive`·`rhwp:idempotent`·`rhwp:openWorld` 로
그대로 싣는다. 그 PR 이 머지되기 전에는 이 축을 **싣지 않고**
`missingAxes: ["annotations"]` 로 부재를 광고한다(`export-agent-manifest`
의 `missingAxes` 규약 승계, `src/main.rs:13958` 주석). null 로 채우면
"값이 없음"과 "축이 아직 없음"을 소비자가 구분할 수 없다.

### 3.4 신뢰 축 — provenance → 신뢰 술어

`src/provenance.rs::MAP` 의 명령별 선언에서:

| 출처 지도 원료 | 온톨로지 요소 |
|---|---|
| `untrusted[].path` (예: `matches[].text`) | 해당 봉투 필드 항에 `rhwp:documentDerived: true` |
| `untrusted[].origin` (근거 문자열) | `rhwp:origin` — 근거 없는 신뢰 선언은 계약 위반이라는 기존 원칙([envelope_provenance.md](../envelope_provenance.md))을 온톨로지에도 그대로 |
| 빈 `untrusted` + `note` | 행위 노드에 `rhwp:documentDerived: false` 와 그 근거 |

신뢰 표지는 **속성(필드) 정의에 붙는다** — 인스턴스 노드마다 반복하지
않는다. "이 필드는 문서를 만든 사람이 값을 정한다"는 성질은 필드의
성질이지 개별 값의 성질이 아니고, 인스턴스 쪽 봉투에는 기존 규약대로
`untrustedContent`/`untrustedFields` 표지가 실린다(§4.3).

### 3.5 정직한 공백 — 봉투 필드의 의미 서술

`recordFields` 는 **이름 목록**이다(`&[&str]`, `src/main.rs:1566-1575`) —
타입도 의미 서술도 코드 선언에 없다. 사람용 사전은
[agent_knowledge_map.md](../../manual/agent_knowledge_map.md) §2(표 198행
실측: `sed -n '/^## 2\./,/^## 3\./p' … | grep -c '^| \``'`, §2-2 전수 사전
148필드)에 있지만 그것은 **문서이지 기계 선언이 아니다.**

따라서 1차(O1) 봉투 필드 항은 이름·등장 명령(`rhwp:emits` 역링크)·신뢰
술어(§3.4)만 갖고, `rdfs:comment` 는 **비워 둔다**. 손으로 채우는 순간
손 OaC 로 퇴행한다. 의미 서술을 원하면 경로는 하나다 — `recordFields` 를
이름+타입+한 줄 뜻의 구조 선언으로 승격(capabilities 스키마 minor)하고
지식 지도 사전은 그 선언의 사람용 조판으로 역전시키는 것. 이는 별도
제안이며 O1 의 선행 조건이 아니다.

---

## 4. 표면 설계 — `export-ontology` 2모드

### 4.1 모드 1 (기본) — 스키마 온톨로지

```
rhwp export-ontology --json          # 전체(타입+행위+신뢰 축)
rhwp export-ontology --bare --json   # 최상위 표지 없이 온톨로지 본문만
```

문서를 입력으로 받지 않는다 — `capabilities`·`export-ir-schema` 와 같은
부류다. 봉투(제안):

```json
{ "schemaVersion": "1.0",
  "ontologyVersion": "1.0",
  "format": "json-ld",
  "mode": "schema",
  "irSchemaVersion": "1.0",
  "classCount": 41, "propertyCount": 0, "commandCount": 0,
  "missingAxes": [],
  "ontology": { "@context": { … }, "@graph": [ … ] },
  "untrustedContent": false, "untrustedFields": [] }
```

- `ontologyVersion` 은 **유도 규칙의 버전**이다(§3 표가 바뀌면 올린다).
  원료의 버전(`irSchemaVersion` 등)은 별도 필드로 함께 실어, 소비자가
  "규칙이 바뀌었나, 원료가 바뀌었나"를 구분한다 — 스키마 버전 대사는
  트랙 I R83 과 한 몸으로 판단할 사안이다.
- `classCount` 등 수치는 위 예시값이 아니라 유도 결과의 실측이 실린다.

### 4.2 모드 2 — 문서 인스턴스 그래프

```
rhwp export-ontology <문서.hwp> --json
rhwp export-ontology <문서.hwp> -p 3 --json   # 쪽 좁히기(기존 -p 관례)
```

문서 하나를 모드 1 온톨로지의 **인스턴스 그래프**로 낸다. IR 노드 하나가
그래프 노드 하나이고, `@type` 은 모드 1 이 정의한 클래스만 쓴다(닫힌 어휘 —
§5.2). `@id` 는 IR 주소 체계(구역/문단 좌표)에서 유도한 불투명 IRI
(`urn:rhwp:doc:s0-p3` 꼴 제안 — 로컬 경로를 IRI 에 싣지 않는다).

```json
{ "schemaVersion": "1.0", "mode": "instances", "source": "문서.hwp",
  "ontologyVersion": "1.0", "irSchemaVersion": "1.0",
  "nodeCount": 0,
  "graph": { "@context": { … }, "@graph": [
    { "@id": "urn:rhwp:doc:s0-p3", "@type": "rhwp:Paragraph", … } ] },
  "untrustedContent": true,
  "untrustedFields": ["graph.@graph[].rhwp:text", …] }
```

문서 본문이 실리므로 `untrustedContent: true` 와 실제 실린 경로의
`untrustedFields` 가 **반드시** 붙는다 — 표지 생략 봉투를 새로 만들지
않는다(트랙 A R5 봉합의 재발 방지). `provenance.rs::MAP` 에
`export-ontology` 항목을 함께 등재한다.

### 4.3 봉투·exit·MCP 관례 (전부 기존 규약)

- **exit**: 0 성공 / 1 런타임(파싱·직렬화 실패) / 2 사용법(알 수 없는
  옵션, 모드 1 에 문서 인자+`--bare` 충돌 등) — R1 의 0/1/2/3 계약
  그대로. 판정용 exit 3 은 이 명령에 없다(비교 명령이 아니다).
- **표지**: `provenance::marked` 경유로 두 모드 다 표지를 싣는다.
- **자기서술**: `cmd_json` 항목 추가 + `recordFields` 등재 →
  `capabilities_covers_every_help_command`·recordFields 전수 대조가
  자동으로 물게 한다. `--help` 동반 갱신.
- **MCP**: `hwp_export_ontology` 무상태 도구 1개(두 모드는 인자 유무로
  구분 — CLI 계약의 얇은 껍데기 원칙). annotations 는 readOnly true·
  destructive false·idempotent true·openWorld false 로 유도될 것이다
  (PR #4226 규칙 적용 시).
- **조립 선례와의 관계**: `export-agent-manifest` 의 다섯 번째 축으로
  스키마 온톨로지를 실을지는 O1 머지 뒤 별도 판단(봉투 크기 실측 선행).

---

## 5. 계약 — 정합을 강제하는 테스트 (제안: `tests/ontology_contract.rs`)

### 5.1 전수성 — 유도 온톨로지 ⊇ 전 원료

1. **전 IR 타입**: `ir_schema()` 의 `$defs` 를 순회하며 각 이름의 클래스
   노드가 존재함을 단언한다. 41 을 하드코딩하지 않는다 — 원료를 순회해야
   원료가 늘 때 테스트가 따라온다(recordFields 전수 대조 R6 과 같은 꼴).
2. **전 명령·도구**: `capabilities` 자기서술의 모든 명령 항목·모든 MCP
   도구에 대응하는 행위 노드가 존재함을 단언한다.
3. **전 신뢰 선언**: `provenance::MAP` 의 모든 `untrusted[].path` 가
   대응 필드 항의 `rhwp:documentDerived`/`rhwp:origin` 으로 나타남을
   단언한다.

### 5.2 문서 인스턴스 ↔ IR 왕복 정합

1. **닫힌 어휘**: 인스턴스 그래프의 모든 `@type` 이 모드 1 클래스 집합의
   원소다 — 스키마에 없는 타입을 쓰는 순간 red.
2. **노드 대응**: `samples/` 실문서에 대해 IR 노드 수(구역·문단·컨트롤)와
   그래프 노드 수의 대응을 단언한다. 그래프에만 있는 노드도, IR 에만 있는
   노드도 없어야 한다.
3. **값 왕복**: 문단 텍스트·표 격자 등 대표 값이 IR 값과 문자 단위로
   같음을 단언한다 — 그래프는 IR 의 재서술이지 재해석이 아니다.

### 5.3 컨텍스트 무결

`@graph` 에 등장하는 모든 술어가 내장 `@context` 에 정의되어 있음을
단언한다 — 끊어진 `$ref` 가 코드 생성기를 망가뜨리듯, 미정의 술어는
그래프 소비자를 망가뜨린다(`every_ref_resolves_to_a_definition` 의
온톨로지판).

### 5.4 왜 이것으로 드리프트가 구조적으로 불가능한가

온톨로지는 저장된 파일이 아니라 매 호출 유도되는 함수 산출이다(§1.4).
어긋날 수 있는 자리는 유도기의 **누락**뿐인데, §5.1 이 원료 쪽을 순회하며
전수 대조하므로 누락은 CI 에서 red 다. 손 OaC 의 "온톨로지 따로, 실물
따로"라는 어긋남의 전제 자체가 없다.

---

## 6. 단계 — O1 → O2 → O3

| 단계 | 내용 | 착수 게이트 | DoD |
|---|---|---|---|
| **O1 유도 층** | 모드 1(스키마 온톨로지): §3 규칙으로 타입·행위·신뢰 축 유도. annotations 축은 PR #4226 머지 전이면 `missingAxes` | 원료 안정 — `ir_schema`·`capabilities`·`provenance::MAP` 모두 머지된 계약(충족). 본 설계의 메인테이너 방향 승인 | `export-ontology --json`/`--bare` green + §5.1·§5.3 계약 테스트 + 자기서술·`--help`·지식 지도 1행 등재 + fmt/clippy/release-test |
| **O2 문서 인스턴스** | 모드 2(인스턴스 그래프) + `provenance::MAP` 등재 + `untrustedFields` 표지 | O1 머지. `@id` 주소 체계 판정(구역·문단 좌표의 안정성 — 편집 후 재조판과의 관계 명시) | §5.2 왕복 정합 테스트가 `samples/` 실문서로 green + 표지 계약 테스트 |
| **O3 코퍼스 집계** | `batch` 축 편입(문서 여러 개 → NDJSON 레코드) + 레코드 병합으로 코퍼스 지식그래프 구성 | O2 머지 + **소비자 실증 1건**(집계 그래프로 실질 질의 — 예: "이 코퍼스에서 누름틀 서식 문서만" — 가 기존 표면 대비 이득임을 실측) | batch 레코드 규약 green + 병합 산출 실측 기록 |

단계 번호는 의존성 순서지 일정이 아니다(#3907 운영 규칙 3 승계). O3 은
소비자 실증 없이는 착수하지 않는다 — 요구가 실증되기 전의 표면은 제안이
아니라 소망이다(PR #4221 선별 기준 4).

---

## 7. 확인되지 않음 / 하지 않는 것

- **확인되지 않음**: devel 헤드의 MCP 도구 수(v0.8.2 실측 51, PR #4226
  본문은 무상태 43+세션 12 를 말한다 — 릴리스 빌드로 재실측 전에는 이
  문서에서 확정하지 않는다). 온톨로지 봉투의 크기(직렬화 실측 전).
  JSON-LD 소비 도구와의 상호운용(외부 트리플 스토어 적재는 시도하지
  않았다).
- **하지 않는 것**: 코드를 짜지 않는다(이 문서는 설계다). 기존 표면을
  대체하지 않는다 — `export-ir-schema`·`capabilities`·
  `export-provenance-map` 은 그대로이고 온톨로지는 그 위의 유도 층이다.
  추론(reasoning) 엔진을 들이지 않는다 — 의존성 0 원칙. 온톨로지 전용
  손 서술을 도입하지 않는다(§3.5).

---

## 출처 (조사 노트 — 접속일 2026-08-08)

동향·어휘 서술의 근거다. 본문은 원리로만 서술했고, 실명 비교는 하지 않는다.

**선언적 온톨로지·지식그래프 동향**

- 온톨로지 르네상스와 지식그래프 실무 논의(2025~26):
  <https://linkeddataorchestration.substack.com/p/the-ontology-issue-from-knowledge>
- Ontology Learning and KG Construction — RAG 성능 관점 비교(arXiv 2511.05991, 2025-11):
  <https://arxiv.org/abs/2511.05991>
- LLM-Driven Ontology Construction for Enterprise KGs(arXiv 2602.01276, 2026-02):
  <https://arxiv.org/abs/2602.01276>
- LLM-empowered KG construction survey(arXiv 2510.20345, 2025-10):
  <https://arxiv.org/abs/2510.20345>
- 스키마 유도·온톨로지 유도 구축(arXiv 2604.03496 · 2604.20795, 2026-04):
  <https://arxiv.org/abs/2604.03496> · <https://arxiv.org/abs/2604.20795>

**Git·CI 기반 온톨로지 공학**

- 온톨로지 검증 파이프라인·Git 워크플로 방법론(ISWC 2023):
  <https://iswc2023.semanticweb.org/wp-content/uploads/2023/11/142660133.pdf>
- SHACL 검증을 문서 빌드·CI 에 거는 도구 사례:
  <https://github.com/ISE-FIZKarlsruhe/ontoink>
- AI-네이티브 온톨로지 엔진(Rust·검증·버저닝) 사례:
  <https://github.com/fabio-rovai/open-ontologies>

**어휘·직렬화**

- JSON-LD 1.1, W3C Recommendation: <https://www.w3.org/TR/json-ld11/>
- JSON-LD WG 2025 재차터(유지보수 계속): <https://w3c.github.io/json-ld-charter-2025/>
- OWL(열린 세계) vs SHACL(닫힌 세계) 트레이드오프:
  <https://dfrnt.com/blog/2023-06-10-exploring-tradeoffs-rdf-owl-sparql-shacl-terminusdb-make-an-informed-decision>
- Semantic Web 회고·전망(LLM 과 지식그래프 접점, arXiv 2412.17159, 2024-12):
  <https://arxiv.org/abs/2412.17159>
- 에이전트의 구조화 데이터(JSON-LD) 소비 실무 동향(2025~26):
  <https://almcorp.com/blog/structured-data-for-llms-technical-guide/> ·
  <https://www.openhermit.com/blog/json-ld-schema-agents-2026>
