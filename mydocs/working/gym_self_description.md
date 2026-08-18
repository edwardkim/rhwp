---
kind: working
status: active
canonical: mydocs/working/gym_self_description.md
last_verified: 2026-08-18
---

# 자기서술 gym pack 확장 (SD13+) — 작업 기록

Issue: [#5217](https://github.com/edwardkim/rhwp/issues/5217)
PR: [#5226](https://github.com/edwardkim/rhwp/pull/5226)
브랜치: `feat/gym-selfdesc-expand`

## 1. 무엇을

`gym/packs/self-description` 에 SD13 이후 과제와 짝 기준풀이를 더하고, pack
README·pack 전용 계약 테스트·이 작업 문서를 남긴다. 대상은 문서가 아니라
**도구가 스스로 내놓는 계약**이다.

하지 않은 것:

- 새 CLI 명령·새 플래그·새 pack
- `gym/README.md` · `gym/PARK.md` · `gym/profiles/*` · 다른 pack
- `gym/core/checks.py` 연산자 추가
- `cargo fmt --all` (Rust 변경 없음)

## 2. 왜

SD01–SD07 은 입구다. 명령 수, 읽기·쓰기 형식 수, 매니페스트의 빈 축, 출처
지도에 실린 명령 수, 온톨로지 클래스·속성, 계획 스키마 정의 수, IR 스키마
산출. 에이전트가 "도구가 있다"는 것은 알지만, **어느 축이 무슨 버전인지**,
**`--json` 실패 규약이 무엇인지**, **도움 검색이 무엇을 메아리치는지**,
**MCP 호스트가 베끼지 않고 등록할 신원이 무엇인지**는 아직 묻지 않는다.

SD08–SD12 가 그 구멍을 다섯 자리 메웠다 (MCP 도구 수, 레지스트리 축 수,
`schema` 검색 적중, `actionCount`, 종료 코드 수). 그래도 레지스트리 네 축의
이름·버전, `jsonContract` 문구, batch 계약, 검색 키워드  Diversification,
스키마 `$ref` 뿌리는 비어 있다. 에이전트가 도구를 고를 때 쓰는 자기서술
계약이 얇으면, 다음 행동은 추측이 된다.

이 확장은 그 얇은 자리를 **기존 명령만**으로 채운다.

## 3. 명령 가족 — 단일 출처

과제가 부르는 명령은 아래뿐이다. 문서 편집·표 추출·검색은 없다.

| 명령 | 소스 | 이 pack 이 묻는 자리 |
| --- | --- | --- |
| `capabilities` | `src/main.rs` `capabilities_value` | `tool` · `schemaVersion` · `schemaRegistry.*` · `jsonContract.*` · `batch.*` · `exitCodes` · `formats` · `commands` |
| `capabilities --mcp` | `mcp_manifest_value` | `protocol` · `server.*` · `invocation.*` · `tools` · `profiles` |
| `capabilities --search K --json` | `show_capabilities_search` | `commands` 길이 · `search` 메아리 |
| `export-agent-manifest` | `agent_manifest_value` | `schemaVersion` · `missingAxes` · `capabilities.commands` · `irSchema.$defs` |
| `export-ontology` | `src/ontology.rs` `envelope` | `schemaVersion` · `classCount` · `propertyCount` · `actionCount` · 산출물 |
| `export-plan-schema` | `src/plan_schema.rs` `envelope` | `planSchemaVersion` · `dialect` · `definitionCount` · `schema.$ref` · 산출물 |
| `export-ir-schema` | `src/ir_schema.rs` `envelope` | `irSchemaVersion` · `definitionCount` · `schema.$ref` · dialect · 산출물 |
| `export-capabilities-schema` | `src/capabilities_schema.rs` `envelope` | `capabilitiesSchemaVersion` · `definitionCount` · `schema.$ref` · `mcpSchema.$ref` · 산출물 |
| `export-provenance-map` | `src/provenance.rs` `map_json` | `commands` · `envelopeFlags` · `pathSyntax` · `tool` |

`pack.json` 의 `requires.commands` 에 `export-ir-schema` 와
`export-capabilities-schema` 를 더했다. SD07 이 이미 전자를 쓰고, SD13+ 가
후자를 쓴다. 요구 목록에 없으면 옛 바이너리가 과제를 0점으로 받아 부재를
실패로 위장한다. runner 신원(`rhwpVersion`/`rhwpCommit`/`capabilitiesSha256`)은
손대지 않았다.

## 4. 과제 설계

### 4.1 레이어

1. **입구 (SD01–SD07)** — 이미 있다. 손대지 않았다.
2. **한 겹 (SD08–SD12)** — 이미 있다. 손대지 않았다.
3. **레지스트리 축 (SD13–SD25)** — 네 축의 이름·버전·표면·정책 경로.
   `axes[n]` 순서 가정은 `schema_registry.rs` 의 고정 집합과 같다. 축을
   끼워 넣으면 이 과제가 깨지는 것이 의도다.
4. **전역 계약 (SD26–SD38)** — `jsonContract` 와 `batch`. 개별 명령
   `recordFields` 가 아니라 모든 `--json` 이 공유하는 규약.
5. **도움 검색 (SD39–SD45)** — 키워드 export/inspect/edit/ontology/provenance/plan
   과 질의 메아리. SD10 의 `schema` 검색을 같은 형식으로 넓힌다.
6. **MCP 신원 (SD46–SD51)** — SD08 의 도구 수 위에 protocol·서버 이름·
   프로필 수·transport·stdin 도구·stdio 서버 명령.
7. **스키마 메타 (SD52–SD70)** — 버전·dialect·정의 수·`$ref`.
8. **산출물 (SD71–SD74)** — SD07 형식. 숫자만 맞추고 본문을 내지 않는
   제출을 `file_exists` + `differs_from_input` + `json_value_eq` 로 거절.

### 4.2 연산자 선택

| 연산자 | 쓰는 곳 | 이유 |
| --- | --- | --- |
| `answer_eq` | 스칼라 값 | 채점 시점에 같은 명령을 다시 돌려 대조 |
| `len_answer_eq` | 배열·객체 길이 | 숫자를 박제하지 않는다 |
| `file_exists` | 산출물 | 빈 파일 거부 (`minBytes: 256`) |
| `differs_from_input` | 산출물 | 입력 HWP 복사 거부 |
| `json_value_eq` | 산출물 고정 상수 | dialect·버전·`$ref` |

쓰지 않은 것: `deep_contains` (전역 훑기), `cell_text_eq` (표 좌표),
`value_eq` (답 제출 없이 하드코딩 — 이 pack 은 에이전트가 값을 내도록
`answer_eq` 를 쓴다).

### 4.3 기준풀이

답 과제:

```json
{
  "id": "SD29",
  "steps": [
    {
      "answer": {
        "fields": {
          "cmd": ["capabilities"],
          "path": "jsonContract.provenance.fields",
          "len": true
        }
      }
    }
  ]
}
```

산출 과제:

```json
{
  "id": "SD71",
  "steps": [
    {
      "run": [
        "export-capabilities-schema",
        "-o",
        "{sub:capabilities-schema.json}"
      ]
    }
  ]
}
```

`build_baseline.py` 가 `{sub:}` 를 제출 폴더로 치환하고, `score.py` 가 같은
pack 경로에서 채점한다. 기준풀이 `cmd`/`path` 가 과제 검사와 다르면
`test_gym_self_description_pack.py` 가 막는다.

### 4.4 입력 파일

모든 과제의 `input` 은 `samples/table-001.hwp` 다. 자기서술 명령은 문서를
열지 않는다. gym 스키마가 입력을 요구하므로 자리만 채운다. 산출 과제의
`differs_from_input` 은 이 자리 입력이 산출물과 바이트가 같으면 실패한다 —
스키마 JSON 이 HWP 와 같을 수 없으므로 판별력은 유지된다.

## 5. 과제 전표 (SD13–SD74)

### 5.1 신원·레지스트리

| ID | 제목 | cmd | path | op |
| --- | --- | --- | --- | --- |
| SD13 | 도구 이름 | `capabilities` | `tool` | `answer_eq` |
| SD14 | 봉투 schemaVersion | `capabilities` | `schemaVersion` | `answer_eq` |
| SD15 | crate 버전 | `capabilities` | `schemaRegistry.crateVersion` | `answer_eq` |
| SD16 | 버전 정책 경로 | `capabilities` | `schemaRegistry.policy` | `answer_eq` |
| SD17 | envelope 축 이름 | `capabilities` | `schemaRegistry.axes[0].axis` | `answer_eq` |
| SD18 | ir 축 이름 | `capabilities` | `schemaRegistry.axes[1].axis` | `answer_eq` |
| SD19 | capabilities 축 이름 | `capabilities` | `schemaRegistry.axes[2].axis` | `answer_eq` |
| SD20 | plan 축 이름 | `capabilities` | `schemaRegistry.axes[3].axis` | `answer_eq` |
| SD21 | envelope 축 버전 | `capabilities` | `schemaRegistry.axes[0].version` | `answer_eq` |
| SD22 | ir 축 버전 | `capabilities` | `schemaRegistry.axes[1].version` | `answer_eq` |
| SD23 | capabilities 축 버전 | `capabilities` | `schemaRegistry.axes[2].version` | `answer_eq` |
| SD24 | plan 축 버전 | `capabilities` | `schemaRegistry.axes[3].version` | `answer_eq` |
| SD25 | envelope 축 표면 | `capabilities` | `schemaRegistry.axes[0].surface` | `answer_eq` |

기대 상수 (소스, 라이브로 재확인):

- `tool` = `"rhwp"`
- `schemaVersion` = `"1.0"` (`ENVELOPE_SCHEMA_VERSION`)
- `schemaRegistry.policy` = `"mydocs/tech/agent_runtime/version_policy.md"`
- 축 이름 순서 = `envelope`, `ir`, `capabilities`, `plan`
- 축 버전 = `1.0`, `1.0`, `1.3`, `1.1`

### 5.2 jsonContract·batch

| ID | 제목 | path | op |
| --- | --- | --- | --- |
| SD26 | stdout 규약 | `jsonContract.stdout` | `answer_eq` |
| SD27 | schemaPolicy | `jsonContract.schemaPolicy` | `answer_eq` |
| SD28 | failure 규약 | `jsonContract.failure` | `answer_eq` |
| SD29 | 출처 표지 필드 수 | `jsonContract.provenance.fields` | `len_answer_eq` |
| SD30 | 출처 지도 조회 | `jsonContract.provenance.map` | `answer_eq` |
| SD31 | textSecurity 종류 수 | `jsonContract.textSecurity.kinds` | `len_answer_eq` |
| SD32 | textSecurity 상태 수 | `jsonContract.textSecurity.status` | `len_answer_eq` |
| SD33 | textSecurity 정책 | `jsonContract.textSecurity.policy` | `answer_eq` |
| SD34 | batch 하위명령 수 | `batch.subcommands` | `len_answer_eq` |
| SD35 | batch 플래그 수 | `batch.flags` | `len_answer_eq` |
| SD36 | batch MCP 노출 수 | `batch.mcp.available` | `len_answer_eq` |
| SD37 | batch 입력 규약 | `batch.input` | `answer_eq` |
| SD38 | batch 순서 규약 | `batch.ordering` | `answer_eq` |

`jsonContract.provenance.fields` 는 `untrustedContent` 와 `untrustedFields` 두
개다. `textSecurity.kinds` 는 다섯 (`confusableFieldName`·`mixedScript`·
`bidiControl`·`invisibleChar`·`ansiEscape`). `textSecurity.status` 는
`clean`·`warning`. 이 숫자도 박제하지 않고 길이로 센다.

### 5.3 도움 검색

검색은 대소문자 무시 부분 문자열이고, 공백으로 나눈 여러 단어는 AND 다.
하위 명령 이름·요약도 haystack 에 들어간다 (`#3884 G4`). `--search` 와
`--mcp` 는 함께 쓸 수 없다 — 과제도 섞지 않는다.

| ID | 키워드 | 묻는 자리 |
| --- | --- | --- |
| SD39 | export | `commands` 길이 |
| SD40 | inspect | `commands` 길이 |
| SD41 | edit | `commands` 길이 |
| SD42 | ontology | `commands` 길이 |
| SD43 | provenance | `commands` 길이 |
| SD44 | plan | `commands` 길이 |
| SD45 | schema | `search` == `"schema"` |

SD10 과 SD45 는 같은 키워드를 쓰지만 묻는 자리가 다르다. 하나는 적중
집합의 크기, 하나는 질의 메아리다. 제목도 다르다.

### 5.4 MCP

| ID | path | 기대 (참고, 라이브 재계산) |
| --- | --- | --- |
| SD46 | `protocol` | `"mcp"` |
| SD47 | `server.suggestedName` | `"rhwp"` |
| SD48 | `profiles` 길이 | `agent_profiles::names()` |
| SD49 | `invocation.transport` | `"cli"` |
| SD50 | `invocation.stdinTools` 길이 | 3 (`hwp_batch`·`hwp_batch_search`·`hwp_batch_extract_data`) |
| SD51 | `invocation.server` | `"mcp-serve"` |

### 5.5 스키마 메타

| ID | 명령 | path |
| --- | --- | --- |
| SD52 | `export-plan-schema --json` | `planSchemaVersion` |
| SD53 | `export-plan-schema --json` | `dialect` |
| SD54 | `export-plan-schema --json` | `definitionCount` |
| SD55 | `export-ir-schema --json` | `irSchemaVersion` |
| SD56 | `export-ir-schema --json` | `definitionCount` |
| SD57 | `export-capabilities-schema --json` | `capabilitiesSchemaVersion` |
| SD58 | `export-capabilities-schema --json` | `definitionCount` |
| SD59 | `export-capabilities-schema --json` | `dialect` |
| SD60 | `export-ontology --json` | `schemaVersion` |
| SD61 | `export-agent-manifest --json` | `capabilities.commands` 길이 |
| SD62 | `export-agent-manifest --json` | `irSchema.$defs` 길이 |
| SD63 | `export-provenance-map --json` | `envelopeFlags` 키 수 |
| SD64 | `export-provenance-map --json` | `pathSyntax` |
| SD65 | `export-ir-schema --json` | `schema.$ref` |
| SD66 | `export-plan-schema --json` | `schema.$ref` |
| SD67 | `export-capabilities-schema --json` | `schema.$ref` |
| SD68 | `export-capabilities-schema --json` | `mcpSchema.$ref` |
| SD69 | `export-agent-manifest --json` | `schemaVersion` |
| SD70 | `export-provenance-map --json` | `tool` |

`$ref` 기대:

- IR: `#/$defs/Document`
- 계획: `#/$defs/Plan`
- capabilities: `#/$defs/Capabilities`
- MCP: `#/$defs/McpManifest`

dialect 는 세 스키마 모두 `https://json-schema.org/draft/2020-12/schema`.

SD61 은 SD01 과 같은 명령 수를 **매니페스트 조립 경로**로 다시 센다.
두 경로가 어긋나면 매니페스트가 capabilities 를 그대로 싣지 않는 것이고,
그것은 `#3828 B2` 계약 위반이다. 과제가 그 드리프트를 드러낸다.

### 5.6 산출물

| ID | 파일 | 고정 대조 |
| --- | --- | --- |
| SD71 | `capabilities-schema.json` | dialect, `capabilitiesSchemaVersion=1.3`, `schema.$ref=#/$defs/Capabilities` |
| SD72 | `plan-schema.json` | dialect, `planSchemaVersion=1.1`, `schema.$ref=#/$defs/Plan` |
| SD73 | `ontology.json` | `schemaVersion=1.0` |
| SD74 | `ir-schema.json` | dialect, `irSchemaVersion=1.0`, `schema.$ref=#/$defs/Document` |

`-o` 를 주면 명령은 봉투 전체(표지 포함)를 pretty JSON 으로 쓴다.
`--bare` 는 쓰지 않았다. bare 본문은 `dialect` 최상위가 없고 `$schema` 만
있어, SD07 이 이미 쓰는 봉투 경로와 어긋난다.

`export-agent-manifest` 는 `-o` 가 없다. 매니페스트는 답 과제(SD03·SD61·
SD62·SD69)로만 묻는다.

## 6. 고정 상수와 라이브 값의 경계

박제해도 되는 것 (소스 상수, 계약 테스트가 이미 고정):

- 도구 이름 `rhwp`
- 봉투 `schemaVersion` `1.0`
- 레지스트리 축 이름 집합과 순서
- dialect URL
- `$ref` 뿌리
- `capabilitiesSchemaVersion` `1.3`
- `planSchemaVersion` `1.1`
- `irSchemaVersion` `1.0`
- MCP `protocol` / `transport` / `invocation.server`

박제하면 안 되는 것 (명령 표면이 늘면 같이 늘음):

- `commands` 길이, MCP `tools` 길이
- `--search` 적중 수
- `classCount` / `propertyCount` / `actionCount`
- `definitionCount` (IR·계획·capabilities)
- `batch.subcommands` / `batch.flags` / `profiles`

이 확장은 후자를 전부 `answer_eq`/`len_answer_eq` 로 두어, 채점 바이너리가
정답을 다시 계산하게 했다.

## 7. 검증

로컬에서 수행한 것:

```text
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs scripts.tests.test_gym_self_description_pack
```

수행하지 않은 것:

- `python gym/tools/build_baseline.py --pack self-description` (바이너리
  왕복). 검사 경로와 기준풀이 cmd/path 는 SD01–SD12 형식과 같고, 봉투
  필드는 소스에서 확인했다.
- `cargo fmt --all -- --check` — Rust 변경 없음, 생략.
- `cargo test` / clippy / 시각 검증 — 해당 없음.

## 8. 파일 목록

| 경로 | 역할 |
| --- | --- |
| `gym/packs/self-description/pack.json` | requires 에 스키마 명령 2개 추가. runner 불변 |
| `gym/packs/self-description/README.md` | pack 정본 안내 |
| `gym/packs/self-description/tasks/SD13.json` … `SD74.json` | 신규 과제 |
| `gym/packs/self-description/reference/SD13.json` … `SD74.json` | 짝 기준풀이 |
| `scripts/tests/test_gym_self_description_pack.py` | pack 전용 계약 테스트 |
| `mydocs/working/gym_self_description.md` | 이 문서 |

기존 SD01–SD12 과제·기준풀이 내용은 바꾸지 않았다.

## 9. 위험과 의도된 깨짐

- **축 순서.** `axes[0]` 이 `envelope` 가 아니면 SD17·SD21·SD25 가 실패한다.
  레지스트리에 축을 끼워 넣는 변경은 capabilities minor 이고, 그 변경을
  과제가 드러내는 것이 맞다. 과제를 느슨한 `value_in` 으로 바꾸지 않는다.
- **검색 문구.** 명령 요약을 고치면 `--search ontology` 적중이 줄 수 있다.
  기준풀이가 같은 명령을 쓰므로 채점은 따라간다. 사람이 적은 노트의
  숫자를 믿지 말 것.
- **스키마 버전 고정.** SD71 의 `1.3` 은
  `schema_registry::CAPABILITIES_SCHEMA_VERSION` 과 같다. 버전을 올리는
  커밋은 이 과제와 `test_gym_self_description_pack.py` 의 기댓값을 함께
  고쳐야 한다.
- **매니페스트 조립.** SD61 이 SD01 과 다른 수를 내면
  `export-agent-manifest` 가 capabilities 를 그대로 싣지 않는 것이다.
  그 차이를 삼키지 않는다.
- **프로필 필터.** `--profile` 과제는 넣지 않았다. 프로필이 늘거나 도구
  배정이 바뀌면 적중 집합이 흔들리고, 이 pack 의 축(자기서술 계약)이 아니라
  프로필 정책 축이 된다.

## 10. 다른 pack 과의 경계

| pack | 이 pack 과의 관계 |
| --- | --- |
| `core-cli` | 문서에 대한 조사·추출. 도구 자신은 묻지 않는다. |
| `automation` | 계획·캡슐·서명. 계획 **스키마**는 여기, 계획 **실행**은 저기. |
| `security` | 문서 안의 은닉·주입·PII. 출처 **표지 계약**은 여기. |
| `studio-e2e` | 스튜디오 e2e 의 문서 데이터 계약. 자기서술과 무관. |

`starter` 프로파일은 이미 `core-cli` + `self-description` 이다. 과제만
늘어나고 pack id 는 그대로라 프로파일을 고칠 필요가 없다.

## 11. 후속

- 로컬 `build_baseline` 왕복을 한 번 돌려 scorecard 를 남기면, SD13+ 의
  "풀 수 있음"이 파일 정합을 넘어 실행으로 닫힌다.
- `--search` AND 다중 단어 과제는 키워드 선택이 흔들려 보류했다. 필요하면
  고정 구(`"export schema"`) 한 건만 넣는 편이 안전하다.
- `capabilities --mcp --profile <이름>` 은 프로필 pack 이 따로 생길 때
  그쪽으로 보내는 것이 맞다.

## 12. 작업 메모 — 경로 대조표

소스에서 확인한 봉투 키 (발췌):

`capabilities` 최상위:

- `schemaVersion`, `tool`, `version`, `schemaRegistry`, `formats`,
  `exitCodes`, `jsonContract`, `batch`, `commands`,
  (`untrustedContent`, `untrustedFields` — `provenance::marked` 가 부착)

`schemaRegistry`:

- `crateVersion`, `axes[]` (`axis`·`version`·`surface`·`bump`), `policy`

`jsonContract`:

- `stdout`, `schemaPolicy`, `failure`, `textSecurity`
  (`field`·`status`·`kinds`·`policy`·`surfaces`),
  `provenance` (`fields`·`meaning`·`map`·`policy`)

`batch`:

- `subcommands`, `flags`, `ordering`, `input`, `authentication`,
  `output`, `limit`, `mcp` (`available`·`excluded`), `exitAggregation`

`capabilities --mcp`:

- `schemaVersion`, `protocol`, `server` (`suggestedName`·`version`·
  `description`), `invocation` (`transport`·`note`·`stdinTools`·`server`),
  `tools`, `profile`, `profiles`

`capabilities --search --json`:

- `schemaVersion`, `tool`, `version`, `search`, `commands`

`export-*-schema` 공통:

- `schemaVersion`, `<축>SchemaVersion`, `dialect`, `definitionCount`,
  `schema` (`$ref`·`$defs`·…)

`export-capabilities-schema` 추가:

- `mcpSchema`

`export-ontology`:

- `schemaVersion`, `ontology` (`@context`·`@graph`), `classCount`,
  `propertyCount`, `actionCount`

`export-agent-manifest`:

- `schemaVersion`, `capabilities`, `irSchema`, `provenanceMap`,
  `planSchema`, `missingAxes`

`export-provenance-map`:

- `schemaVersion`, `tool`, `version`, `envelopeFlags`, `pathSyntax`,
  `policy`, `commands`

이 표에 없는 자리(`commands[i].recordFields` 전수, MCP `tools[i].annotations`
전수)는 이번 확장에 넣지 않았다. 전수는 명령이 늘 때마다 길이만 의미 있고,
개별 인덱스는 순서가 계약이 아니기 때문이다. 길이는 이미 SD01·SD08·SD61 이
센다.
