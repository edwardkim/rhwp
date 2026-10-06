---
name: rhwp-agent-surface
description: rhwp 에이전트 표면(CLI JSON · MCP 무상태 · MCP 세션)을 capabilities 단일 출처로 더하고 굴리는 계약 스킬입니다. 3층과 각 층의 SSOT, 규칙 1(선언·실행·문서 한 곳 — mcp_tool_definitions·드리프트 가드), 규칙 2(편집 로직 발명 금지 — 검증된 코어+봉투 helper), 규칙 3(판정은 데이터 — identical:false/replaced 0/notFound 는 필드, isError 는 런타임만), rhwp capabilities · --mcp · --search 사용법, agent_surface_playbook 수용 기준으로 조각을 더하는 절차, 예외(키 부재·가드 실패·닫힌 핸들·프로필 차단)를 다룹니다. 트리거 — "표면 추가/새 MCP 도구/새 --json 명령", "3층 계약", "capabilities 가 SSOT", "드리프트 가드", "판정은 데이터", "조각을 더하는 수용 기준". gym 벤치가 아니라 실 에이전트 표면 계약. 호스트 부착은 rhwp-mcp-session, 명령 매핑은 rhwp-cli, 대전 항해는 rhwp-codex — 이 스킬은 그 셋을 다시 쓰지 않는다.
---

# rhwp-agent-surface — 표면 플레이북 Skill

## 목적

실 에이전트가 **표면을 더하거나**, 이미 있는 표면을 **계약대로 굴릴** 때 쓴다.
정본은 [`mydocs/manual/agent_surface_playbook.md`](../../../mydocs/manual/agent_surface_playbook.md).
도구 정의의 단일 출처는 `rhwp capabilities` / `rhwp capabilities --mcp` /
`src/main.rs` `mcp_tool_definitions()` 이다. 숫자를 외우지 말고 바이너리를 찍는다.

이 스킬이 **아닌** 것:

| 스킬 | 그 스킬이 맡는 축 | 여기로 끌어오지 말 것 |
|---|---|---|
| `rhwp-mcp-session` | 호스트 부착(`.mcp.json`)·세션/무상태 **선택** | 등록 절차, 호스트별 킷 |
| `rhwp-cli` | 사용자 요청 → CLI **명령 매핑**·디버깅 순서 | SVG/PNG 내보내기 표 |
| `rhwp-codex` | 대전 교본 **항해**·재생성 | `agent_codex/` 장 순서 |

gym 팩·트레이스를 실행 경로로 쓰지 않는다.

상세는 [`references/`](references/README.md). 레시피는 [`examples/`](examples/).
기계 픽스처는 [`fixtures/`](fixtures/).

## 0. 30초 판단

