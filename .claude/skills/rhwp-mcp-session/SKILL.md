---
name: rhwp-mcp-session
description: rhwp 를 MCP 서버(mcp-serve)로 에이전트 호스트에 붙이고 세션·무상태 도구를 고르는 통합 규약입니다. .mcp.json 등록, 세션 도구(hwp_open→hwp_doc_*→hwp_close)와 무상태 도구의 선택 기준, resources(스키마·레시피·문서) 소비, capabilities --mcp 가 도구 정의의 단일 출처라는 계약을 다룹니다. 트리거 — 사용자가 "rhwp 를 MCP로 붙여/등록해", "mcp-serve", ".mcp.json", "세션으로 문서 열어", "hwp_open/hwp_doc_*", "재파싱 없이 반복 조회", "MCP 도구 목록/스키마/레시피 리소스", "프로필로 도구 좁혀" 등을 요청할 때. 전체 통합 절차는 mydocs/manual/mcp_integration_guide.md.
---

# rhwp-mcp-session — MCP 세션 통합 Skill

## 목적

`rhwp mcp-serve` 를 MCP 호스트(Claude Code 등)에 붙이고, **무상태 도구와 세션 도구를
올바르게 골라** 재파싱 비용 없이 문서를 다루게 한다. 도구 정의의 단일 출처 계약과
서버가 직접 주는 resources(스키마·레시피)를 함께 소비한다.

권위 출처: [`mydocs/manual/mcp_integration_guide.md`](../../../mydocs/manual/mcp_integration_guide.md),
[`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md) §6,
[`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md) §capabilities·§mcp-serve.

## 등록 (.mcp.json)

```jsonc
// 프로젝트 루트 .mcp.json — Claude Code 등 MCP 호스트
{ "mcpServers": { "rhwp": { "command": "rhwp", "args": ["mcp-serve"] } } }
```

- `rhwp` 가 PATH 에 없으면 `command` 에 **절대 경로**를 쓴다(예: `./target/release/rhwp`).
- 전송은 MCP 표준 stdio(줄 단위 JSON-RPC 2.0)뿐 — 포트·인증 설정이 없다. stdin EOF 에서 종료.
- 역할이 정해져 있으면 `"args": ["mcp-serve", "--profile", "행정서식"]` 처럼 도구를 좁힌다.

## 단일 출처 계약 — `capabilities --mcp`

도구 정의는 `src/main.rs` 의 `mcp_tool_definitions()` **한 곳**에서 나온다.

- `rhwp capabilities --mcp` 가 내는 선언(JSON)과 `mcp-serve` 의 `tools/list` 가 **같은
  코드**다. 어긋남은 계약 테스트
  (`tests/mcp_server_contract.rs::tools_list_matches_capabilities_manifest`)가 잡는다.
- 서버(호스트)가 도구 목록을 손으로 베껴 쓰면 rhwp 가 바뀔 때 조용히 낡는다 — 원천을
  도구 자신이 낸다. `--json` 계약 명령이 늘었는데 MCP 에서 빠지면 드리프트 가드
  (`capabilities_mcp_covers_every_json_command`)가 잡는다.
- MCP 가 아닌 함수콜 클라이언트는 선언을 직접 소비한다: `cli.args` 의 `{path}` 등
  자리표시자를 `inputSchema` 의 같은 이름 값으로 치환해 CLI 를 조립·실행한다.

```bash
rhwp capabilities --mcp | jq '.tools[] | {name, description}'   # 선언 확인
rhwp capabilities --mcp --profile 행정서식                        # 역할별로 좁힌 목록

# 호스트 없이 서버를 손으로 검증(핸드셰이크 → tools/list, 지식 지도 §0 실측 절차)
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"x","version":"0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' | rhwp mcp-serve
```

### 프로필 라우터 — 작은 모델일수록 좁혀서 문다

도구 51종을 전부 물리면 작은 모델은 도구 선택에서 진다. `--profile` 은 역할별 도구
목록과 권장 호출 순서(`profile.recipe[]`)를 함께 준다(v0.8.2 실측 7종):
`경영보고`·`행정서식`(세션 포함 20)·`데이터분석`·`콘텐츠제작`·`아카이브검색`(세션 포함 15)·
`품질검증`·`개발통합`(필터 없음, 51). 없는 이름은 실행 전에 막힌다
(`오류: 알 수 없는 프로필 …`).

