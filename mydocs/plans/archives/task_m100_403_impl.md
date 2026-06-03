# Task #403 구현 계획서 — AI 워크로드를 위한 FFI 인터페이스 개발

**제목**: AI 워크로드를 위한 FFI 인터페이스 개발  
**마일스톤**: M100 (v1.0.0)  
**Issue**: https://github.com/edwardkim/rhwp/issues/403  
**브랜치**: `local/ffi-csharp`  
**수행계획서**: `mydocs/plans/task_m100_403.md`

---

## 1. 구현 요약

console 프로세스 실행 방식의 `export-text`, `export-markdown` 기능을 AI/서버 워크로드에서 직접 호출할 수 있도록 `bindings` 하위에 별도 FFI 계층을 추가한다.

현재 git 변경 목록 기준 구현은 다음 구조를 따른다.

| 경로 | 역할 |
|------|------|
| `bindings/README.md` | bindings 디렉터리 구조와 확장 규칙 문서화 |
| `bindings/Native/` | Rust `cdylib` 기반 공통 C ABI 구현 |
| `bindings/Native/src/lib.rs` | `export-text`, `export-markdown` FFI entry point |
| `bindings/csharp/RhwpNative.cs` | C# P/Invoke wrapper |

`bindings/Native`는 언어별 폴더가 아니라 모든 언어 바인딩이 공유할 네이티브 ABI 계층이다. 언어별 구현은 `bindings/csharp`처럼 `bindings`의 sibling 폴더로 추가한다.

## 2. Native FFI 설계

### 2.1 Crate 구성

`bindings/Native/Cargo.toml`:

| 항목 | 값 |
|------|-----|
| package | `rhwp-native-ffi` |
| lib name | `rhwp_native_ffi` |
| crate type | `cdylib` |
| rhwp 의존성 | `rhwp_core = { package = "rhwp", path = "../.." }` |

빌드 산출물 이름은 플랫폼별 동적 라이브러리 규칙을 따른다.

- Windows: `rhwp_native_ffi.dll`
- macOS: `librhwp_native_ffi.dylib`
- Linux: `librhwp_native_ffi.so`

### 2.2 Export 함수

현재 공개 ABI는 수행계획서 범위대로 `export-text`, `export-markdown`에 한정한다.

```rust
#[no_mangle]
pub extern "C" fn rhwp_export_text(
    input_path: *const c_char,
    output_dir: *const c_char,
    page: i32,
) -> *mut c_char

#[no_mangle]
pub extern "C" fn rhwp_export_markdown(
    input_path: *const c_char,
    output_dir: *const c_char,
    page: i32,
) -> *mut c_char

#[no_mangle]
pub extern "C" fn rhwp_string_free(ptr: *mut c_char)
```

호출 규약:

- `input_path`, `output_dir`는 null-terminated UTF-8 문자열이다.
- `page = -1`이면 전체 페이지, `page >= 0`이면 해당 0-based 페이지 하나만 export한다.
- 반환값은 UTF-8 JSON 문자열 포인터다.
- 호출자는 반환 포인터를 사용한 뒤 반드시 `rhwp_string_free`로 해제해야 한다.

### 2.3 반환 JSON

성공:

```json
{"ok":true,"pageCount":3,"files":["..."],"imageCount":2}
```

`imageCount`는 Markdown export에서 이미지가 저장된 경우에만 포함한다.

실패:

```json
{"ok":false,"error":"..."}
```

FFI 경계에서 panic은 `catch_unwind`로 잡아 실패 JSON으로 반환한다. 파일 읽기, HWP 파싱, 페이지 범위, 출력 폴더 생성, 파일 저장 실패도 모두 실패 JSON으로 정규화한다.

## 3. Export 동작

### 3.1 `rhwp_export_text`

1. 입력 HWP 파일을 byte로 읽는다.
2. `HwpDocument::from_bytes`로 문서를 파싱한다.
3. 대상 페이지 목록을 계산한다.
4. 각 페이지를 `extract_page_text_native`로 추출한다.
5. 출력 폴더에 `.txt` 파일을 저장한다.
6. 저장된 파일 목록을 JSON으로 반환한다.

파일명 규칙:

- 단일 페이지 문서: `{원본파일명}.txt`
- 다중 페이지 문서: `{원본파일명}_{page:03}.txt`

### 3.2 `rhwp_export_markdown`

1. 입력 HWP 파일을 byte로 읽는다.
2. `HwpDocument::from_bytes`로 문서를 파싱한다.
3. 대상 페이지 목록을 계산한다.
4. 각 페이지를 `extract_page_markdown_with_images_native`로 추출한다.
5. 이미지 토큰 `[[RHWP_IMAGE:N]]`을 실제 Markdown 이미지 링크로 치환한다.
6. 이미지가 있으면 `{원본파일명}_assets/` 아래에 저장한다.
7. 출력 폴더에 `.md` 파일을 저장한다.
8. 저장된 Markdown 파일 목록과 이미지 수를 JSON으로 반환한다.

