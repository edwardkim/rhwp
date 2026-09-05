# #5293 실 에이전트 MCP 세션 규약 — 작업 기록

날짜: 2026-08-18
이슈: https://github.com/edwardkim/rhwp/issues/5293
브랜치: `feat/agent-mcp-session` (`upstream/devel` 기준 격리 worktree)
범위: `.agents/skills/rhwp-mcp-session/` · `scripts/tests/test_agent_mcp_session.py` · 본 문서
비범위: `gym/` · `rhwp-onboarding` · `rhwp-safe-edit` · `rhwp-provenance` · `rhwp-doc-triage` · 새 CLI · 새 MCP 도구

## 무엇을

에이전트가 rhwp 를 MCP 로 붙일 때 **세션 도구와 무상태 도구를 잘못 고르고**,
도구 이름을 문서에서 외워 발명하는 문제를 줄인다. 기존 `rhwp-mcp-session` 스킬은
한 파일에 요약만 있었고, 세션 수명·복구·단일 출처를 실행 가능한 픽스처로 고정하지
않았다. 지식 지도의 개수(51/12, 82, 181)가 버전에 따라 갈리므로 스킬이 숫자를
계약처럼 적으면 곧바로 낡는다.

## 왜

`mcp-serve` 의 세션 축(`hwp_open` → `hwp_doc_*` → `hwp_close`)은 대형 문서 반복
조회에서 재파싱을 없앤다. 반대로 단건 작업에 세션을 열면 오버헤드만 늘고,
세션에 없는 동사(`hwp_doc_redact` 따위)를 만들어 부르면 `알 수 없는 도구` 로
죽는다. 실 에이전트 호스트는 gym 트레이스가 아니라 이 판단이 필요하다.

이슈 DoD: additions 5000–10000, PR 전 `cargo fmt --all -- --check`, gym 금지.

## 어떻게

1. 격리 worktree `C:/Users/swsz9/rhwp-agent-mcp-session` 에
   `feat/agent-mcp-session` 을 `upstream/devel` 에서 분기.
2. 도구 이름은 `src/agent_profiles.rs` `ALL_SESSION_TOOLS` 와
   `src/main.rs` `mcp_tool_definitions()` 에서만 추출.
   생성기 `.agents/skills/rhwp-mcp-session/references/_gen_pack.py`.
3. 스킬 본문을 내비게이터로 재작성하고 `references/` 에 수명·무상태·SSOT·복구·짝·
   부착·판단 트리를 분리.
4. `references/fixtures/` 에 allowlist, 도구 카드 18, 트레이스 20, 오류 18,
   결정 30, 시나리오 카탈로그 150.
5. `scripts/tests/test_agent_mcp_session.py` 가 소스와 픽스처를 대조하고
   마크다운의 `hwp_*` 토큰이 allowlist 밖이면 실패.
6. capability 등록부 `CAP-5293` / `rhwp-mcp-session` 행 추가.

새 CLI 없음. `mcp_serve.rs` 를 바꾸지 않음. 없는 세션 동사를 만들지 않음.

## 소스에서 읽은 세션 표면 (devel)

`ALL_SESSION_TOOLS` 18종:

- 수명: `hwp_open` · `hwp_close`
- 조회: `hwp_doc_info` · `hwp_doc_text` · `hwp_doc_fields` · `hwp_doc_tables` ·
  `hwp_doc_search` · `hwp_doc_render_page` · `hwp_doc_structure` ·
  `hwp_doc_extract_data` · `hwp_doc_tree`
- 변이: `hwp_doc_fill_fields` · `hwp_doc_replace_text` · `hwp_doc_set_cell`
- 기록: `hwp_doc_save`
- 워크스페이스: `hwp_ws_list` · `hwp_ws_open` · `hwp_ws_journal`

지식 지도 §6-2 본문 목록이 `hwp_doc_structure` · `hwp_doc_extract_data` 를
빠뜨린 채 "18개"라고 적힌 곳이 있다. 스킬은 소스를 따르고, 개수는 계약이 아니라
`tools/list` 가 이긴다고 못 박았다. 지도 본문 수정은 이 PR 범위 밖(지도는
"기존 행 재서술 금지").

무상태 선언은 생성기가 `mcp_tool_definitions()` 에서 162종을 읽었다.
지도가 163을 말하면 빠진 1종은 선언 매크로가 `tool(` / `tool_with_optional_args(`
가 아닌 경로일 수 있다. 에이전트는 숫자를 외우지 말고 `--mcp` 를 찍는다.