## 무상태냐 세션이냐 — 선택 기준

| 상황 | 선택 | 근거 |
|---|---|---|
| 호출 하나 = 작업 하나 (1회성) | **무상태 도구** (`hwp_info`·`hwp_search`·`hwp_fill_fields` 등 39종) | CLI 계약의 얇은 껍데기 — 봉투·종료 코드 계약을 문자 그대로 재사용 |
| 같은 문서를 반복 조회·편집 | **세션 도구** (`hwp_open`→`hwp_doc_*`→`hwp_close`, 12종 실측) | 프로세스별 재파싱 비용이 사라진다 |
| 대형 문서(수백 쪽) 다회 접근 | 세션 | 실측(387쪽): 검색 3회+info 를 세션 **310ms** vs 무상태 CLI **810ms** |
| 파일 목록 일괄 처리 | `hwp_batch`·`hwp_batch_search` (stdin 도구) | NDJSON 스트림 — 아래 함정 ③ |

세션 흐름(모든 판정 필드는 무상태 대응 도구와 동형):

```jsonc
→ hwp_open        {"path":"C:/절대/경로/편람.hwp"}        // 파싱 1회 → docId 핸들
← {"docId":"doc-1","pageCount":393, …}
→ hwp_doc_search  {"docId":"doc-1","query":"위임전결"}     // 재파싱 없음
→ hwp_doc_fill_fields {"docId":"doc-1","data":{"회사명":"페타플로"}}   // 인메모리 편집
→ hwp_doc_render_page {"docId":"doc-1","page":0,"output":"out/p0"}    // 바뀐 쪽 눈검증
→ hwp_doc_save    {"docId":"doc-1","output":"out/저장본.hwp","verify":true}
→ hwp_close       {"docId":"doc-1"}
```

- **세션의 유일한 기록 지점은 `hwp_doc_save`** 다. `hwp_doc_*` 편집은 전부 인메모리
  누적이고, 저장하지 않고 닫으면 사라진다. 저장 후에도 핸들은 열려 있어 이어서 편집 가능.
- 핸들 수명 = 서버 프로세스 수명(영속 아님). 닫힌/모르는 `docId` 는
  `isError:true` + `nextCall{name:"hwp_open"}` 로 온다.
- 세션 12종 전수(지식 지도 §6-2): `hwp_open`·`hwp_doc_info`·`hwp_doc_text`·
  `hwp_doc_fields`·`hwp_doc_tables`·`hwp_doc_search`·`hwp_doc_render_page`·
  `hwp_doc_fill_fields`·`hwp_doc_replace_text`·`hwp_doc_set_cell`·`hwp_doc_save`·`hwp_close`.
  봉투 어휘는 무상태 대응 도구와 동형이다(`hwp_doc_search` ↔ `hwp_search`).

## resources 소비 — 서버가 문서·스키마를 직접 준다

