# Task 238 3단계 완료 보고서: 찾기/찾아바꾸기 대화상자

## 완료 항목

### FindDialog (find-dialog.ts)
- **모달리스 대화상자**: overlay 없이 편집 영역 조작 가능
- **찾기 모드**: 검색어 입력, 다음 찾기/이전 찾기
- **바꾸기 모드**: 바꿀 내용 입력, 바꾸기/모두 바꾸기
- **대소문자 구분**: 체크박스 옵션
- **검색 결과 하이라이트**: cursor setAnchor + moveTo로 선택 영역 표시
- **결과 스크롤**: moveCursorTo + updateCaret으로 검색 위치로 자동 스크롤
- **키보드 처리**: Enter=다음 찾기, Shift+Enter=이전 찾기, Escape=닫기
- **드래그 이동**: 타이틀 바 드래그로 대화상자 위치 이동
- **싱글톤 관리**: 이미 열려 있으면 포커스, 모드 전환 가능
- **마지막 검색어 기억**: static lastQuery, lastCaseSensitive

### 다시 찾기 (Ctrl+L) 개선
- 대화상자가 열려 있으면 findNext() 호출
- 대화상자 없이도 lastQuery로 직접 WASM 검색 + 선택 영역 표시

### CSS (find-dialog.css)
- 고정 위치 (우측 상단), 한글 워드 스타일 UI

## 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/find-dialog.ts` | 전체 재작성 (~270줄) |
| `rhwp-studio/src/styles/find-dialog.css` | 신규 |
| `rhwp-studio/src/style.css` | import 추가 |
| `rhwp-studio/src/command/commands/edit.ts` | edit:find-again 직접 검색 로직 |

## 검증
- TypeScript 타입 체크: 새 에러 없음