## 세션 수명 계약 (스킬에 옮긴 것)

```
hwp_open|hwp_ws_open → OPEN(docId)
  조회 hwp_doc_*
  변이 fill/replace/set_cell   (IR 만)
  기록 hwp_doc_save            (핸들 유지)
hwp_close → CLOSED
CLOSED 재사용 → isError + nextCall(hwp_open)
```

- `path`/`output` 은 절대 경로. 상대 경로는 서버 cwd.
- `password` 는 writeOnly. 응답·세션에 안 남긴다.
- `changedPages` 가 있으면 그 쪽만 `hwp_doc_render_page`.
- `truncated` 가 아니라 `nextOffset` 이 이어보기 판정.
- 세션에 없는 동사(PDF, redact, run, insert_row, …)는 무상태/CLI.

## 무상태를 고르는 때

- 호출 1회: `hwp_info` · `hwp_search` · `hwp_fill_fields` · `hwp_export_pdf`
- 폴더: `hwp_scan` · `hwp_batch*`
- 세션에 짝이 없는 동사: 변환·검증 사다리·보안·원자 계획(`hwp_run_plan`)
- 두 파일 비교: `hwp_ir_diff` (세션 밖)

배치 `structuredContent=null`, `batch convert` MCP 미노출은 기존 함정 그대로.

## 단일 출처

| 묻고 싶은 것 | 어디서 |
|---|---|
| 무상태 도구 이름·스키마 | `rhwp capabilities --mcp` 또는 `rhwp://capabilities/mcp` |
| 세션 도구 포함 전체 | `mcp-serve` `tools/list` |
| CLI 조립 | 선언의 `cli.args` 자리표시자 |

호스트 설정에 도구 목록을 하드코딩하지 않는다.

## 오류 복구

1. JSON-RPC `error` → 프로토콜 수정. 재시도 금지.
2. `isError:true` → 닫힌 핸들만 `nextCall` 로 `hwp_open`. 필수 인자/exit 2 는 인자 수정.
3. `isError:false` → 봉투 필드 게이트 (`identical`/`notFound`/`invalid`/`nextOffset`).

실측 바늘은 실패 사전 §14 와 `references/fixtures/errors/`.

## 검증

```
python -m unittest scripts.tests.test_agent_mcp_session
cargo fmt --all -- --check
```

픽스처 가드:

- 모든 `hwp_*` 토큰 ⊂ 소스 allowlist (`hwp_doc_foo` 는 오타 복구 트레이스 하나뿐)
- 세션 트레이스는 open/ws 로 시작해 `hwp_close` 로 끝
- 오류 카드 3층 전부 존재
- 결정 카드의 `first_tool` 실존
- capability 등록부 `CAP-5293` 행

코드·CLI 변경이 없으므로 clippy/test/시각 게이트는 해당 없음.
포맷 게이트는 저장소 하드 게이트라 PR 전에 통과시킨다.

## 고의로 안 한 것

- `gym/` 아래 세션 트레이스 도구·팩. 이슈가 gym 금지.
- 온보딩 닥터, 안전 편집, 출처 표지, 트리아지 스킬. 병렬 이슈(#5292·#5294·#5295·#5296).
- `mcp_serve.rs` 에 도구 추가. "Do not invent tools."
- 지식 지도 §6 숫자 재서술. 지도 규약이 기존 행 재서술을 금한다.

## 파일 지도

| 경로 | 역할 |
|---|---|
| `.agents/skills/rhwp-mcp-session/SKILL.md` | 에이전트 진입·30초 판단 |
| `references/session_lifecycle.md` | 수명 상태 기계 |
| `references/stateless_when.md` | 무상태 선택 |
| `references/capabilities_ssot.md` | `--mcp` 단일 출처 |
| `references/error_recovery.md` | 판정 3층 |
| `references/fixtures/` | 기계 검증 JSON |
| `references/_gen_pack.py` | 소스 → 픽스처 재생성 |
| `scripts/tests/test_agent_mcp_session.py` | allowlist·수명·복구 가드 |
| `mydocs/manual/agent_capability_registry.md` | `CAP-5293` |

## 재생성

소스에 세션 도구가 늘면 (새 이름을 **구현이 먼저** 추가한 뒤에만):

```
python .agents/skills/rhwp-mcp-session/references/_gen_pack.py
python -m unittest scripts.tests.test_agent_mcp_session
```

생성기가 `SESSION_META` 누락을 에러로 막는다. 메타 없이 이름을 픽스처에 넣지 않는다.
