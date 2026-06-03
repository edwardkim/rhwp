# Task 231 2단계 완료 보고서: 필드 내 텍스트 입력/삭제

## 구현 결과

### Rust 측

**`src/document_core/queries/field_query.rs`**:
- `get_field_info_at(section, para, char_offset)` — 본문 문단 커서 위치의 필드 범위 조회
- `get_field_info_at_in_cell(section, parent_para, ctrl, cell, cell_para, offset, is_textbox)` — 셀/글상자 내 필드 범위 조회
- `field_info_at_in_para()` 내부 함수 — 문단의 field_ranges에서 ClickHere 필드 범위 탐색
- 반환 JSON: `{inField, fieldId, fieldType, startCharIdx, endCharIdx, isGuide, guideName}`

**`src/wasm_api.rs`**:
- `getFieldInfoAt(section, para, charOffset)` — WASM API
- `getFieldInfoAtInCell(section, ppi, ci, cei, cpi, offset, isTextbox)` — WASM API

### 프론트엔드

**`rhwp-studio/src/core/types.ts`**:
- `FieldInfoResult` 인터페이스 추가

**`rhwp-studio/src/core/wasm-bridge.ts`**:
- `getFieldInfoAt(pos: DocumentPosition)` 래퍼 추가 — 본문/셀/글상자 자동 분기

**`rhwp-studio/src/engine/input-handler-text.ts`**:
- `handleBackspace`: 필드 시작 위치에서 Backspace 차단 (`charOffset <= startCharIdx`)
- `handleDelete`: 필드 끝 위치에서 Delete 차단 (`charOffset >= endCharIdx`)

### 자동 동작 (기존 코드 활용)

- 빈 필드(start==end)에 텍스트 입력 시 `insert_text_at()`이 `field_range.end_char_idx`를 자동 확장 → 안내문 사라지고 사용자 텍스트 표시
- 필드 내 텍스트 삭제 시 `delete_text_at()`이 `field_range` 자동 축소
- 필드가 비면(start==end) 안내문 자동 재표시 (렌더러가 빈 필드 감지)

## 경계 보호 검증

| 상황 | 동작 |
|------|------|
| 필드 시작에서 Backspace | 차단 |
| 필드 끝에서 Delete | 차단 |
| 빈 필드에서 Backspace | 차단 (start == end == cursor) |
| 빈 필드에서 Delete | 차단 (start == end == cursor) |
| 필드 내부에서 Backspace | 허용 (텍스트 삭제) |
| 필드 내부에서 Delete | 허용 (텍스트 삭제) |

## 테스트 결과

- 703개 테스트 전체 통과
- Rust 빌드 정상
- TypeScript 타입 체크 정상
