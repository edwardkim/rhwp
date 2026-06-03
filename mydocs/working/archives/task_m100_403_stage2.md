# Task #403 Stage 2 완료 보고서

**제목**: Native FFI 구현  
**브랜치**: `local/ffi-csharp`  
**이슈**: https://github.com/edwardkim/rhwp/issues/403

---

## 1. 작업 요약

`bindings/Native`에 Rust `cdylib` crate를 추가하고, `export-text`, `export-markdown` 기능을 C ABI로 노출했다.

## 2. Crate 구성

`bindings/Native/Cargo.toml`:

| 항목 | 값 |
|------|-----|
| package | `rhwp-native-ffi` |
| lib name | `rhwp_native_ffi` |
| crate type | `cdylib` |
| rhwp 의존성 | `rhwp_core = { package = "rhwp", path = "../.." }` |

## 3. Export ABI

`bindings/Native/src/lib.rs`에 다음 함수를 추가했다.

```rust
rhwp_export_text(input_path, output_dir, page) -> *mut c_char
rhwp_export_markdown(input_path, output_dir, page) -> *mut c_char
rhwp_string_free(ptr)
```

호출 규칙:

- `input_path`, `output_dir`: null-terminated UTF-8 C string
- `page = -1`: 전체 페이지 export
- `page >= 0`: 0-based 단일 페이지 export
- 반환값: UTF-8 JSON 문자열 포인터
- 반환 포인터는 반드시 `rhwp_string_free`로 해제

## 4. Export 동작

### 4.1 Text export

1. 입력 HWP 파일 읽기
2. `HwpDocument::from_bytes` 파싱
3. 대상 페이지 선택
4. `extract_page_text_native` 호출
5. 출력 폴더에 `.txt` 저장
6. 저장된 파일 목록 JSON 반환

### 4.2 Markdown export

1. 입력 HWP 파일 읽기
2. `HwpDocument::from_bytes` 파싱
3. 대상 페이지 선택
4. `extract_page_markdown_with_images_native` 호출
5. 이미지 토큰을 Markdown 링크로 치환
6. 이미지 파일은 `{stem}_assets/` 하위에 저장
7. `.md` 파일과 image count를 JSON 반환

## 5. 오류 처리

| 오류 영역 | 처리 |
|----------|------|
| null pointer | 실패 JSON 반환 |
| invalid UTF-8 | 실패 JSON 반환 |
| 파일 읽기 실패 | 실패 JSON 반환 |
| HWP 파싱 실패 | 실패 JSON 반환 |
| 페이지 범위 오류 | 실패 JSON 반환 |
| 출력 폴더/파일 저장 실패 | 실패 JSON 반환 |
| panic | `catch_unwind` 후 실패 JSON 반환 |

성공 예:

```json
{"ok":true,"pageCount":3,"files":["..."],"imageCount":2}
```

실패 예:

```json
{"ok":false,"error":"..."}
```

## 6. 메모리 관리

native 반환 문자열은 `CString::into_raw()`로 호출자에게 넘긴다. 모든 언어 바인딩은 사용 후 `rhwp_string_free()`를 호출해야 한다.

Stage 2 완료. 공통 Native ABI가 구현되었고, C# 외 다른 언어 binding도 동일 ABI 위에 추가 가능하다.