`resources/list` → `resources/read` 로 별도 파일 접근 권한 없이 읽는다
(본문은 바이너리에 `include_str!` 로 내장 — 설치본에서도 동작, #3627).

| URI | 무엇 |
|---|---|
| `rhwp://capabilities/mcp` | 도구 선언 매니페스트(= `capabilities --mcp`) |
| `rhwp://docs/llms.txt` · `rhwp://docs/agent_knowledge_map.md` · `rhwp://docs/agent_troubleshooting_guide.md` | 진입점·지식 지도·실패 사전 |
| `rhwp://recipes/01…06` | 완주 레시피 6편(서식 채움·표 CSV 왕복·마스킹·미신뢰 문서 선검사·메일머지·시각 회귀) |
| `rhwp://schemas/ir` · `rhwp://schemas/plan` · `rhwp://schemas/capabilities` | JSON Schema 생성기 직결(`export-ir-schema`/`export-plan-schema`/`export-capabilities-schema` 동일 원천) |

프로필은 리소스 **목록**을 필터하지 않는다(공통 문서라서). 단, 매니페스트 리소스의
**내용**은 프로필로 렌더된다 — tools/list 에 없는 도구를 광고하지 않기 위해서다.

## 판정 3층 — `isError` 만 보면 오독한다

| 층 | 신호 | 예 |
|---|---|---|
| JSON-RPC 오류 | `error{code,message}` | 알 수 없는 메서드(-32601), `params` 구조 오류(-32602) |
| 도구 실행 실패 | `isError:true` | 없는 파일, 닫힌 핸들 재사용, 필수 인자 누락(CLI exit 1·2 대응) |
| 봉투 판정 | `isError:false` + 봉투 필드 | `identical:false`(CLI exit 3), `notFound`, 치환 0건 |

**차이 발견·부분 실패는 오류가 아니라 데이터다.** `hwp_ir_diff` 가 차이를 찾으면 서버는
`isError:false` 로 `{"identical":false,"diffCount":…}` 를 그대로 준다. exit 2 대응
(`isError:true` + 사용법 오류)은 호출 조립 버그이므로 재시도 대신 인자를 고친다.
이름을 틀리면 `didYouMean[]`·`nextCall{}` 교정 힌트가 실측으로 온다.

## 절차 (권장)

1. `.mcp.json` 등록(위) → 호스트 재시작으로 `initialize`/`tools/list` 확인.
2. 온보딩은 추측이 아니라 자기서술로: `rhwp://capabilities/mcp` 리소스(또는
   `rhwp capabilities`) 1회 캐시.
3. 1회성 호출은 무상태 도구, 반복·대형 문서는 `hwp_open` 세션.
4. 편집은 인메모리 누적 → `hwp_doc_render_page` 로 `changedPages` 쪽 눈검증 →
   `hwp_doc_save --verify` → `hwp_close`.
5. 막히면 `rhwp://docs/agent_troubleshooting_guide.md` 리소스를 읽는다.

## 함정 (실측된 것만)

1. **상대 경로는 MCP 서버 프로세스의 cwd 기준**이다 — 에이전트 호스트의 작업 디렉터리와
   다른 것이 정상. MCP 로는 **절대 경로만** 넘긴다.
2. **`hwp_batch` 계열은 `structuredContent` 가 `null`** 이다(NDJSON 여러 줄이라 객체
   하나로 못 담는다). `content[0].text` 를 줄 단위로 파싱한다. 단건 도구는
   `content[0].text` 와 `structuredContent` 를 둘 다 준다.
3. **`batch convert` 는 MCP 에 의도적으로 미노출**(파일 쓰는 축, CLI 전용).
   `capabilities` 의 `batch.mcp.excluded` 가 이유를 문자열로 적어 준다.
4. **`password` 는 `writeOnly`** — 서버는 응답·오류·세션 상태에 값을 넣지 않고 자식
   프로세스의 `--password-stdin` 으로만 넘긴다. 다만 MCP 호스트의 대화 기록·telemetry 가
   도구 인자를 보관할 수 있으므로 신뢰된 로컬 호스트에서만 쓴다. stdin 을 경로 목록에
   쓰는 `hwp_batch`·`hwp_batch_search` 는 password 를 지원하지 않는다.
5. **`-` 로 시작하는 검색어**: CLI `search` 는 `--` 구분자가 필요하지만 MCP `hwp_search`
   는 배선에 이미 넣어 두었으므로 `query` 를 그대로 준다.
6. **세션 도구는 `capabilities --mcp` 선언에 없다**(서버 전용). 전체 51종(무상태 39 +
   세션 12, v0.8.2 실측)은 `mcp-serve` 의 `tools/list` 가 원천이다. 수치가 문서와 다르면
   손에 든 바이너리가 이긴다.

## 상세 레퍼런스

- 통합 두 경로(서버·매니페스트 직접 소비)와 오류 의미론: [`mydocs/manual/mcp_integration_guide.md`](../../../mydocs/manual/mcp_integration_guide.md)
- MCP 도구 전수 지도·세션 계약·판정 3층: [`mydocs/manual/agent_knowledge_map.md`](../../../mydocs/manual/agent_knowledge_map.md) §4·§6
- `capabilities --mcp`·`mcp-serve` 명령 상세: [`mydocs/manual/cli_commands.md`](../../../mydocs/manual/cli_commands.md) §2
- 증상별 실패 사전(§14 MCP): [`mydocs/manual/agent_troubleshooting_guide.md`](../../../mydocs/manual/agent_troubleshooting_guide.md)
