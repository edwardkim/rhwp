# 3층과 각 층의 단일 출처

에이전트 표면은 세 층이다. 층을 섞어 더하면 선언과 실행이 갈라진다.
픽스처: [`../fixtures/layers.json`](../fixtures/layers.json).

## CLI `--json`

**무엇.** 명령 하나가 stdout 에 JSON 봉투 하나(배치는 NDJSON)를 낸다.
종료 코드는 `rhwp capabilities` 의 `exitCodes` 가 정본이다.

| 코드 | 뜻 | 다음 수 |
|---|---|---|
| 0 | 실행 성공 | **봉투 판정 필드를 마저 읽는다** |
| 1 | 런타임 실패 | 입력·환경을 고친다 |
| 2 | 호출 조립 버그 | 같은 인자로 재시도 금지 |
| 3 | 검증 단언 실패(판정) | `identical`/`regression` — 오류가 아님 |
| 4 | 쪽 수 불일치 | `verifyPages` |

**단일 출처.** 각 명령 구현 + 봉투 helper (`*_json_value`).
목록의 출처는 `capabilities_command_entries()`.
첫 호출은 `rhwp capabilities` — `--json` 을 붙이지 않는다. 언제나 JSON.

**계약이 아닌 것.** `rhwp --help` 사람용 서식. help 를 파싱해 명령 집합을
만들지 않는다. help 와 capabilities 의 어긋남은
`capabilities_covers_every_help_command` 가 잡는다.

## MCP 무상태

**무엇.** 호출 하나 = 작업 하나. CLI `--json` 의 얇은 껍데기.
자리표시자 `{path}` `{query}` 를 `inputSchema` 같은 이름 값으로 치환한다.

**단일 출처.** `src/main.rs` `mcp_tool_definitions()`.
이 함수 하나가 `rhwp capabilities --mcp` 와 `mcp-serve` `tools/list` 의
무상태 부분을 같이 낸다.

**세션 도구는 여기 없다.** `--mcp` 출력에 `hwp_open` 이 없다고 세션이
없는 것이 아니다. 세션은 다음 층.

**자리표시자 규칙.** `inputSchema.required` 에 없는 값을 `cli.args` 자리표시자로
쓰지 않는다. 미치환 문자열이 CLI 로 새면 `{output}` 파일이 만들어진다.

## MCP 세션

**무엇.** `hwp_open` → `docId` → `hwp_doc_*` → `hwp_close`.
같은 문서를 재파싱 없이 다룬다. 디스크 기록은 `hwp_doc_save` 만.

**단일 출처.** `src/agent_profiles.rs` `ALL_SESSION_TOOLS` +
`src/mcp_serve.rs` `served_tools()` 디스패치.
자기서술은 `mcp-serve` 의 `tools/list`.

**핸들.** 인자 이름은 `docId` (`handle` 이 아니다).
수명 = 서버 프로세스 수명. 닫힌/모르는 id 는 `isError` + `nextCall.hwp_open`.

이 층의 **호스트 부착** (`.mcp.json`, 핸드셰이크 줄)은 `rhwp-mcp-session` 스킬이다.
여기는 "세션 조각의 계약이 어느 소스에서 갈라지는가"만 다룬다.

## 층 선택 (표면을 더할 때)

```
새 조회/편집을 에이전트가 부르게 하려면
  ├─ CLI --json 이 이미 있는가?
  │    └─ 없으면 CLI JSON 층부터 (규칙 2: 코어 재사용)
  ├─ 무상태 MCP 가 필요한가?
  │    └─ mcp_tool_definitions() 한 줄. 서버 전용 로직 금지
  └─ 같은 문서 반복인가?
       └─ 무상태 짝이 난 뒤에 ALL_SESSION_TOOLS + served_tools
```

세션만 먼저 더하지 마라. 세션은 무상태 짝의 인메모리 면이다.
짝이 없는 동사(PDF · redact · run · convert)를 `hwp_doc_*` 로 만들지 마라.
