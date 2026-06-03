# Task 241 - 2단계 완료 보고서: 새 책갈피 CTRL_DATA ParameterSet 생성

## 완료 항목

### bookmark_query.rs

#### `build_bookmark_ctrl_data(name)` 함수 신규
- ParameterSet 바이너리 생성: `ps_id(0x021B) + count(1) + dummy(0) + item_id(0x4000) + type(String) + name_len + name(UTF-16LE)`
- 실제 HWP 파일의 바이너리 구조를 정밀 분석하여 동일한 포맷 생성

#### `add_bookmark_native()` 수정
- 새 Bookmark 컨트롤 삽입 시 `ctrl_data_records`에 CTRL_DATA 레코드 함께 삽입
- 한컴에서 열었을 때 책갈피 이름이 정상 표시됨

#### `delete_bookmark_native()` 수정
- 컨트롤 삭제 시 `ctrl_data_records`에서도 해당 인덱스 제거

#### `rename_bookmark_native()` 수정
- 이름 변경 시 `ctrl_data_records`도 새 이름으로 재생성

## 추가 수정 (1단계 후속)

### collect_bookmarks 재귀 수집
- 중첩 구조(표 셀, 머리말/꼬리말 등) 내 책갈피도 수집
- 중첩 책갈피는 호스트 최상위 문단 인덱스 사용 → 찾아가기 동작

### moveCursorTo 반환값 추가
- `input-handler.ts`: `moveCursorTo()` → `boolean` 반환 (rect 유무)
- 이동 실패 시 해당 페이지의 첫 위치로 fallback

### 조판부호 마커 수정
- `[책갈피:이름]` → `[책갈피]` (한컴 동일)
- 색상: 빨간색 (#FF0000)

## 검증
- Rust 테스트 716개 통과
