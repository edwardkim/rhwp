# Task 255 수행 계획서: 각주 내용 편집 기능

## 현재 상태

- 각주 삽입(`insertFootnote`) WASM API 구현 완료
- 각주 번호 인라인 렌더링 구현 완료
- 각주 영역 하단 표시 구현 완료
- **미구현**: 각주 내용 텍스트 입력/삭제/문단분할/병합

## 참조 패턴

머리말/꼬리말 편집 API (`header_footer_ops.rs`)와 동일한 패턴 적용:
- 각주 컨트롤 내부의 `paragraphs: Vec<Paragraph>`에 대해 텍스트 편집
- 편집 후 리플로우 + raw_stream 무효화 + 재페이지네이션

## 구현 계획

### 1단계: 각주 편집 Rust API (document_core)

`src/document_core/commands/footnote_ops.rs` 신규 파일:

| API | 설명 |
|-----|------|
| `find_footnote_control()` | 구역 내 (para_idx, control_idx)로 각주 컨트롤 위치 |
| `get_footnote_paragraph_mut()` | 각주 내부 문단 가변 참조 |
| `get_footnote_info_native()` | 각주 문단 수/텍스트 길이 조회 |
| `insert_text_in_footnote_native()` | 각주 내 텍스트 삽입 |
| `delete_text_in_footnote_native()` | 각주 내 텍스트 삭제 |
| `split_paragraph_in_footnote_native()` | 각주 내 문단 분할 (Enter) |
| `merge_paragraph_in_footnote_native()` | 각주 내 문단 병합 (Backspace) |
| `reflow_footnote_paragraph()` | 각주 문단 리플로우 |

### 2단계: WASM 바인딩

`src/wasm_api.rs`에 각 API의 `#[wasm_bindgen]` 바인딩 추가:
- `getFootnoteInfo(sec, paraIdx, controlIdx)`
- `insertTextInFootnote(sec, paraIdx, controlIdx, fnParaIdx, charOffset, text)`
- `deleteTextInFootnote(sec, paraIdx, controlIdx, fnParaIdx, charOffset, count)`
- `splitParagraphInFootnote(sec, paraIdx, controlIdx, fnParaIdx, charOffset)`
- `mergeParagraphInFootnote(sec, paraIdx, controlIdx, fnParaIdx)`

### 3단계: rhwp-studio UI 연동

- `wasm-bridge.ts`에 각주 편집 메서드 추가
- 각주 영역 클릭 → 각주 편집 모드 진입
- 각주 내 텍스트 입력/삭제/Enter/Backspace 처리

### 참조 파일

- 머리말/꼬리말 패턴: `src/document_core/commands/header_footer_ops.rs`
- 각주 삽입: `src/document_core/commands/object_ops.rs` (insert_footnote_native)
- 각주 모델: `src/model/footnote.rs`
- WASM: `src/wasm_api.rs`
