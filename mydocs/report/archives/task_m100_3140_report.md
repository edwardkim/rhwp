---
kind: report
status: active
canonical: mydocs/report/archives/task_m100_3140_report.md
last_verified: 2026-07-30
---

# #3140 처리 기록 — `mcp-serve`: rhwp 를 실제 MCP 서버로

## 배경

#3140 은 rhwp 를 MCP 서버로 노출하자는 제안이다. 0단계(도구 스키마)는
`capabilities --mcp`(#3263)로 끝났지만, 그것은 **선언**일 뿐이었다 — 실행하려면
외부 호스트가 매니페스트를 해석해 CLI 를 직접 fork 해야 했고, #3140 이 짚은
세 공백 중 "상태 유지 세션"은 CLI 로는 원리적으로 못 메운다(프로세스마다 재파싱).

## 구현 — `rhwp mcp-serve`

`src/mcp_serve.rs` (신규, 바이너리 전용 모듈. WASM 대상 미포함).

### 프로토콜
MCP 표준 stdio 전송: 줄 단위 JSON-RPC 2.0.
`initialize` / `notifications/initialized` / `ping` / `tools/list` / `tools/call` 지원.
알 수 없는 메서드는 `-32601`, 파싱 불가 입력은 `-32700`, 요청 구조 오류는 `-32602`.
도구 실행 실패(없는 파일 등)는 MCP 규약대로 프로토콜 오류가 아니라
`isError:true` **도구 결과**로 돌아간다.

### 설계 결정 1 — 단일 출처
도구 정의를 `mcp_tool_definitions()` 로 추출해 `capabilities --mcp`(선언 출력)와
`mcp-serve`(실행 서버)가 **같은 목록**을 쓴다. 여기에만 추가하면 양쪽이 함께
갱신되고, 계약 테스트(`tools_list_matches_capabilities_manifest`)가 이를 고정한다.

### 설계 결정 2 — 무상태 도구는 검증된 CLI 배선을 그대로 실행
13종(`hwp_info` … `hwp_set_cell`)은 선언의 `cli.args` 자리표시자를 arguments 로
치환해 **자기 자신을 서브프로세스로** 실행한다. 서버 전용 실행 경로를 새로 만들면
CLI 와 어긋날 수 있지만, 이 방식은 #2707 종료 코드·stdout 순수성·`--json` 봉투
계약을 문자 그대로 재사용한다. stdout 이 JSON 이면 `structuredContent` 로도 준다.
`hwp_batch`/`hwp_batch_search` 의 `paths` 배열은 stdin 으로 흘려 넣는다
(선언의 `stdinTools` 와 정합).

### 설계 결정 3 — 세션 도구로 "상태 유지" 공백을 채움 (서버 전용 3종)
- `hwp_open {path}` → 문서를 한 번 파싱해 `docId` 핸들 발급 (`pageCount` 동봉)
- `hwp_doc_text {docId, page?}` → **재파싱 없이** 핸들에서 페이지 텍스트 조회
- `hwp_close {docId}` → 해제. 닫힌 핸들 재사용은 `isError:true`

### 설계 결정 4 — 의존성 추가 없음
프로토콜 표면이 좁아(메서드 5종) `rmcp`+tokio 없이 serde_json 만으로 구현했다.
비동기 런타임·신규 크레이트가 들어오지 않아 빌드·감사 표면이 그대로다.
세션 수요가 편집 왕복으로 확장될 때 `rmcp` 전환을 재평가하면 된다.

## 검증

- 신규 계약 테스트 `tests/mcp_server_contract.rs` 6건 all green:
  - `initialize_handshake_and_ping`
  - `tools_list_matches_capabilities_manifest` (선언·서버 단일 출처 드리프트 가드)
  - `tools_call_stateless_info_works` (hwp_info 실호출)
  - `session_open_read_close_without_reparse` (open→2회 조회→close→닫힌 핸들 isError)
  - `unknown_method_returns_jsonrpc_error` (-32601)
  - `unknown_tool_returns_is_error`
- 기존 `cli_json_contract` 22건 무회귀 (capabilities/--mcp/드리프트 가드 포함)
- `cargo clippy --release --bin rhwp -- -D warnings`: 0 warnings
- `cargo fmt --check`: 변경 파일 기준 clean

## 문서

- `mydocs/manual/cli_commands.md` 에 `### mcp-serve` 절 추가 (호스트 등록 예 포함)
- `--help`·`capabilities` 명령 목록에 `mcp-serve` 등재 (드리프트 가드 ② 정합)
- `capabilities --mcp` 의 `invocation` 에 `server: "mcp-serve"` 힌트 추가
- 서버 자기서술의 낡은 문구 수정: "읽는 도구 모음 (읽기 전용)" → "읽고 편집하는
  도구 모음" (hwp_fill_fields·hwp_replace_text·hwp_set_cell 이 이미 편집 도구)

## 남은 일 (이 PR 범위 밖)

- 세션 도구 확장: 핸들 기반 검색/표 추출, 편집 왕복(`hwp_doc_edit` → `save_as`)
- 비밀번호 보호 문서의 세션 열기(`--password` 대응)
- `resources`/`prompts` 등 MCP 확장 표면
