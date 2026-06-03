# 구현 계획서: HWPX 저장 WASM API 노출 및 사용자 오류 전달

- **타스크**: [#176](https://github.com/edwardkim/rhwp/issues/176)
- **마일스톤**: M100
- **브랜치**: `local/task176`
- **작성일**: 2026-04-17
- **수행계획서**: `mydocs/plans/task_m100_176.md`

## 단계 구성 (3단계)

---

### 1단계: WASM API에 exportHwpx() 추가

**수정 파일**:
- `src/document_core/commands/document.rs` — `export_hwpx_native()` 추가
- `src/wasm_api.rs` — `exportHwpx()` WASM 바인딩 추가

**변경 내용**:

```rust
// document.rs
pub fn export_hwpx_native(&self) -> Result<Vec<u8>, HwpError> {
    crate::serializer::serialize_hwpx(&self.document)
        .map_err(|e| HwpError::RenderError(e.to_string()))
}

// wasm_api.rs
#[wasm_bindgen(js_name = exportHwpx)]
pub fn export_hwpx(&self) -> Result<Vec<u8>, JsValue> {
    self.export_hwpx_native().map_err(|e| e.into())
}
```

---

### 2단계: SaveAs() 파일 형식 분기

**수정 파일**:
- `src/document_core/mod.rs` 또는 `src/wasm_api.rs` — 원본 파일 형식 추적 필드 추가
- `src/wasm_api.rs` — `getSourceFormat()` WASM 바인딩
- `rhwp-studio/src/hwpctl/index.ts` — `SaveAs()`에서 형식 분기

**변경 내용**:
- `DocumentCore`에 `source_format: FileFormat` 필드 추가
- 파일 로드 시 `detect_format()`으로 판별하여 저장
- `SaveAs()`에서 `getSourceFormat()` → "hwpx"면 `exportHwpx()`, "hwp"면 `exportHwp()`
- 확장자를 원본 형식에 맞게 지정 (`.hwp` 또는 `.hwpx`)

---

### 3단계: 오류 전달 및 테스트

- `cargo test` 전체 통과
- WASM 빌드
- 웹뷰어에서 HWPX 열기 → 편집 → 저장 → HWPX 파일 확인

---

## 검증 기준

| 단계 | 검증 항목 |
|------|----------|
| 1단계 | `cargo test` 통과, `exportHwpx()` 함수 존재 |
| 2단계 | HWPX 열고 저장 시 `.hwpx` 확장자로 저장 |
| 3단계 | WASM 빌드 + 웹뷰어 검증 |
