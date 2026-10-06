# T3 — MCP 도구 주석(annotations) 선언 (#4220, refs #3907)

## 무엇

`capabilities --mcp` 매니페스트와 `mcp-serve` `tools/list` 의 전 도구(무상태 43 + 세션 12)에
MCP 표준 `annotations` 4필드를 선언했다. 종전에는 0건이라, MCP 호스트가 "이 도구가 읽기
전용인가 / 원본을 덮는가"를 실행 전에 판정할 근거가 없었다.

## 스펙 출처

- https://modelcontextprotocol.io/specification/2025-06-18/server/tools — `Tool.annotations`
  ("optional properties describing tool behavior"; 클라이언트는 신뢰할 수 없는 서버의
  힌트를 신뢰하면 안 된다는 경고 포함).
- `modelcontextprotocol/schema/2025-06-18/schema.ts` — `ToolAnnotations`:
  `readOnlyHint`(기본 false) · `destructiveHint`(기본 true, readOnlyHint=false 일 때만 의미) ·
  `idempotentHint`(기본 false) · `openWorldHint`(기본 true). 2025-03-26 개정판 신설, 2025-06-18 유지.

스펙 기본값이 위험 쪽(destructive=true, openWorld=true)으로 기울어 있으므로, 기본값에
기대지 않고 4필드를 전부 명시한다 — `inputSchema.required` 를 빈 배열이라도 선언하는
기존 규약과 같은 이유다.

## 판정 규칙 (단일 출처에서 유도 — 손 나열 금지)

무상태 도구(`derive_mcp_tool_annotations`, src/main.rs):

| 필드 | 유도 근거 |
|---|---|
| `readOnlyHint` | 봉투 `outputFields` 에 산출 경로 필드(`output`/`outputDir`)가 **없으면** true. 출력이 선택인 도구(`hwp_table_to_csv`)는 "쓸 수 있다"는 이유로 false — 힌트는 안전 방향으로 보수적이어야 한다. |
| `destructiveHint` | cli 배선(`args`+`optionalArgs`)에 `--in-place` 축이 있을 때만 true. 그 밖의 쓰기는 산출 분리(-o) 원칙의 추가형이다. 현재 해당 도구는 **hwp_redact 하나**. |
| `idempotentHint` | 전부 true — 무상태 도구는 매 호출이 같은 원본에서 다시 계산하는 결정론 변환이라 같은 인자 재실행은 같은 산출을 다시 쓸 뿐이다. |
| `openWorldHint` | 전부 false — 로컬 파일만 다루며 네트워크 등 개방 세계 축이 없다. |

세션 도구(`session_tool_annotations`, src/mcp_serve.rs): 읽기/편집 경계는 프로필 경계의
단일 출처 `agent_profiles::SESSION_READ_TOOLS` 에서 유도하고, 그 표가 말하지 않는 축만 판정:

- `hwp_doc_render_page`/`hwp_doc_save` 는 read 표 소속 여부와 무관하게 파일을 쓰므로
  (inputSchema 의 `output` 속성) readOnlyHint=false.
- `hwp_doc_save` 만 destructiveHint=true — `output` 이 hwp_open 으로 연 **원본 경로일 수
  있고** `session_save` 에 같은 경로 거부가 없다. 무상태 `--in-place` 축의 세션판이다.
- `hwp_open` idempotent=false(호출마다 새 docId), `hwp_doc_replace_text` idempotent=false
  (이미 치환된 IR 위에 겹쳐 적용될 수 있다 — 매번 원본에서 다시 계산하는 무상태
  `hwp_replace_text` 가 true 인 것과 대비).

`tools/list` 의 무상태 도구 주석은 매니페스트 값을 **그대로 되비춘다** — 서버가 따로
판정하면 두 표면이 어긋난다.

## 동반 변경

- `capabilities_schema.rs`: `McpTool.annotations` + `McpToolAnnotations` 정의 추가,
  `CAPABILITIES_SCHEMA_VERSION` 1.1 → 1.2 (필드 추가 = minor, #4114 교훈 — 봉투에 새
  필드를 실으면 스키마 선언을 동반한다. 미동반 시 `mcp_schema_matches_live_manifest_output`
  가 잡는다).
- Node 바인딩: `gen:types` 는 `capabilities`(–mcp 아님)에서 생성하므로 드리프트 없음
  (gen:check 로 확인).

## 대조 테스트 (tests/mcp_tool_annotations_contract.rs)

① 전 도구 4필드 boolean 선언 + 스펙 밖 필드 금지 ② readOnlyHint ↔ 봉투 산출 경로
선언 정합 + category=edit 도구는 readOnly 불가(원천이 다른 두 선언의 교차 검증)
③ destructiveHint ↔ `--in-place` 배선 실물 + 황금 목록 `[hwp_redact]`
④ `tools/list` 되비춤 정합 + 세션 12종 도구별 판정 고정.

red 실증: hwp_redact 의 annotations 를 제거하는 변이 → ① red → 복원 green.

## 게이트

- cargo test: mcp_tool_annotations_contract(신규) · mcp_server_contract ·
  capabilities_schema_contract · capabilities_subcommands_contract ·
  agent_profile_router_contract — 전부 green.
- cargo clippy -D warnings, rustfmt(변경 파일) — 통과.
- bindings/node `gen:check` — 드리프트 없음.
