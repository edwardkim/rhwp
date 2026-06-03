# Task 231 최종 보고서: 누름틀 편집 UI

## 구현 결과

### 1단계: 필드 클릭 진입 + 커서 배치

**Rust 측** (`src/document_core/queries/cursor_rect.rs`):
- `GuideRunInfo` 구조체 — 안내문 TextRun (char_start: None) 정보 수집
- `collect_runs()` 확장 — guide_runs 파라미터 추가
- hitTest에 안내문 영역 감지 추가 (step 0)
- `find_field_hit_for_guide()` — 필드 시작 위치로 커서 반환, JSON에 `isField:true, fieldId, fieldType` 포함

**프론트엔드**:
- `HitTestResult`에 `isField?, fieldId?, fieldType?` 추가
- 클릭 시 `field-info-changed` 이벤트 발생
- 상태 표시줄에 `sb-field` 스팬 추가

### 2단계: 필드 내 텍스트 입력/삭제

**Rust 측** (`src/document_core/queries/field_query.rs`):
- `get_field_info_at()` — 본문 문단 커서 위치의 필드 범위 조회
- `get_field_info_at_in_cell()` — 셀/글상자 내 필드 범위 조회
- 반환: `{inField, fieldId, fieldType, startCharIdx, endCharIdx, isGuide, guideName}`

**WASM API** (`src/wasm_api.rs`):
- `getFieldInfoAt(section, para, charOffset)`
- `getFieldInfoAtInCell(section, ppi, ci, cei, cpi, offset, isTextbox)`

**프론트엔드**:
- `FieldInfoResult` 인터페이스 + `WasmBridge.getFieldInfoAt()` 래퍼
- Backspace: 필드 시작에서 차단 (`charOffset <= startCharIdx`)
- Delete: 필드 끝에서 차단 (`charOffset >= endCharIdx`)
- 기존 `insert_text_at`/`delete_text_at`이 field_range 자동 확장/축소

### 3단계: F11 블록 선택 + 상태 표시줄

**프론트엔드**:
- F11 키: 필드 내 텍스트 전체 블록 선택
- 상태 표시줄: `[누름틀] {안내문}` 형식 표시

## 변경 파일

| 파일 | 변경 |
|------|------|
| `src/document_core/queries/cursor_rect.rs` | GuideRunInfo, guide text hitTest, find_field_hit_for_guide |
| `src/document_core/queries/field_query.rs` | get_field_info_at, get_field_info_at_in_cell, field_info_at_in_para |
| `src/wasm_api.rs` | getFieldInfoAt, getFieldInfoAtInCell API 추가 |
| `rhwp-studio/src/core/types.ts` | HitTestResult 확장, FieldInfoResult 추가 |
| `rhwp-studio/src/core/wasm-bridge.ts` | getFieldInfoAt 래퍼 추가 |
| `rhwp-studio/src/engine/input-handler-mouse.ts` | 필드 클릭 이벤트 + guideName 조회 |
| `rhwp-studio/src/engine/input-handler-text.ts` | Backspace/Delete 필드 경계 보호 |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | F11 필드 블록 선택 |
| `rhwp-studio/src/main.ts` | field-info-changed 이벤트 리스너 |
| `rhwp-studio/index.html` | sb-field 스팬 추가 |

## 테스트 결과

- 704개 테스트 실행, 703개 통과 (1개 ignored)
