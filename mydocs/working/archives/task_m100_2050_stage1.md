# Stage 1 완료보고서 — task_m100_2050

- **이슈**: #2050  **브랜치**: `local/task2050`

## 작업: `getStructure` WASM 바인딩

### 변경 1: 코어 메서드 (`src/document_core/queries/structure.rs`)
- import 추가: `DocumentCore`, `HwpError`
- `impl DocumentCore`에 `get_structure_native(&self, mode: &str) -> Result<String, HwpError>` 추가
  - `StructureMode::parse(mode)` (실패 시 `auto` 폴백) → 기존 `build_structure(&self.document, mode)` 재사용 → `serde_json` 직렬화
  - serde 오류는 `HwpError::RenderError`로 매핑 (기존 `rendering.rs` 패턴 준수)

### 변경 2: WASM 바인딩 (`src/wasm_api.rs`)
- `getBookmarks` 옆에 `#[wasm_bindgen(js_name = getStructure)] pub fn get_structure(&self, mode: &str)` 추가 → `core.get_structure_native` 위임

## 검증
- `cargo build` 성공
- `cargo test --lib structure` → 10 passed, 0 failed (기존 structure 테스트 회귀 없음)

## 범위
- 파서/렌더/레이아웃/문서코어 로직 무변경. 기존 `build_structure`(읽기 전용 질의) 노출만.

## 상태: ✅ 완료
