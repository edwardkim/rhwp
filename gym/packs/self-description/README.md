---
kind: guide
status: active
canonical: gym/packs/self-description/README.md
last_verified: 2026-08-18
---

# self-description — 도구가 스스로를 설명하는 계약

## 왜 이 pack 인가 (온램프)

에이전트가 rhwp 를 처음 만나면 문서가 아니라 **도구 자신**을 읽어야 한다.
어느 명령을 부를지, 봉투에 어떤 필드가 실리는지, 스키마 버전이 어디 축인지,
MCP 호스트에 무엇을 등록할지 — 이 정보는 위키에 있지 않다. `capabilities` 와
스키마 계열 명령이 그 단일 출처다.

`starter` 프로파일이 이 pack 을 `core-cli` 와 묶는 이유이기도 하다. 본문·표를
고치기 전에, 도구가 스스로 신고하는 표면을 숫자와 경로로 읽을 수 있어야 한다.
SD01–SD07 은 그 입구(명령 수·형식·매니페스트·출처 지도·온톨로지·계획 스키마·
IR 스키마)다. SD08–SD12 는 MCP·레지스트리·도움 검색·행위 수·종료 코드를
한 겹 더 묻는다. SD13+ 는 같은 명령 가족 안에서 **축 이름·계약 문구·검색
키워드·스키마 `$ref`** 까지 내려가, "capabilities 를 한 번 훑었다"가 아니라
"봉투의 어느 자리가 무슨 뜻인지 짚을 수 있다"를 채점한다.

이 pack 은 새 CLI 를 만들지 않는다. 새 pack 도 만들지 않는다. runner 신원은
기존 `pack.json` 을 그대로 쓴다. 요구 명령은 자기서술 가족만 늘린다
(`export-ir-schema`, `export-capabilities-schema`).

## 명령 가족 — 이 pack 이 쓰는 것

| 명령 | 묻는 것 |
| --- | --- |
| `capabilities` | 명령 표면·형식·종료 코드·레지스트리·jsonContract·batch |
| `capabilities --mcp` | MCP 도구 매니페스트·서버 신원·프로필 |
| `capabilities --search <키워드> --json` | 이름·요약·하위명령 부분 문자열 검색 |
| `export-agent-manifest` | capabilities+IR+출처+계획 을 1회 조립 |
| `export-ontology` | JSON-LD 클래스·속성·행위 규모 |
| `export-plan-schema` | `run` 계획서 JSON Schema |
| `export-ir-schema` | 문서 IR JSON Schema |
| `export-capabilities-schema` | capabilities 자체와 MCP 매니페스트 스키마 |
| `export-provenance-map` | 봉투 필드가 문서에서 왔는지의 지도 |

`edit`·`export-tables`·`search` 같은 문서 작업 명령은 여기 없다. 대상이 문서가
아니라 도구 자신이기 때문이다. 입력 `samples/table-001.hwp` 는 gym 과제가
입력을 갖도록 하는 자리일 뿐, 자기서술 명령은 그 파일을 열지 않는다.

## 채점 계약

- **라이브 오라클.** `answer_eq` / `len_answer_eq` 는 채점 시점에 같은 `cmd` 를
  다시 돌려 기대값을 센다. 골든 숫자를 박제하지 않는다.
- **고정 계약은 `json_value_eq`.** dialect URL, `$ref`, `capabilitiesSchemaVersion`
  처럼 소스 상수가 있는 자리는 산출물 파일에서 직접 대조한다.
- **무편집 복사 거부.** 산출물 과제(SD07·SD71–SD74)는 `differs_from_input` 으로
  입력 HWP 를 그대로 낸 제출을 거절한다.
- **정답은 바이너리를 따른다.** MCP 도구 수·검색 적중 수·`actionCount` 는
  명령이 늘면 같이 는다. 기준풀이와 검사가 같은 명령을 쓰므로 채점은 따라간다.

## 재현

```bash
python gym/tools/audit.py
python -m unittest scripts.tests.test_gym_packs scripts.tests.test_gym_self_description_pack
# 바이너리가 있으면 (선택) 기준풀이 왕복
python gym/tools/build_baseline.py --agent baseline --pack self-description --bin target/debug/rhwp
python gym/score.py --agent baseline --pack self-description --bin target/debug/rhwp
```

`cargo fmt --all` 은 이 pack 과 무관하다. JSON·Markdown·Python 만 바뀐다.

## 과제 목록

### SD01–SD07 — 입구

| ID | 티어 | 축 | 명령 | 묻는 자리 |
| --- | --- | --- | --- | --- |
| SD01 | 1 | 조사 | `capabilities` | `commands` 길이 |
| SD02 | 1 | 조사 | `capabilities` | `formats.read` / `formats.write` 길이 |
| SD03 | 2 | 조사 | `export-agent-manifest --json` | `missingAxes` 길이 |
| SD04 | 2 | 조사 | `export-provenance-map --json` | `commands` 길이 |
| SD05 | 2 | 조사 | `export-ontology --json` | `classCount` · `propertyCount` |
| SD06 | 2 | 조사 | `export-plan-schema --json` | `definitionCount` |
| SD07 | 2 | — | `export-ir-schema` | 산출 `ir.json` dialect |

