# Task #1951 구현 계획서

## 1. 좌표 계약

`CursorRect`는 기존 `pageIndex`, `x`, `y`, `height`를 유지한다. 표 셀 내부에서 반환할 때만
선택적 `cellBounds: { x, y, w, h }`를 함께 포함한다.

- `x`는 셀 좌우 경계 안에 제한한다.
- `y`는 caret 높이를 포함해 셀 상하 경계 안에 제한한다.
- 실제 TextRun의 좌표와 셀 bbox는 모두 같은 render tree에서 얻는다.
- 텍스트 모델, char offset, LINE_SEG, 저장 셀 높이는 바꾸지 않는다.

## 2. 수정 범위

- `src/document_core/queries/cursor_rect.rs`
  - 단일 셀과 cell path cursor query에서 현재 TextRun이 속한 `TableCell` bbox를 함께 추적한다.
  - 계산된 caret rect를 해당 bbox에 맞춰 제한하고 `cellBounds`를 직렬화한다.
- `rhwp-studio/src/core/types.ts`
  - `CursorRect.cellBounds` 선택 타입을 추가한다.
- `rhwp-studio/src/engine/caret-renderer.ts`
  - 일반 caret과 IME 조합창의 DOM left/top/width/height를 `cellBounds`에 맞춰 제한한다.
- `tests/issue_1951_table_cell_cursor_clip.rs`
  - `복학원서.hwp`에 장문 입력 후 단일/path cursor rect가 대상 셀 bbox 안에 있는지 검증한다.
- `rhwp-studio/tests/`
  - renderer 소스 계약 테스트로 cell bounds가 IME 조합창의 위치와 크기에 적용되는지 검증한다.

## 3. 회귀 위험과 방어

- 셀 bbox를 매 입력마다 별도 조회하지 않는다. cursor query가 이미 빌드한 page render tree의
  `TableCell` 조상 bbox를 재사용한다.
- 고정 셀 밖의 숨은 텍스트는 보존한다. 이번 수정은 UI cursor와 조합창의 가시 위치만 정한다.
- `cellBounds`가 없는 본문/글상자/각주/머리말 경로는 기존 DOM 배치를 그대로 유지한다.
- 최소 가시 폭이나 높이보다 작은 셀도 DOM 요소가 셀 밖으로 나가지 않도록 width/height를
  셀 크기에 맞춰 제한한다.
