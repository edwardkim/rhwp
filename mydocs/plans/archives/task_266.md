# Task 266 수행 계획서: 감추기 편집 + 조판부호 보이기 확장

## 구현 범위

### A. 감추기 편집 기능 (Ctrl+N,S)
- 현재 커서가 있는 쪽에 PageHide 컨트롤 삽입/제거
- 대화상자: 감출 내용 선택 (머리말, 꼬리말, 쪽 번호, 쪽 테두리, 배경, 바탕쪽)
- 한컴 동작: 조판부호는 현재 문단의 맨 앞에 삽입

### B. 조판부호 보이기 확장
- 기존 조판부호 (구역나누기, 쪽나누기 등)에 추가:
  - `[감추기]` — PageHide 컨트롤
  - `[쪽 번호 위치]` — PageNumberPos 컨트롤
  - `[머리말(양 쪽)]` / `[꼬리말(양 쪽)]` — Header/Footer 컨트롤

## 구현 계획

### 1단계: Rust API — 감추기 삽입/제거
- `insert_page_hide_native(sec, para, hide_flags)` — PageHide 컨트롤 삽입
- `remove_page_hide_native(sec, para)` — PageHide 컨트롤 제거
- `get_page_hide_native(sec, para)` — 현재 문단의 PageHide 조회

### 2단계: WASM 바인딩 + 대화상자
- WASM: insertPageHide, removePageHide, getPageHide
- 대화상자: 체크박스 6개 (머리말, 꼬리말, 쪽 번호, 테두리, 배경, 바탕쪽)
- 단축키: Ctrl+N,S (코드 단축키 chordMapN)

### 3단계: 조판부호 렌더링
- 조판부호 보이기 모드에서 컨트롤 마커 텍스트 표시
- `[감추기]`, `[쪽 번호 위치]`, `[머리말(양 쪽)]` 등

## 참조
- 한컴 도움말: format/hide.htm
- 단축키: Ctrl+N,S
- HWP 스펙: 표 147 (감추기), 표 149 (쪽 번호 위치)
