---
name: rhwp-mcp-session
description: rhwp 를 MCP 서버(mcp-serve)로 에이전트 호스트에 붙이고 세션·무상태 도구를 고르는 통합 규약입니다. .mcp.json 등록, 세션 수명(hwp_open→hwp_doc_*→hwp_close)과 무상태 선택 기준, resources(스키마·레시피·문서) 소비, capabilities --mcp 가 도구 정의의 단일 출처라는 계약, 판정 3층 오류 복구를 다룹니다. 트리거 — 사용자가 "rhwp 를 MCP로 붙여/등록해", "mcp-serve", ".mcp.json", "세션으로 문서 열어", "hwp_open/hwp_doc_*", "재파싱 없이 반복 조회", "MCP 도구 목록/스키마/레시피 리소스", "프로필로 도구 좁혀" 등을 요청할 때. gym 트레이스가 아니라 실 에이전트 부착용. 전체 통합 절차는 mydocs/manual/mcp_integration_guide.md.
---

# rhwp-mcp-session — 실 에이전트 MCP 부착 Skill

## 목적

`rhwp mcp-serve` 를 MCP 호스트에 붙이고, **무상태 도구와 세션 도구를 올바르게 골라**
재파싱 비용 없이 문서를 다룬다. 이 스킬은 gym 벤치·트레이스가 아니라 **실제 에이전트
호스트**가 rhwp 를 도구로 쓰는 규약이다. 새 CLI 를 만들지 않고, 없는 도구 이름을
발명하지 않는다.

권위 출처:

