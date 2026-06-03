# Task 275: 중첩 표 셀 편집 지원

## 목표

표 > 셀 > 표 > 셀 구조(중첩 표)에서 커서 진입, 텍스트 편집, 누름틀 감지가 정상 동작하도록 한다.

## 현황 분석

### 정상 동작하는 부분
- `hitTest`: 중첩 표 위치를 `cellPath[]` 배열로 정확히 반환
- `cursor.moveTo()`: `cellPath`를 `DocumentPosition`에 저장
- `getCellParagraphCountByPath()`, `getCellParagraphLengthByPath()`: path 기반 쿼리 API 존재

### 문제점
- 텍스트 편집 WASM API가 **flat 파라미터만 지원** (depth=1만 가능)
  - `insertTextInCell(sec, ppi, ci, cei, cpi, offset, text)`
  - `deleteTextInCell(sec, ppi, ci, cei, cpi, offset, count)`
  - `splitParagraphInCell(sec, ppi, ci, cei, cpi, offset)`
  - `mergeParagraphInCell(sec, ppi, ci, cei, cpi)`
- TypeScript command layer (`command.ts`)가 `cellPath`를 무시하고 flat 파라미터만 전달
- `getFieldInfoAt`가 중첩 표 내부 필드를 감지하지 못함
- `setActiveFieldInCell`이 중첩 표를 지원하지 않음

### 영향 범위
- 파일: BlogForm_BookReview.hwp (표 > 셀 > 7×2 표 > 셀 > 누름틀)
- 기타 중첩 표가 포함된 HWP 파일 전반

## 구현 계획

### 1단계: WASM path 기반 편집 API 추가

**수정 파일**: `src/wasm_api.rs`, `src/document_core/commands/editing.rs`

- `cellPath` JSON 문자열을 파싱하여 중첩 표의 실제 문단에 접근하는 헬퍼 함수 추가
  - `resolve_cell_path(document, sec, ppi, path_json) -> &mut Paragraph`
- path 기반 편집 API 4종 추가:
  - `insertTextInCellByPath(sec, ppi, path_json, offset, text)`
  - `deleteTextInCellByPath(sec, ppi, path_json, offset, count)`
  - `splitParagraphInCellByPath(sec, ppi, path_json, offset)`
  - `mergeParagraphInCellByPath(sec, ppi, path_json)`

### 2단계: TypeScript command layer 수정

**수정 파일**: `rhwp-studio/src/engine/command.ts`, `rhwp-studio/src/core/wasm-bridge.ts`

- `wasm-bridge.ts`에 path 기반 API 래퍼 추가
- `command.ts`의 `doInsertText`, `doDeleteText`, `doSplitParagraph`, `doMergeParagraph`에서
  `pos.cellPath?.length > 1`이면 path 기반 API 호출

### 3단계: 중첩 표 필드 감지 + 활성 필드

**수정 파일**: `src/document_core/queries/field_query.rs`, `src/wasm_api.rs`

- `getFieldInfoAt`를 path 기반으로 확장: `getFieldInfoAtByPath(sec, ppi, path_json, offset)`
- `setActiveFieldInCell`을 path 기반으로 확장
- TypeScript `updateFieldMarkers()`에서 `cellPath`가 있으면 path 기반 API 사용

### 4단계: E2E 검증

**수정 파일**: `rhwp-studio/e2e/blogform.test.mjs`

- BlogForm_BookReview.hwp 중첩 표 셀 클릭 → 누름틀 안내문 숨김 확인
- 중첩 표 셀에서 텍스트 입력 → 정상 입력 확인

## 검증 방법

1. `cargo test` 전체 통과
2. BlogForm_BookReview.hwp E2E: 중첩 표 "제목" 셀 클릭 → 안내문 숨김 + 텍스트 입력 가능
3. 기존 단일 레벨 표 편집 기능 정상 동작 확인
