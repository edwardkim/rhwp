# 세션 도구 카드

각 카드의 기계 가독 정본은 `fixtures/tools/<name>.json` 이다.
아래는 에이전트가 훑는 요약이다. 스키마는 `tools/list` 가 이긴다.

## `hwp_open`

- 가족: lifecycle
- 필수: path
- 선택: password
- 봉투: docId, pageCount, source, schemaVersion
- 무상태 짝: `—`
- 언제: 같은 문서를 두 번 이상 조회·편집할 때 파싱 1회로 핸들을 연다.
- 언제 아닌가: 호출 하나면 끝인 단건 작업. 그때는 무상태 도구가 싸다.
- 복구: path 를 절대 경로로 고친 뒤 다시 hwp_open.
- IR 기록: False / 디스크: False / idempotent: False / destructive: False

## `hwp_doc_text`

- 가족: query
- 필수: docId
- 선택: page, maxChars, charOffset
- 봉투: pages, truncated, omittedCount, nextOffset
- 무상태 짝: `hwp_export_text`
- 언제: 연 핸들에서 쪽 본문을 이어 읽을 때. nextOffset 으로 창을 잇는다.
- 언제 아닌가: 전문을 한 번만 뽑으면 hwp_export_text.
- 복구: 핸들 만료면 hwp_open. 쪽 범위 밖이면 page 를 고친다.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_info`

- 가족: query
- 필수: docId
- 선택: (없음)
- 봉투: format, pageCount, paraCount, fonts, title, warnings
- 무상태 짝: `hwp_info`
- 언제: 편집 후 pageCount 변화를 추적하거나 규모를 재확인할 때.
- 언제 아닌가: 파일을 한 번만 보고 끝이면 hwp_info.
- 복구: nextCall.name=hwp_open 으로 docId 재발급.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_fields`

- 가족: query
- 필수: docId
- 선택: (없음)
- 봉투: fieldCount, fields, textSecurity
- 무상태 짝: `hwp_fields`
- 언제: hwp_doc_fill_fields 전에 이름·반복 순번을 조사할 때.
- 언제 아닌가: 서식 한 번 조사 후 프로세스 종료면 hwp_fields.
- 복구: nextCall 로 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_tables`

- 가족: query
- 필수: docId
- 선택: (없음)
- 봉투: tableCount, tables
- 무상태 짝: `hwp_export_tables`
- 언제: hwp_doc_set_cell 전에 표 번호·병합 범위를 확인할 때.
- 언제 아닌가: 표만 한 번 뽑아 CSV 로 넘기면 hwp_export_tables 또는 hwp_table_to_csv.
- 복구: nextCall 로 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_render_page`

- 가족: query
- 필수: docId, page, output
- 선택: (없음)
- 봉투: output, page, bytes
- 무상태 짝: `hwp_export_svg`
- 언제: changedPages 쪽만 SVG 로 눈검증할 때.
- 언제 아닌가: 문서 전체를 SVG 로 한 번에 렌더하면 hwp_export_svg.
- 복구: output 은 절대 경로. page 는 0 기준.
- IR 기록: False / 디스크: True / idempotent: True / destructive: False

## `hwp_doc_search`

- 가족: query
- 필수: docId, query
- 선택: caseSensitive, maxMatches, offset
- 봉투: matchCount, totalMatchCount, truncated, omittedCount, matches, nextOffset
- 무상태 짝: `hwp_search`
- 언제: 대형 문서에서 '어디를 고칠까'를 반복 탐색할 때.
- 언제 아닌가: 검색 1회면 hwp_search. 폴더 전수면 hwp_batch_search.
- 복구: 핸들 만료면 hwp_open. query 누락이면 인자를 고친다(재시도 금지).
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_structure`

- 가족: query
- 필수: docId
- 선택: mode
- 봉투: schemaVersion, source, mode, nodeCount, structure
- 무상태 짝: `hwp_export_structure`
- 언제: 법령·규정 조문 계층을 세션 안에서 반복 인용할 때.
- 언제 아닌가: 목차 한 번이면 hwp_export_structure. 안정 노드 ID 가 필요하면 hwp_doc_tree.
- 복구: nextCall 로 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_extract_data`

