# Task 252 수행 계획서: 각주 편집 기능

## 한컴 동작 (도움말 기준)

### 각주 삽입 (Ctrl+N,N)
1. 본문에서 커서 위치에 각주 번호가 자동 매겨짐
2. 커서가 쪽 아래 각주 영역으로 이동
3. 각주 내용 입력
4. Shift+Esc로 본문 복귀

### 각주 편집
- 본문에서 각주 번호 더블클릭 → 각주 영역으로 이동
- 각주 영역에서 직접 클릭하여 편집
- Shift+Esc로 본문 복귀

### 각주 삭제
- 본문에서 각주 번호를 Delete/Backspace로 삭제 → 각주 내용도 함께 삭제
- 나머지 각주 번호 자동 재정렬

## 구현 계획

### 1단계: 각주 삽입 WASM API
- Rust: `insert_footnote_native(sec, para, char_offset)` → Footnote 컨트롤 생성
- 빈 문단 1개 포함, 자동 번호 할당
- WASM: `insertFootnote` 바인딩
- 페이지네이션 + 렌더 트리 재생성

### 2단계: 각주 영역 편집 모드
- 각주 영역 클릭 → 각주 편집 모드 진입 (머리말/꼬리말과 유사)
- 각주 내 커서 이동 + 텍스트 입력/삭제
- Shift+Esc → 본문 복귀
- 본문에서 각주 번호 더블클릭 → 해당 각주 편집

### 3단계: 각주 삭제 + 번호 재정렬
- 본문에서 각주 번호 컨트롤 삭제 시 Footnote 컨트롤 제거
- 남은 각주의 번호 자동 재정렬

### 참조
- 한컴 도움말: insert/annotations/footnotes.htm
- 모델: src/model/footnote.rs (Footnote, FootnoteShape)
- 레이아웃: src/renderer/layout/picture_footnote.rs
- 파서: src/parser/control.rs (parse_footnote_control)