### SD08–SD12 — 한 겹 더

| ID | 티어 | 명령 | 묻는 자리 |
| --- | --- | --- | --- |
| SD08 | 2 | `capabilities --mcp` | `tools` 길이 |
| SD09 | 2 | `capabilities` | `schemaRegistry.axes` 길이 |
| SD10 | 2 | `capabilities --search schema --json` | `commands` 길이 |
| SD11 | 2 | `export-ontology --json` | `actionCount` |
| SD12 | 2 | `capabilities` | `exitCodes` 길이 |

### SD13–SD25 — 신원과 레지스트리 축

에이전트가 "버전이 몇인가"를 추측하지 않게, 레지스트리 네 축의 이름·버전·
표면·정책 경로를 각각 묻는다. 축 순서는 소스 계약으로 고정이다
(`envelope`, `ir`, `capabilities`, `plan`).

| ID | 제목 | 경로 |
| --- | --- | --- |
| SD13 | 도구 이름 | `tool` |
| SD14 | 봉투 schemaVersion | `schemaVersion` |
| SD15 | crate 버전 | `schemaRegistry.crateVersion` |
| SD16 | 버전 정책 경로 | `schemaRegistry.policy` |
| SD17 | envelope 축 이름 | `schemaRegistry.axes[0].axis` |
| SD18 | ir 축 이름 | `schemaRegistry.axes[1].axis` |
| SD19 | capabilities 축 이름 | `schemaRegistry.axes[2].axis` |
| SD20 | plan 축 이름 | `schemaRegistry.axes[3].axis` |
| SD21 | envelope 축 버전 | `schemaRegistry.axes[0].version` |
| SD22 | ir 축 버전 | `schemaRegistry.axes[1].version` |
| SD23 | capabilities 축 버전 | `schemaRegistry.axes[2].version` |
| SD24 | plan 축 버전 | `schemaRegistry.axes[3].version` |
| SD25 | envelope 축 표면 | `schemaRegistry.axes[0].surface` |

### SD26–SD38 — jsonContract 와 batch

`--json` 계약의 전역 규약과 batch 축 전체를 묻는다. 개별 명령의
`recordFields` 가 아니라, 모든 명령이 공유하는 stdout·실패·출처 표지·
문자열 보안·batch 입출력이다.

| ID | 제목 | 경로 |
| --- | --- | --- |
| SD26 | jsonContract.stdout | `jsonContract.stdout` |
| SD27 | jsonContract.schemaPolicy | `jsonContract.schemaPolicy` |
| SD28 | jsonContract.failure | `jsonContract.failure` |
| SD29 | 출처 표지 필드 수 | `jsonContract.provenance.fields` |
| SD30 | 출처 지도 조회 방법 | `jsonContract.provenance.map` |
| SD31 | textSecurity 종류 수 | `jsonContract.textSecurity.kinds` |
| SD32 | textSecurity 상태 수 | `jsonContract.textSecurity.status` |
| SD33 | textSecurity 정책 | `jsonContract.textSecurity.policy` |
| SD34 | batch 하위명령 수 | `batch.subcommands` |
| SD35 | batch 플래그 수 | `batch.flags` |
| SD36 | batch MCP 노출 축 수 | `batch.mcp.available` |
| SD37 | batch 입력 규약 | `batch.input` |
| SD38 | batch 순서 규약 | `batch.ordering` |

### SD39–SD45 — 도움 검색

`--search` 는 대소문자 무시 부분 문자열, 여러 단어는 AND 다. 하위 명령
이름·요약도 검색 대상이다. 적중 수는 명령이 늘면 같이 늘므로 라이브로 센다.
SD45 는 검색 봉투가 질의 문자열을 메아리치는지 본다 — 적중 집합만 보고
무엇을 검색했는지 잃는 소비자를 막는다.

| ID | 키워드 | 경로 |
| --- | --- | --- |
| SD39 | export | `commands` 길이 |
| SD40 | inspect | `commands` 길이 |
| SD41 | edit | `commands` 길이 |
| SD42 | ontology | `commands` 길이 |
| SD43 | provenance | `commands` 길이 |
| SD44 | plan | `commands` 길이 |
| SD45 | schema | `search` 메아리 |

### SD46–SD51 — MCP 매니페스트

SD08 이 도구 수만 묻는다면, 여기서는 호스트가 등록 파일을 베끼지 않고도
쓸 수 있는 신원 필드를 묻는다. `protocol` 은 항상 `mcp`, 권고 서버 이름은
`rhwp`, 전송은 `cli`, stdio 서버 명령은 `mcp-serve`.

