# Task 238 최종 결과 보고서: 검색 기능 구현

## 개요

편집 메뉴의 검색 관련 4개 기능을 구현하였다.

| 기능 | 단축키 | 설명 |
|------|--------|------|
| 찾기(F) | Ctrl+F | 모달리스 검색 대화상자, 다음/이전 탐색 |
| 찾아바꾸기(E) | Ctrl+F2 | 찾기 + 바꾸기/모두 바꾸기 |
| 다시찾기(X) | Ctrl+L | 마지막 검색어로 다음 결과 이동 (대화상자 없이) |
| 찾아가기(G) | Alt+G | 쪽 번호 입력 → 해당 쪽으로 이동 |

## 완료 항목

### WASM 검색 엔진 (Rust)
- `searchText(query, fromSec, fromPara, fromChar, forward, caseSensitive)` — 문서 전체 텍스트 검색
  - 본문 문단, 표 셀, 글상자 내부 포함
  - 정방향/역방향 + wrap-around
  - 대소문자 구분 옵션
- `replaceText(sec, para, charOffset, length, newText)` — 단일 치환
- `replaceAll(query, newText, caseSensitive)` — 전체 치환 (역순 처리, 영향 섹션 일괄 recompose)
- `getPageOfPosition(sectionIdx, paraIdx)` — 위치→쪽 번호
- `getPositionOfPage(globalPage)` — 쪽 번호→위치 (찾아가기용)

### 프론트엔드
- **FindDialog**: 모달리스 대화상자 (편집 영역 조작 가능)
  - 찾기/바꾸기 모드 전환
  - 검색 결과 선택 영역 하이라이트
  - 키보드: Enter=다음, Shift+Enter=이전, Escape=닫기
  - 드래그 이동, 싱글톤 관리
- **GotoDialog**: ModalDialog 기반, 쪽 번호 입력 → 이동
- **다시 찾기**: 대화상자 없이 WASM 직접 검색
- **커맨드/단축키**: Ctrl+F, Ctrl+F2, Ctrl+L, Alt+G
- **메뉴**: 4개 항목 활성화

## 변경 파일 목록

| 파일 | 변경 내용 |
|------|-----------|
| `src/document_core/queries/search_query.rs` | 신규: 검색/치환 엔진 + 쪽↔위치 변환 |
| `src/document_core/queries/mod.rs` | search_query 모듈 등록 |
| `src/wasm_api.rs` | 5개 WASM API 추가 |
| `rhwp-studio/src/core/types.ts` | SearchResult, ReplaceResult 등 인터페이스 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 5개 API 래퍼 |
| `rhwp-studio/src/ui/find-dialog.ts` | 신규: 찾기/찾아바꾸기 모달리스 대화상자 |
| `rhwp-studio/src/ui/goto-dialog.ts` | 신규: 찾아가기 대화상자 |
| `rhwp-studio/src/styles/find-dialog.css` | 신규: 대화상자 스타일 |
| `rhwp-studio/src/style.css` | import 추가 |
| `rhwp-studio/src/command/commands/edit.ts` | 4개 커맨드 구현 |
| `rhwp-studio/src/command/shortcut-map.ts` | 5개 단축키 추가 |
| `rhwp-studio/index.html` | 메뉴 4개 항목 갱신 |
| `mydocs/orders/20260316.md` | Task 238 등록 |

## 검증 결과
- cargo test: 716개 통과, 0개 실패
- WASM 빌드: 성공
- TypeScript 타입 체크: 새 에러 없음
