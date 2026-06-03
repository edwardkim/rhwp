# Task 238 2단계 완료 보고서: 프론트엔드 API 연결 및 커맨드/단축키

## 완료 항목

### TypeScript 인터페이스 (types.ts)
- `SearchResult` — 검색 결과 (found, sec, para, charOffset, length, cellContext)
- `ReplaceResult` — 단일 치환 결과
- `ReplaceAllResult` — 전체 치환 결과
- `PageOfPositionResult` — 쪽 번호 조회 결과

### WASM 브릿지 래퍼 (wasm-bridge.ts)
- `searchText()`, `replaceText()`, `replaceAll()`, `getPageOfPosition()`
- typeof 방어 코드 포함

### 커맨드 구현 (edit.ts)
- `edit:find` — FindDialog 싱글톤, 이미 열려 있으면 포커스
- `edit:find-replace` — FindDialog replace 모드, 이미 열려 있으면 모드 전환
- `edit:find-again` — 대화상자 열려 있으면 다음 검색, 없으면 마지막 검색어로 새 대화상자
- `edit:goto` — GotoDialog 생성

### 단축키 (shortcut-map.ts)
- Ctrl+F → `edit:find`
- Ctrl+F2 → `edit:find-replace`
- Ctrl+L → `edit:find-again`
- Alt+G / Alt+ㅎ → `edit:goto`

### 메뉴 HTML (index.html)
- 찾기(F), 찾아 바꾸기(E), 다시 찾기(X), 찾아가기(G) 4개 항목 활성화

### 대화상자 스텁
- `find-dialog.ts` — 3단계에서 본격 구현
- `goto-dialog.ts` — 4단계에서 본격 구현

## 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/core/types.ts` | 4개 인터페이스 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 4개 API 래퍼 |
| `rhwp-studio/src/command/commands/edit.ts` | 4개 커맨드 구현 |
| `rhwp-studio/src/command/shortcut-map.ts` | 5개 단축키 추가 |
| `rhwp-studio/index.html` | 메뉴 4개 항목 갱신 |
| `rhwp-studio/src/ui/find-dialog.ts` | 신규 (스텁) |
| `rhwp-studio/src/ui/goto-dialog.ts` | 신규 (스텁) |

## 검증
- TypeScript 타입 체크: 새 에러 없음
- cargo test: 716개 통과