| ID | 제목 | 경로 |
| --- | --- | --- |
| SD46 | MCP protocol | `protocol` |
| SD47 | MCP 서버 권고 이름 | `server.suggestedName` |
| SD48 | MCP 프로필 수 | `profiles` |
| SD49 | MCP invocation.transport | `invocation.transport` |
| SD50 | MCP stdin 도구 수 | `invocation.stdinTools` |
| SD51 | MCP stdio 서버 명령 | `invocation.server` |

### SD52–SD70 — 스키마 내보내기 메타

계획·IR·capabilities 스키마와 매니페스트·출처 지도의 **메타 필드**를 묻는다.
본문 `$defs` 를 다 읽지 않아도 규모(`definitionCount`)와 뿌리(`$ref`)를 대조할
수 있어야 코드 생성기가 빈 스키마를 즉시 거부한다.

| ID | 명령 | 경로 |
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

### SD71–SD74 — 스키마 산출물

숫자만 맞추고 본문을 내지 않는 제출을 거절한다. SD07 과 같은 형식이다.

| ID | 산출 | 고정 대조 |
| --- | --- | --- |
| SD71 | `capabilities-schema.json` | dialect · `capabilitiesSchemaVersion`=`1.3` · `schema.$ref`=`#/$defs/Capabilities` |
| SD72 | `plan-schema.json` | dialect · `planSchemaVersion`=`1.1` · `schema.$ref`=`#/$defs/Plan` |
| SD73 | `ontology.json` | `schemaVersion`=`1.0` |
| SD74 | `ir-schema.json` | dialect · `irSchemaVersion`=`1.0` · `schema.$ref`=`#/$defs/Document` |

## 기준풀이 형식

답 과제는 채점과 같은 `cmd`/`path` 를 기준풀이에 적어 라이브로 길다.

```json
{
  "id": "SD13",
  "steps": [
    {
      "answer": {
        "tool": { "cmd": ["capabilities"], "path": "tool" }
      }
    }
  ]
}
```

산출 과제는 `-o {sub:파일}` 로 제출 폴더에 쓴다.

```json
{
  "id": "SD71",
  "steps": [
    { "run": ["export-capabilities-schema", "-o", "{sub:capabilities-schema.json}"] }
  ]
}
```

## 정직한 경계

- **문서 인스턴스 온톨로지(O2)는 없다.** `export-ontology` 는 도구 자신의
  스키마 모드만 낸다. 특정 HWP 한 부를 개체로 서술하는 과제는 이 pack 이
  아니다.
- **새 검색 연산·새 플래그는 없다.** `--search` 의 AND 부분 문자열과
  `--mcp` 매니페스트만 쓴다. `--profile` 필터 과제는 프로필 이름이 늘면
  적중 집합이 바뀌어 판별력이 떨어지므로 넣지 않았다.
- **gym/README·PARK·profiles·다른 pack·checks.py 는 그대로다.** 과제 수
  표가 구식이 되는 것은 이 pack 의 README 가 정본이 되기 때문이다.
- **라이브 채점을 이 확장에서 다시 돌리지는 않았다.** 검사 경로와 기준풀이
  `cmd`/`path` 는 SD01–SD12 와 같고, 고정값은 소스 계약
  (`schema_registry.rs`·`capabilities_schema.rs`·`plan_schema.rs`·`ir_schema.rs`)
  에서 확인했다.

## 위험

- 바이너리 버전이 바뀌면 MCP 도구 수·검색 적중 수·`definitionCount`·
  `actionCount` 가 달라질 수 있다. 기준풀이와 검사가 같은 명령을 쓰므로
  채점은 따라가지만, 사람이 박제한 노트를 믿으면 안 된다.
- `capabilities --search` 적중 집합은 명령 이름·요약·하위 명령 문구에
  의존한다. 요약을 고치면 수가 줄 수 있다.
- `schemaRegistry.axes[n]` 순서 가정은 레지스트리 계약 테스트가 고정한다.
  축을 끼워 넣으면 이 pack 의 SD17–SD24 가 깨지는 것이 의도다 — 축 집합
  변경은 capabilities minor 이고, 과제가 그 변경을 드러내야 한다.
- `capabilitiesSchemaVersion`=`1.3` 같은 고정 대조는 소스 상수를 올린
  커밋에서 함께 고쳐야 한다. 스키마 major/minor 규약 자체는
  `mydocs/tech/agent_runtime/version_policy.md` 가 정본이다.

## 관련

- 이슈: [#5217](https://github.com/edwardkim/rhwp/issues/5217)
- 작업 기록: [`mydocs/working/gym_self_description.md`](../../../mydocs/working/gym_self_description.md)
- 계약 테스트: `scripts/tests/test_gym_self_description_pack.py`
- gym 공통 감사: `python gym/tools/audit.py`
