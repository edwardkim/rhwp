# Task 238 1단계 완료 보고서: WASM 검색 엔진

## 완료 항목

### Rust 네이티브 API (search_query.rs)
- `search_text_native(query, from_sec, from_para, from_char, forward, case_sensitive)` — 문서 전체 텍스트 검색
  - 본문 문단, 표 셀, 글상자 내부 텍스트 포함
  - 정방향/역방향 + wrap-around 지원
  - 대소문자 구분 옵션
- `replace_text_native(sec, para, char_offset, length, new_text)` — 단일 치환 (delete + insert)
- `replace_all_native(query, new_text, case_sensitive)` — 전체 치환 (역순 처리)
  - 셀/글상자 내부도 치환
  - 변경된 섹션 일괄 recompose
- `get_page_of_position_native(section_idx, para_idx)` — 위치→쪽 번호

### WASM API (wasm_api.rs)
- `searchText(query, fromSec, fromPara, fromChar, forward, caseSensitive)`
- `replaceText(sec, para, charOffset, length, newText)`
- `replaceAll(query, newText, caseSensitive)`
- `getPageOfPosition(sectionIdx, paraIdx)`

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/search_query.rs` | 신규 (~230줄) |
| `src/document_core/queries/mod.rs` | search_query 모듈 등록 |
| `src/wasm_api.rs` | 4개 WASM API 추가 |

## 검증
- cargo build: 성공 (경고 없음)
- cargo test: 716개 통과
