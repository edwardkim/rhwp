# Task 240 - 3단계 완료 보고서: TypeScript 대화상자 UI

## 완료 항목

### types.ts
- `BookmarkInfo` 인터페이스 추가 (name, sec, para, ctrlIdx, charPos)

### wasm-bridge.ts
- `getBookmarks()` — 책갈피 목록 조회
- `addBookmark()` — 책갈피 추가
- `deleteBookmark()` — 책갈피 삭제
- `renameBookmark()` — 책갈피 이름 변경

### bookmark-dialog.ts (신규)
- 한컴 UI 참고: 이름 입력란 + 목록(이름/종류) + 넣기/취소/이동 버튼
- 이름 바꾸기(✏) / 삭제(✕) 아이콘 버튼
- 정렬 기준: 이름(A) / 위치(P) 라디오
- 중복 이름 거부 + 에러 메시지 표시
- 삭제 시 확인 대화상자
- 더블 클릭 시 이동
- 기본 이름 제안 (책갈피N)

### bookmark-dialog.css (신규)
- 360px 폭 대화상자, 목록 160px 높이

### style.css
- CSS import 추가

## 검증
- TypeScript 컴파일 오류 없음