이미지 추출은 control 위치 기반 조회를 우선 사용하고, 실패 시 `bin_data_id` 기반 fallback을 사용한다.

## 4. C# Binding 설계

`bindings/csharp/RhwpNative.cs`는 C# 소비자가 네이티브 ABI를 직접 다루지 않도록 얇은 wrapper를 제공한다.

```csharp
public static string ExportText(string inputPath, string outputDirectory, int page = AllPages)
public static string ExportMarkdown(string inputPath, string outputDirectory, int page = AllPages)
```

구현 정책:

- `NativeLibraryName = "rhwp_native_ffi"`로 공통 네이티브 라이브러리를 로드한다.
- C# 문자열은 UTF-8 null-terminated byte array로 변환한다.
- native 반환 포인터가 null이면 예외 처리한다.
- `Marshal.PtrToStringUTF8`로 JSON 문자열을 복원한다.
- `finally`에서 항상 `rhwp_string_free`를 호출해 native 메모리를 해제한다.

## 5. 단계 분리

### Stage 1 — bindings 구조 확정

**작업**

1. `bindings/README.md` 추가
2. `bindings/Native/` 공통 네이티브 ABI 폴더 생성
3. `bindings/csharp/` 언어별 구현 폴더 생성
4. 기존 `src/lib.rs` / WASM API 구조는 변경하지 않음

**산출물**

- `bindings/README.md`
- `bindings/Native/.gitignore`
- `bindings/Native/Cargo.toml`

### Stage 2 — Native FFI 구현

**작업**

1. `rhwp_export_text` 구현
2. `rhwp_export_markdown` 구현
3. `rhwp_string_free` 구현
4. UTF-8 C string 입력 검증
5. 성공/실패 JSON 반환 정규화
6. Markdown 이미지 저장 및 링크 치환

**산출물**

- `bindings/Native/src/lib.rs`

### Stage 3 — C# P/Invoke wrapper 구현

**작업**

1. `RhwpNative.ExportText` 추가
2. `RhwpNative.ExportMarkdown` 추가
3. UTF-8 null-terminated 변환
4. native 반환 문자열 수명 관리

**산출물**

- `bindings/csharp/RhwpNative.cs`

### Stage 4 — 빌드 검증 및 문서화

**작업**

1. Native crate `cargo check`
2. Native crate `cargo build`
3. 생성 라이브러리명 확인
4. bindings 구조 문서화

**산출물**

- `mydocs/working/task_m100_403_stage1.md`
- `mydocs/working/task_m100_403_stage2.md`
- `mydocs/report/task_m100_403_report.md`

## 6. 검증 결과

현재 git 변경 목록 기준으로 다음 검증을 수행했다.

```bash
cd bindings/Native
cargo check
cargo build
```

결과:

| 검증 | 결과 |
|------|------|
| `cargo check` | 통과 |
| `cargo build` | 통과 |
| Windows debug 산출물 | `bindings/Native/target/debug/rhwp_native_ffi.dll` 생성 확인 |

`target/`은 `bindings/Native/.gitignore`로 제외한다. `Cargo.lock`은 루트 `.gitignore`의 `Cargo.lock` 규칙으로 추적 대상에서 제외된다.

## 7. 위험 및 완화

| 위험 | 완화 |
|------|------|
| FFI 문자열 수명 오류 | 반환 문자열은 `CString::into_raw`, 해제는 `rhwp_string_free`로 단일화 |
| 호출자 메모리 해제 누락 | C# wrapper에서 `finally`로 항상 해제 |
| panic이 FFI 경계를 넘어감 | `ffi_result`에서 `catch_unwind` 후 실패 JSON 반환 |
| UTF-8 경로 처리 오류 | 입력 C string을 `CStr::to_str`로 검증하고 오류 JSON 반환 |
| 언어별 binding과 native 구현 결합 | `bindings/Native`와 `bindings/csharp`를 sibling 구조로 분리 |

## 8. 산출물 경로 정정

원 수행계획서의 산출물 항목에는 `task_m100_402_*`, `src/bindings/...` 경로가 일부 남아 있다. 본 구현에서는 Task #403 기준으로 다음 경로를 사용한다.

| 구분 | 경로 |
|------|------|
| 구현 계획서 | `mydocs/plans/task_m100_403_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_403_stage{N}.md` |
| 최종 보고서 | `mydocs/report/task_m100_403_report.md` |
| Native FFI | `bindings/Native/src/lib.rs` |
| C# binding | `bindings/csharp/RhwpNative.cs` |

## 9. 완료 및 후속

완료된 항목:

1. C# 실제 호출 샘플 또는 테스트 앱에서 `ExportText`, `ExportMarkdown` 호출 확인
2. JSON 반환과 입력 받은 출력 폴더의 파일 생성 확인
3. 단계별 보고서 및 최종 보고서 작성

후속 후보:

1. 반환 JSON schema를 C# typed result로 감쌀지 결정
2. platform별 native library 배포 규칙 정의
