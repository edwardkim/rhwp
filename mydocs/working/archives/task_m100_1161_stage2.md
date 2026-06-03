# 단계별 완료 보고서 — Task M100-1161 Stage 2

## 목표

wasm_api 클립보드 4 래퍼에 `cell_path_json` 인자 연결(WASM 경계에 cell_path 노출).

## 변경 사항

### `src/wasm_api.rs`
- **공통 파싱 헬퍼** `parse_cell_path_arg(cell_path_json) -> Result<Vec<(usize,usize,usize)>, JsValue>` 신설
  (빈 문자열/`"[]"` = 본문, 그 외 `parse_cell_path` 위임). `insert_picture` 인라인 패턴을 헬퍼로 통합.
- 4 래퍼에 `cell_path_json: &str` 인자 추가 후 `parse_cell_path_arg` → native 전달:
  - `copyControl`, `exportControlHtml`, `getControlImageData`, `getControlImageMime`
  - 인자 위치는 native 정합 순서(`section, para, cell_path, control`).

### `src/wasm_api/tests.rs`
- `test_clipboard_copy_control_cell_path_json_arg` 신설: 빈 문자열/`"[]"` → 본문 복사("[표]") 검증.
  (에러 경로는 JsValue 구성으로 native 테스트 abort → OK 경로만. cell 경로 자체는 Stage 1 통합 테스트로 가드.)

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --lib clipboard` | ✅ 6 passed (신규 1 포함) |
| `cargo test --test issue_1161_copy_picture_in_cell` (Stage 1 회귀) | ✅ 4 passed |
| `cargo fmt --check`(변경 파일) | ✅ |
| `cargo clippy --lib` | ✅ 0 warning |

## 비고

- Rust 측 4 래퍼의 다른 호출처 없음(JS 전용) → TS 배선은 Stage 4.
- 기존 native 호출처(main.rs, Stage 1 테스트)는 `&[]` 그대로 유효.

## 다음 단계

Stage 3 — ImageNode 에 `cell_context` 필드 추가 + `make_picture_image_node` 단일 chokepoint 에서
다단계 path 보존 + rendering.rs ImageNode `cellPath:[...]` 방출 + 회귀 테스트.
