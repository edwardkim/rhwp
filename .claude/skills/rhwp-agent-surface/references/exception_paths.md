# 예외 경로 — 네 바늘과 그 옆

픽스처: [`../fixtures/exceptions/`](../fixtures/exceptions/).

## 1. capabilities 키 부재 (`missing_capabilities_key`)

**증상.** 표지를 읽는 파서가 키 부재에서 죽거나, `false` 로 단정해
문서 파생 값을 신뢰한다.

**실측.** 플레이북 §3-1 / §10-7. 6개 봉투에 키가 없다. 그중
`edit redact` · `edit sanitize` · `run --dry-run` 은 원문 값을 실제로 싣는다.

**소비자 처방.** 키 부재 = 미표기. 미표기는 보수적으로 문서 파생.
정본 지도: `rhwp export-provenance-map --json` 의 `commands.<명령>.untrusted[]`.

**구현자 처방.** 새 봉투는 모든 모드에서 두 키를 명시한다.
문서를 열지 않으면 `untrustedContent:false` + `untrustedFields:[]`.

## 2. 드리프트 가드 실패 (`drift_guard_fail`)

**증상.** CI 에서 `capabilities_mcp_covers_every_json_command` 또는
`tools_list_matches_capabilities_manifest` 가 붉다.

**뜻.** 선언과 실행이 갈라졌다. `--json` 명령을 capabilities 에만 넣었거나,
`tools/list` 가 `mcp_tool_definitions()` 가 아닌 다른 배열을 쓴다.

**처방.**

1. 빠진 이름을 가드 메시지에서 읽는다.
2. `mcp_tool_definitions()` 에 `tool()` 한 줄 (또는 `ALL_SESSION_TOOLS`).
3. 제외 목록(`capabilities` 자신, `dump-pages`)을 감으로 늘리지 않는다.
   제외는 이슈 번호와 사유가 있는 것만.

상세: [`drift_guards.md`](drift_guards.md).

## 3. 닫힌 핸들 (`closed_handle`)

**증상.** `hwp_doc_*` 가 `isError:true`.

```json
{"error":"열려 있지 않은 핸들: doc-1 (hwp_open 먼저)",
 "nextCall":{"name":"hwp_open",
             "arguments":{"path":"<열 문서 경로>"},
             "why":"핸들이 없거나 만료 — hwp_open 으로 docId 를 재발급한 뒤 재시도"}}
```

**처방.** `nextCall` 대로 `hwp_open` → 새 `docId`. 옛 id 로 재시도 금지.
서버를 재시작했으면 모든 핸들이 죽었다.

호스트 부착·수명 다이어그램은 `rhwp-mcp-session` 이 맡는다.
여기는 "닫힌 핸들은 런타임 오류이고 nextCall 이 다음 층"만 고정한다.

## 4. 프로필 차단 (`profile_blocked`)

**증상 A.** `오류: 알 수 없는 프로필 '<name>'` + 사용 가능 목록 + exit 2.

**증상 B.** `tools/list` 에 없는 이름을 `tools/call` 한다.
프로필은 목록과 호출이 **같은 함수**로 막는다. 우회 금지.

**예.** `경영보고` 는 `hwp_fill_fields` 를 열지 않는다.
`아카이브검색` 은 세션 조회만 (`SESSION_READ_TOOLS`) — `hwp_doc_save` 차단.
`개발통합` 은 필터 없음.

**처방.** 프로필을 바꾸거나 필터 없는 프로필을 쓴다. 도구 이름을 발명하지 않는다.

## 옆 바늘 (자주 같은 자리에서)

| id | stderr / 신호 | exit |
|---|---|---|
| `search_combined_with_mcp` | `--search` 는 `--mcp/--profile` 과 함께 쓸 수 없습니다 | 2 |
| `json_without_search` | `--json` 은 `--search` 와 함께 | 2 |
| `identical_false_is_data` | `identical:false` | 3 / isError false |
| `replaced_zero_is_data` | `replacedCount:0`, 파일 없음 | 0 |
| `not_found_is_data` | `notFound` 가 찬 채 exit 0 | 0 |