- [`mydocs/manual/mcp_integration_guide.md`](../../../mydocs/manual/mcp_integration_guide.md)
- [`mydocs/manual/mcp_attach_kit.md`](../../../mydocs/manual/mcp_attach_kit.md)
- [`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md) §6
- [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md) §capabilities·§mcp-serve
- 소스 상수: `src/main.rs` `mcp_tool_definitions()`, `src/agent_profiles.rs` `ALL_SESSION_TOOLS`

상세는 [`references/`](references/README.md). 기계 픽스처는
[`references/fixtures/`](references/fixtures/).

## 0. 30초 판단

| 질문 | 예 | 다음 |
|---|---|---|
| 호스트에 아직 안 붙였는가? | Claude Code / Cursor / VS Code | [등록](#등록-mcpjson) → [`host_attach.md`](references/host_attach.md) |
| 도구 이름이 기억나지 않는가? | "세션 검색이 뭐였지" | **`capabilities --mcp` / `tools/list`**. 추측 금지. [`capabilities_ssot.md`](references/capabilities_ssot.md) |
| 호출 하나면 끝나는가? | 쪽수, PDF, 검색 1회 | **무상태**. [`stateless_when.md`](references/stateless_when.md) |
| 같은 문서를 반복 조회·편집하는가? | 검색 3회, 채움+눈검증 | **세션** `hwp_open` → `hwp_doc_*` → `hwp_close`. [`session_lifecycle.md`](references/session_lifecycle.md) |
| 폴더/목록인가? | 스윕, 메일머지 | 무상태 배치 `hwp_batch*` |
| 실패했는가? | isError / nextCall / identical | 층부터 가른다. [`error_recovery.md`](references/error_recovery.md) |

**개수를 외우지 마라.** 문서와 바이너리가 다르면 바이너리가 이긴다.

## 등록 (.mcp.json)

```jsonc
// 프로젝트 루트 .mcp.json — Claude Code 등 A형 호스트
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- `rhwp` 가 PATH 에 없으면 `command` 에 **절대 경로** (`C:/…/target/release/rhwp.exe`).
- 전송은 stdio JSON-RPC 2.0 뿐. 포트·인증·URL 이 없다. stdin EOF 에서 종료.
- 직무가 정해져 있으면 `"args": ["mcp-serve", "--profile", "행정서식"]`.
- 코퍼스 인벤토리가 필요하면 `"args": ["mcp-serve", "--workspace", "C:/abs/corpus"]`.
- VS Code 는 B형(`servers` + `type: stdio`). Zed/Goose 는 킷을 본다.

세션 이득은 **같은 서버 프로세스**가 여러 `tools/call` 을 받을 때만 난다.
호스트가 호출마다 새 `mcp-serve` 를 띄우면 `hwp_open` 이 이득이 없다.

## 단일 출처 계약 — `capabilities --mcp`

도구 정의는 `mcp_tool_definitions()` **한 곳**에서 나온다.

| 표면 | 내용 | 세션 도구 |
|---|---|---|
| `rhwp capabilities --mcp` | 무상태 선언 | **없음** (서버 전용이라) |
| `mcp-serve` `tools/list` | 위 선언 + 세션 | **여기가 세션 정본** |
| `rhwp://capabilities/mcp` | `--mcp` 와 동일 | 없음 |

- 호스트가 도구 목록을 손으로 베끼면 rhwp 가 바뀔 때 조용히 낡는다.
- 어긋남은 `tests/mcp_server_contract.rs::tools_list_matches_capabilities_manifest`.
- `--json` 명령이 늘었는데 MCP 에서 빠지면 `capabilities_mcp_covers_every_json_command`.
- 함수콜 클라이언트는 `cli.args` 의 `{path}` 를 `inputSchema` 같은 이름 값으로 치환한다.

```bash
rhwp capabilities --mcp | jq '.tools[] | {name, description}'
rhwp capabilities --mcp --profile 행정서식

printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | rhwp mcp-serve
```

이름이 없으면 `didYouMean` → `tools/list` → 무상태 CLI. **`hwp_doc_…` 를 만들지 마라.**

### 프로필 — 작은 모델일수록 좁혀서 문다

`--profile` 은 추천이 아니라 **서버가 제공하는 집합의 경계**다. 목록에 없는 세션
도구는 `tools/call` 로도 우회할 수 없다. 실측 7종: `경영보고` · `행정서식` ·
`데이터분석` · `콘텐츠제작` · `아카이브검색` · `품질검증` · `개발통합`(필터 없음).
없는 이름은 실행 전 거부 (`오류: 알 수 없는 프로필`).

## 세션 수명 — `hwp_open` → `hwp_doc_*` → `hwp_close`

```jsonc
→ hwp_open              {"path":"C:/절대/경로/편람.hwp"}     // 파싱 1회 → docId
← {"docId":"doc-1","pageCount":393, …}
→ hwp_doc_search        {"docId":"doc-1","query":"위임전결"}  // 재파싱 없음
→ hwp_doc_fill_fields   {"docId":"doc-1","data":{"회사명":"페타플로"}}
→ hwp_doc_render_page   {"docId":"doc-1","page":0,"output":"C:/abs/out/p0.svg"}
→ hwp_doc_save          {"docId":"doc-1","output":"C:/abs/out/저장본.hwp","verify":true}
→ hwp_close             {"docId":"doc-1"}
```

계약:

- 인자 이름은 **`docId`** (`handle` 이 아니다).
- **유일한 문서 기록 지점은 `hwp_doc_save`.** 편집은 인메모리 누적. 저장 없이 닫으면 사라진다.
- `hwp_doc_render_page` 는 호출자가 준 SVG 경로에만 쓴다. 원본 HWP 를 덮지 않는다.
- `hwp_doc_save` 의 `output` 이 원본 경로일 수 있다 (`destructiveHint=true`).
- 저장 후에도 핸들은 OPEN. 이어서 편집·재저장 가능.
- 핸들 수명 = 서버 프로세스 수명. 재시작 후 `doc-1` 을 재사용하지 않는다.
- 닫힌/모르는 `docId` → `isError:true` + `nextCall{name:"hwp_open"}`.
- 워크스페이스 기동 시 `hwp_ws_list` → `hwp_ws_open`(id=w1..) 이 `hwp_open` 의 id 축.

세션 도구 전수는 `ALL_SESSION_TOOLS` / `tools/list` 가 정본이다. 카드는
[`session_tools.md`](references/session_tools.md), 짝은 [`pairing.md`](references/pairing.md).
지식 지도 §6-2 가 소스보다 짧으면 **소스가 이긴다** (`hwp_doc_structure` ·
`hwp_doc_extract_data` 는 조회 파리티 축으로 소스에 있다).

## 무상태냐 세션이냐

| 상황 | 선택 | 근거 |
|---|---|---|
| 호출 하나 = 작업 하나 | **무상태** (`hwp_info` · `hwp_search` · `hwp_fill_fields` …) | CLI 계약의 얇은 껍데기 |
| 같은 문서 반복 조회·편집 | **세션** | 재파싱이 사라진다 |
| 대형 문서(수백 쪽) 다회 접근 | 세션 | 실측 387쪽: 검색 3회+info 세션 310ms vs CLI 810ms |
| 파일 목록 일괄 | `hwp_batch` · `hwp_batch_search` · `hwp_batch_extract_data` | NDJSON |
| 세션에 짝이 없는 동사 | 무상태만 | PDF · redact · run · ir-diff · convert. 이름을 만들지 말 것 |

판단 트리: [`decision_tree.md`](references/decision_tree.md).
언제 무상태: [`stateless_when.md`](references/stateless_when.md).

## resources

`resources/list` → `resources/read`. 본문은 바이너리 `include_str!` — 설치본에서도 동작.

| URI | 무엇 |
|---|---|
| `rhwp://capabilities/mcp` | 무상태 선언 (= `--mcp`) |
| `rhwp://docs/llms.txt` · `rhwp://docs/agent_knowledge_map.md` · `rhwp://docs/agent_troubleshooting_guide.md` | 진입점·지도·실패 사전 |
| `rhwp://recipes/01…` | 완주 레시피 |
| `rhwp://schemas/ir` · `plan` · `capabilities` | JSON Schema 생성기 직결 |

프로필은 리소스 **목록**을 필터하지 않는다. 매니페스트 **내용**은 프로필로 렌더된다.

## 판정 3층 — `isError` 만 보면 오독한다

| 층 | 신호 | 재시도 |
|---|---|---|
| JSON-RPC | `error{code,message}` | 금지. 프로토콜을 고친다 |
| 도구 실패 | `isError:true` | 닫힌 핸들만 `nextCall` 로 `hwp_open`. exit 2 는 인자 수정 |
| 봉투 | `isError:false` + 필드 | `identical`/`notFound`/`invalid`/`nextOffset` 을 게이트로 |

차이 발견·부분 실패는 오류가 아니라 데이터다. `hwp_ir_diff` 의 `identical:false`,
`hwp_doc_fill_fields` 의 `notFound`, `replacedCount:0`, `hwp_run_plan` 의
`invalid != []`(MCP 는 isError:false) 를 성공으로 읽지 마라.

복구 의사코드와 실측 바늘은 [`error_recovery.md`](references/error_recovery.md).

## 절차

1. `.mcp.json` 등록 → 호스트 재시작 → `initialize` / `tools/list`.
2. 온보딩은 자기서술: `rhwp://capabilities/mcp` 또는 `rhwp capabilities --mcp` 1회 캐시.
3. 1회성 = 무상태, 반복·대형 = `hwp_open` 세션. 폴더 = `hwp_batch*`.
4. 편집은 인메모리 누적 → `hwp_doc_render_page`(changedPages) → `hwp_doc_save` verify → `hwp_close`.
5. 막히면 `rhwp://docs/agent_troubleshooting_guide.md` §14.

## 함정 (실측된 것만)

1. **상대 경로는 서버 cwd 기준.** MCP 로는 절대 경로만.
2. **`hwp_batch*` 는 `structuredContent=null`.** `content[0].text` 를 NDJSON 으로.
3. **`batch convert` 는 MCP 미노출.** `capabilities.batch.mcp.excluded` 가 이유.
4. **`password` 는 `writeOnly`.** 응답·세션에 안 남긴다. 호스트 telemetry 는 별개.
   stdin 경로 목록을 쓰는 batch 는 password 미지원.
5. **`-` 로 시작하는 검색어:** MCP `hwp_search` 는 배선에 `--` 가 있다. `query` 그대로.
6. **세션 도구는 `--mcp` 선언에 없다.** 전체 목록은 `tools/list`. 수치가 문서와 다르면 바이너리.
7. **쪽 기준:** `hwp_doc_text`/`hwp_doc_render_page` 의 `page` 는 0 기준.
   `hwp_split_document` 의 from/to 만 1 기준.
8. **이어보기:** `truncated` 가 아니라 `nextOffset` 이 "더 있는가"의 판정이다.

## 하지 않는 것

- 새 CLI / 새 MCP 도구 이름 발명.
- gym 팩·트레이스를 이 스킬의 실행 경로로 쓰기.
- 다른 스킬(rhwp-onboarding / safe-edit / provenance / doc-triage)의 책임을 여기로 끌어오기.
- 호스트 설정에 도구 목록을 하드코딩하기.

## 픽스처·검증

- `references/fixtures/allowlist.json` — 소스에서 추출한 세션·무상태 이름.
- `references/fixtures/traces/` — 수명·복구·무상태 한 방 시나리오.
- `references/fixtures/errors/` — 층별 복구 카드.
- `references/fixtures/decisions/` — 세션/무상태 선택.
- 가드: `python -m unittest scripts.tests.test_agent_mcp_session`

레퍼런스 목차: [`references/README.md`](references/README.md).
작업 기록: [`mydocs/working/agent_mcp_session.md`](../../../mydocs/working/archives/agent_mcp_session.md).
