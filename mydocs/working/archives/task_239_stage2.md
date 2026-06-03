# Task 239 - 2단계 완료 보고서: 명령 연결 및 문자 삽입

## 완료 항목

### insert.ts
- `insert:symbols` stub → 실제 구현
- `SymbolsDialog` import 및 싱글톤 인스턴스 관리
- `canExecute: (ctx) => ctx.hasDocument` — 문서 로드 시 활성화
- `InsertTextCommand`로 커서 위치에 문자 삽입 (undo/redo 지원)
- `document-changed` 이벤트로 화면 갱신

### index.html
- 메뉴 > 입력 > 문자표: `disabled` 클래스 제거
- 도구상자 문자표 버튼: `data-cmd="insert:symbols"` + title에 단축키 표시

### vite-env.d.ts (신규)
- Vite `import.meta.env` 타입 선언 추가
- 기존 tsc 오류 2건 해결

## 진입점
- 도구상자 "문자표" 버튼 클릭
- 메뉴 > 입력 > 문자표
- 단축키 Alt+F10 (shortcut-map 등록은 3단계에서 확인)

## 검증
- TypeScript 컴파일 오류 없음
