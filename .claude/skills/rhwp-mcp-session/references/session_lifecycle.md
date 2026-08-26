# 세션 수명 — hwp_open → hwp_doc_* → hwp_close

권위: `src/mcp_serve.rs` 세션 핸들, `src/agent_profiles.rs` ALL_SESSION_TOOLS.
도구 목록의 정본은 `mcp-serve` 의 `tools/list` 다. `capabilities --mcp` 는 무상태 선언만 낸다.

## 한 줄 계약

1. `hwp_open`(또는 `hwp_ws_open`)이 `docId` 를 발급한다. 파싱은 이 순간 한 번이다.
2. `hwp_doc_*` 는 그 핸들의 IR 을 재파싱 없이 읽거나 누적 편집한다.
3. 디스크에 쓰는 세션 도구는 `hwp_doc_save` 와 `hwp_doc_render_page`(새 SVG) 뿐이다.
4. `hwp_close` 가 메모리를 해제한다. 저장하지 않은 편집은 사라진다.
5. 핸들 수명 = 서버 프로세스 수명. 영속이 아니다.

## 상태 기계

```
(없음)
  --hwp_open/hwp_ws_open--> OPEN(docId)
                               | 조회: hwp_doc_info/text/fields/tables/search/structure/extract_data/tree/render_page
                               | 변이: hwp_doc_fill_fields/replace_text/set_cell   (IR 만, 디스크 아님)
                               | 기록: hwp_doc_save   (핸들은 그대로 OPEN)
                               v
                            CLOSED  --이미 닫힘--> isError + nextCall(hwp_open)
```

서버가 내려가면 모든 핸들이 무효다. 호스트가 MCP 서버를 재시작하면 `doc-1` 을 재사용하지 말고
다시 `hwp_open` 한다.

## 정상 흐름 (실측 어휘)

```jsonc
→ tools/call hwp_open        { "path": "C:/절대/경로/편람.hwp" }
← { "docId": "doc-1", "pageCount": 393, "source": "…", "schemaVersion": "1.0" }

→ tools/call hwp_doc_search  { "docId": "doc-1", "query": "위임전결" }
← matches[].page / section / paragraph   // hwp_search 와 동형

→ tools/call hwp_doc_fill_fields { "docId": "doc-1", "data": { "회사명": "페타플로" } }
← filledCount / notFound / ambiguous / changedPages

→ tools/call hwp_doc_render_page { "docId": "doc-1", "page": 0, "output": "C:/abs/out/p0.svg" }
→ tools/call hwp_doc_save    { "docId": "doc-1", "output": "C:/abs/out/저장본.hwp", "verify": true }
→ tools/call hwp_close       { "docId": "doc-1" }
← { "closed": true, "docId": "doc-1" }
```

## 세션 도구 18종 (소스 상수, 개수는 계약이 아님)

| `hwp_open` | lifecycle | `path` | `—` | 같은 문서를 두 번 이상 조회·편집할 때 파싱 1회로 핸들을 연다. |
| `hwp_doc_text` | query | `docId` | `hwp_export_text` | 연 핸들에서 쪽 본문을 이어 읽을 때. nextOffset 으로 창을 잇는다. |
| `hwp_doc_info` | query | `docId` | `hwp_info` | 편집 후 pageCount 변화를 추적하거나 규모를 재확인할 때. |
| `hwp_doc_fields` | query | `docId` | `hwp_fields` | hwp_doc_fill_fields 전에 이름·반복 순번을 조사할 때. |
| `hwp_doc_tables` | query | `docId` | `hwp_export_tables` | hwp_doc_set_cell 전에 표 번호·병합 범위를 확인할 때. |
| `hwp_doc_render_page` | query | `docId,page,output` | `hwp_export_svg` | changedPages 쪽만 SVG 로 눈검증할 때. |
| `hwp_doc_search` | query | `docId,query` | `hwp_search` | 대형 문서에서 '어디를 고칠까'를 반복 탐색할 때. |
| `hwp_doc_structure` | query | `docId` | `hwp_export_structure` | 법령·규정 조문 계층을 세션 안에서 반복 인용할 때. |
| `hwp_doc_extract_data` | query | `docId` | `hwp_extract_data` | 연 핸들에서 날짜·금액·수량을 반복 좁혀 뽑을 때. |
| `hwp_doc_replace_text` | mutate | `docId,find,replace` | `hwp_replace_text` | 연 문서에서 문구를 누적 치환하고 나중에 한 번 저장할 때. |
| `hwp_doc_set_cell` | mutate | `docId,table,row,col,text` | `hwp_set_cell` | 누름틀 없는 칸을 좌표로 채울 때. 먼저 hwp_doc_tables. |
| `hwp_doc_fill_fields` | mutate | `docId,data` | `hwp_fill_fields` | 같은 서식에 값을 여러 번 누적 채울 때. |
| `hwp_doc_save` | persist | `docId,output` | `—` | 누적 편집을 형식 보존으로 기록할 때. 세션의 유일한 기록 지점. |
| `hwp_close` | lifecycle | `docId` | `—` | 핸들을 더 쓰지 않을 때 메모리를 해제한다. |
| `hwp_ws_list` | workspace | `(없음)` | `—` | mcp-serve --workspace 로 기동한 코퍼스 인벤토리를 볼 때. |
| `hwp_ws_open` | workspace | `id` | `—` | hwp_ws_list 의 w1.. id 로 핸들을 열 때. |
| `hwp_doc_tree` | query | `docId` | `—` | 페이지 p0..·표 t0.. 안정 ID 로 구조를 볼 때(#4357). |
| `hwp_ws_journal` | workspace | `(없음)` | `—` | 변이 도구 전/후 본문 SHA-256 을 자기검증할 때. |

조회 축(`SESSION_READ_TOOLS`, 14종): `hwp_open`, `hwp_doc_text`, `hwp_doc_info`, `hwp_doc_fields`, `hwp_doc_tables`, `hwp_doc_search`, `hwp_doc_structure`, `hwp_doc_extract_data`, `hwp_doc_render_page`, `hwp_close`, `hwp_ws_list`, `hwp_ws_open`, `hwp_doc_tree`, `hwp_ws_journal`

변이 축: `hwp_doc_fill_fields` · `hwp_doc_replace_text` · `hwp_doc_set_cell`
기록 축: `hwp_doc_save` (`destructiveHint=true` — output 이 원본 경로일 수 있다)

## 하지 않는 것

- `handle` 이라는 인자 이름은 없다. 항상 `docId`.
- 상대 경로. 서버 cwd 와 호스트 cwd 가 다르다.
- 저장 없이 "파일이 바뀌었다"고 보고하기.
- 닫힌 `docId` 를 같은 값으로 재시도하기 — `nextCall` 이 새 `hwp_open` 을 가리킨다.
- 세션에 없는 편집(예: `hwp_doc_redact`, `hwp_doc_insert_row`)을 만들어 부르기.
  그 작업은 무상태 도구이거나 CLI 다.

## 워크스페이스(#4357) 분기

`rhwp mcp-serve --workspace <dir>` 로 기동했을 때만 `hwp_ws_list` / `hwp_ws_open` /
`hwp_doc_tree` / `hwp_ws_journal` 이 의미가 있다. 아니면 경로로 `hwp_open` 한다.