| 질문 | 예 | 다음 |
|---|---|---|
| 어느 층에 더하는가? | 새 `--json` / 새 무상태 / 새 세션 | [3층](#1-표면의-3층) → [`three_layers.md`](references/three_layers.md) |
| 선언을 어디에 쓰는가? | 도구 목록·스키마 | **`mcp_tool_definitions()` 한 곳**. 복제 금지. 규칙 1 |
| 편집 로직을 새로 짜고 싶은가? | "서버만의 set_cell" | **금지.** 코어+봉투 helper. 규칙 2 |
| `identical:false` 가 나왔는가? | ir-diff exit 3 | **오류가 아니다.** 봉투 필드. 규칙 3 |
| 이름이 기억나지 않는가? | "redact 가 어디" | `rhwp capabilities --search redact` |
| 무상태 목록이 필요한가? | MCP 등록·바인딩 | `rhwp capabilities --mcp` |
| 세션 목록이 필요한가? | `hwp_doc_*` | `tools/list` — `--mcp` 선언에 없다 |
| 가드가 붉었는가? | drift / 키 부재 / 닫힌 핸들 / 프로필 | [예외](#5-예외-경로) |

## 1. 표면의 3층

| 층 | 무엇 | 단일 출처 |
|---|---|---|
| CLI `--json` | stdout 순수 JSON 봉투 + 종료 코드 0/1/2/3/4 | 각 명령 구현 + 봉투 helper (`*_json_value`) |
| MCP 무상태 | 선언(`capabilities --mcp`)과 실행(`mcp-serve`)이 공유하는 도구 | `mcp_tool_definitions()` (`src/main.rs`) |
| MCP 세션 | 열린 핸들(`docId`) 위의 재파싱 없는 연산 | `mcp_serve.rs` `served_tools()` + `ALL_SESSION_TOOLS` |

세션 도구는 `--mcp` 선언에 **없다**. 세션을 쓰려면 서버에 묻는다.
개수는 계약이 아니다 — 손에 든 바이너리가 이긴다.

카드: [`fixtures/layers.json`](fixtures/layers.json).

## 2. 규칙 세 줄

### 규칙 1 — 선언·실행·문서는 한 곳에서 갈라진다

무상태 도구는 `mcp_tool_definitions()` 에만 추가하면 선언과 서버가 함께 얻는다.
도구 목록을 호스트 설정·스킬·위키에 복제하지 않는다.

어긋남은 드리프트 가드가 잡는다.

- `tests/cli_json_contract.rs::capabilities_mcp_covers_every_json_command`
- `tests/mcp_server_contract.rs::tools_list_matches_capabilities_manifest`
- `tests/cli_json_contract.rs::capabilities_mcp_tool_definitions_contract`

가드가 붉으면 목록을 고치는 것이 아니라 **한 곳의 원천**을 고친다.
상세: [`rule1_single_source.md`](references/rule1_single_source.md),
[`drift_guards.md`](references/drift_guards.md).

### 규칙 2 — 새 편집·조회 로직을 만들지 않는다

MCP/세션 도구는 검증된 코어와 기존 봉투 helper 를 재사용한다.

`set_field_value_by_name_at` · `replace_all_native` · `grep` ·
`collect_field_records` · `extract_tables` · `edit_serialize` · `*_json_value`.

서버 전용 경로를 새로 만들면 CLI 와 계약이 갈라진다.
상세: [`rule2_reuse_core.md`](references/rule2_reuse_core.md),
[`fixtures/reuse/core_map.json`](fixtures/reuse/core_map.json).

### 규칙 3 — 판정은 데이터다

| 신호 | 뜻 | isError? |
|---|---|---|
| `identical:false` | 차이가 있다 (CLI exit 3) | **아니오** (MCP) |
| `replacedCount:0` | 치환 0건, **출력 파일 없음** | 아니오 |
| `notFound` / `ambiguous` | 서식이 덜 찼다. exit 0 | 아니오 |
| `matchCount:0` | 검색 0건 | 아니오 |
| `invalid[]` | 계획/CSV 선검증 위반 | CLI exit 2, MCP 는 도구마다 |
| 없는 파일·닫힌 핸들 | 런타임 실패 | **예** |

`isError:true` 는 실행 실패에만 쓴다. CLI 는 exit 3/4 로 판정을 신호하되
**봉투를 먼저 낸다.**
상세: [`rule3_judgment_is_data.md`](references/rule3_judgment_is_data.md),
[`fixtures/envelopes/`](fixtures/envelopes/).

## 3. 쓰는 법 — `capabilities` · `--mcp` · `--search`

```bash
# 1) CLI 자기서술. 언제나 JSON. --json 을 붙이지 않는다.
rhwp capabilities

# 2) 무상태 MCP 선언 (세션 없음). 바인딩·등록의 원천.
rhwp capabilities --mcp
rhwp capabilities --mcp --profile 행정서식

# 3) 이름을 모를 때. AND, 대소문자 무시, 하위명령 name+summary 포함.
rhwp capabilities --search redact
rhwp capabilities --search "표 병합" --json
```

함정:

- `rhwp capabilities --json` 만 → exit 2.
  `--json` 은 `--search` 와만 같이 쓴다.
- `--search` 와 `--mcp`/`--profile` 동시 → exit 2.
- `rhwp --help` 를 파싱해 명령 목록을 만들지 마라. 계약이 아니다.
- 세션 이름은 `tools/list` 가 정본. `--mcp` 출력에 `hwp_open` 이 없다고 없는 도구가 아니다.

사용법 카드: [`capabilities_how_to.md`](references/capabilities_how_to.md).
검색 픽스처: [`fixtures/search/`](fixtures/search/).
트랜스크립트: [`fixtures/transcripts/`](fixtures/transcripts/).

## 4. 조각을 더할 때 — 플레이북 수용 기준

순서 고정 ([`add_surface_piece.md`](references/add_surface_piece.md)):

0. 잠금 (이슈 assignee 또는 착수 코멘트). 선점된 이슈는 건드리지 않는다.
1. 이슈에 공백을 실측으로 적는다.
2. **red** 계약 테스트 `tests/*_contract.rs` **신설** (기존 파일 수정보다 신설).
3. 구현 — 규칙 1~3. 실패 경로 stdout 순수성.
4. 신규 green + 인접 계약 + clippy + rustfmt.
5. 누적 머지 충돌검사.
6. 처리 문서 + 증적 2종.
7. 한글 PR, `closes #<이슈>`.

층별 한 줄:

- **CLI JSON** — 명령 + `*_json_value` + `capabilities_command_entries()` 등재.
- **무상태** — 대응하는 `--json` CLI 가 먼저. 그다음 `tool()` 한 줄.
  `inputSchema.required` ↔ `cli.args` 자리표시자 1:1.
- **세션** — 무상태 짝이 먼저. `ALL_SESSION_TOOLS` + `served_tools()` 같은 코어.
  닫힌 핸들 `isError` + `nextCall.hwp_open`. 디스크 기록은 `hwp_doc_save` 만.

수용 체크리스트 ([`acceptance_checklist.md`](references/acceptance_checklist.md),
[`fixtures/add_surface/acceptance.json`](fixtures/add_surface/acceptance.json)):

- [ ] `--json` stdout 에 JSON 하나(배치는 NDJSON). 진단은 stderr.
- [ ] 런타임 실패 시 stdout 비움, exit 1. 조립 오류 exit 2. 미지 옵션 침묵 금지.
- [ ] `schemaVersion` 포함.
- [ ] `untrustedContent`·`untrustedFields` 를 **모든 모드에서** (dry-run 포함).
- [ ] 무상태: required 와 자리표시자 1:1. 선택 인자는 `optionalArgs`.
- [ ] 세션: 닫힌 핸들 `isError`, 기록은 `hwp_doc_save` 만, 판정 어휘 동형.
- [ ] 실패 응답에 `nextCall{name,arguments,why}`.
- [ ] `cli_commands.md` + 지식 지도 행.

새 CLI 를 이 스킬 PR 에서 만들지 않는다. DocumentCore 편집 로직을 발명하지 않는다.

## 5. 예외 경로

네 가지는 이 스킬의 필수 바늘이다. 카드: [`exception_paths.md`](references/exception_paths.md),
[`fixtures/exceptions/`](fixtures/exceptions/).

### 5-1. capabilities 키 부재

`untrustedContent` 가 **없는** 봉투가 실측으로 남아 있다
(`edit redact` / `sanitize` / `run --dry-run` / `insert-image` /
`export-ir-schema` / `export-capabilities-schema`).

키 부재를 `false` 로 읽지 마라. **"미표기"** 다. 미표기는 보수적으로 문서 파생.
지도: `rhwp export-provenance-map --json`.

### 5-2. 드리프트 가드 실패

`--json` 명령을 capabilities 에만 넣고 MCP 에서 빠뜨리면
`capabilities_mcp_covers_every_json_command` 가 붉다.
고치는 곳: `mcp_tool_definitions()` 한 줄. 가드 제외 목록을 감으로 늘리지 마라.

### 5-3. 닫힌 핸들

```json
{"error":"열려 있지 않은 핸들: doc-1 (hwp_open 먼저)",
 "nextCall":{"name":"hwp_open","arguments":{"path":"<열 문서 경로>"},
             "why":"핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도"}}
```

`isError:true`. `nextCall.name` 을 그대로 부른다. 같은 `docId` 로 재시도하지 않는다.

### 5-4. 프로필 차단

`--profile` 은 추천이 아니라 **서버가 제공하는 집합의 경계**다.
목록에 없는 도구는 `tools/call` 로도 우회할 수 없다
(`allows_tool` / `allows_session_tool` 이 `tools/list` 와 같은 함수).
없는 프로필 이름: `오류: 알 수 없는 프로필` + exit 2.
처방: 프로필을 바꾸거나 `개발통합`(필터 없음). 이름을 발명하지 않는다.

## 6. 하지 않는 것

- 새 CLI 명령 / 새 MCP 도구 이름 발명.
- DocumentCore 편집 로직 신설.
- `rhwp-mcp-session` / `rhwp-cli` / `rhwp-codex` 본문 재작성.
- gym 경로를 이 스킬의 실행 경로로 쓰기.
- 호스트 설정에 도구 목록 하드코딩.
- 개수(39/51/162/18)를 계약처럼 적기.

## 픽스처·검증

- `fixtures/allowlist.json` — 소스에서 추출한 세션·무상태·CLI 이름.
- `fixtures/layers.json` · `fixtures/rules.json` — 3층·3규칙.
- `fixtures/exceptions/` · `fixtures/envelopes/` · `fixtures/drift/` — 바늘.
- `fixtures/search/` · `fixtures/transcripts/` — capabilities 사용.
- `fixtures/add_surface/` — 수용 기준.
- 생성: `python .claude/skills/rhwp-agent-surface/references/_gen_pack.py`
- 가드: `python -m unittest scripts.tests.test_agent_surface`

작업 기록: [`mydocs/working/agent_surface_skill.md`](../../../mydocs/working/archives/agent_surface_skill.md).