- 가족: query
- 필수: docId
- 선택: kind, limit
- 봉투: schemaVersion, source, kind, itemCount, totalItemCount, truncated, counts, items
- 무상태 짝: `hwp_extract_data`
- 언제: 연 핸들에서 날짜·금액·수량을 반복 좁혀 뽑을 때.
- 언제 아닌가: 단건 수확은 hwp_extract_data. 폴더 전수는 hwp_batch_extract_data.
- 복구: nextCall 로 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_replace_text`

- 가족: mutate
- 필수: docId, find, replace
- 선택: caseSensitive
- 봉투: replacedCount, changedPages
- 무상태 짝: `hwp_replace_text`
- 언제: 연 문서에서 문구를 누적 치환하고 나중에 한 번 저장할 때.
- 언제 아닌가: 치환 1회 후 파일만 필요하면 hwp_replace_text.
- 복구: replacedCount 0 은 오류가 아니다. 핸들 만료만 hwp_open.
- IR 기록: True / 디스크: False / idempotent: False / destructive: False

## `hwp_doc_set_cell`

- 가족: mutate
- 필수: docId, table, row, col, text
- 선택: keepStyle
- 봉투: overflow, changedPages
- 무상태 짝: `hwp_set_cell`
- 언제: 누름틀 없는 칸을 좌표로 채울 때. 먼저 hwp_doc_tables.
- 언제 아닌가: 칸 하나 고치고 끝이면 hwp_set_cell.
- 복구: 병합 덮인 칸은 앵커 좌표로 고친다. 재시도로 우회하지 않는다.
- IR 기록: True / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_fill_fields`

- 가족: mutate
- 필수: docId, data
- 선택: (없음)
- 봉투: filledCount, notFound, ambiguous, changedPages
- 무상태 짝: `hwp_fill_fields`
- 언제: 같은 서식에 값을 여러 번 누적 채울 때.
- 언제 아닌가: 채움 1회면 hwp_fill_fields. 서식1+데이터N은 hwp_batch_fill.
- 복구: notFound/ambiguous 는 isError:false 데이터다. 이름을 고친다.
- IR 기록: True / 디스크: False / idempotent: True / destructive: False

## `hwp_doc_save`

- 가족: persist
- 필수: docId, output
- 선택: verify
- 봉투: output, format, bytes, verify
- 무상태 짝: `—`
- 언제: 누적 편집을 형식 보존으로 기록할 때. 세션의 유일한 기록 지점.
- 언제 아닌가: 저장 없이 조회만 했으면 호출하지 않는다.
- 복구: output 은 절대 경로. 원본 덮어쓰기는 의도일 때만.
- IR 기록: False / 디스크: True / idempotent: True / destructive: True

## `hwp_close`

- 가족: lifecycle
- 필수: docId
- 선택: (없음)
- 봉투: closed, docId, schemaVersion
- 무상태 짝: `—`
- 언제: 핸들을 더 쓰지 않을 때 메모리를 해제한다.
- 언제 아닌가: 저장하지 않은 편집을 남긴 채 닫으면 인메모리 누적이 사라진다.
- 복구: 이미 닫혔으면 성공으로 보고 끝낸다. 다시 쓰려면 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_ws_list`

- 가족: workspace
- 필수: (없음)
- 선택: (없음)
- 봉투: entries, truncated
- 무상태 짝: `—`
- 언제: mcp-serve --workspace 로 기동한 코퍼스 인벤토리를 볼 때.
- 언제 아닌가: 워크스페이스 없이 기동했으면 이 도구는 실패한다. 경로로 hwp_open.
- 복구: 서버를 --workspace 로 다시 붙이거나 hwp_open 으로 전환.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_ws_open`

- 가족: workspace
- 필수: id
- 선택: password
- 봉투: docId, pageCount, source, schemaVersion
- 무상태 짝: `—`
- 언제: hwp_ws_list 의 w1.. id 로 핸들을 열 때.
- 언제 아닌가: 경로를 알면 hwp_open.
- 복구: id 는 hwp_ws_list 의 entries[].id 만.
- IR 기록: False / 디스크: False / idempotent: False / destructive: False

## `hwp_doc_tree`

- 가족: query
- 필수: docId
- 선택: (없음)
- 봉투: nodes
- 무상태 짝: `—`
- 언제: 페이지 p0..·표 t0.. 안정 ID 로 구조를 볼 때(#4357).
- 언제 아닌가: 제목·조문 의미 계층은 hwp_doc_structure.
- 복구: nextCall 로 hwp_open.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False

## `hwp_ws_journal`

- 가족: workspace
- 필수: (없음)
- 선택: (없음)
- 봉투: entries
- 무상태 짝: `—`
- 언제: 변이 도구 전/후 본문 SHA-256 을 자기검증할 때.
- 언제 아닌가: 조회만 한 세션에서는 저널이 비어 있는 것이 정상이다.
- 복구: 워크스페이스 기동이 아니면 저널 축을 쓰지 않는다.
- IR 기록: False / 디스크: False / idempotent: True / destructive: False
