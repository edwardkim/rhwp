# Task #403 최종 보고서

**제목**: AI 워크로드를 위한 FFI 인터페이스 개발  
**마일스톤**: M100 (v1.0.0)  
**브랜치**: `local/ffi-csharp`  
**이슈**: https://github.com/edwardkim/rhwp/issues/403

---

## 1. 완료 요약

`export-text`, `export-markdown` 기능을 외부 프로세스 실행 없이 호출할 수 있도록 `bindings` 하위에 Native FFI와 C# binding을 추가했다.

기존 WASM/API 구조는 유지하고, AI/서버 워크로드용 네이티브 ABI를 별도 crate로 분리했다.

## 2. 최종 구조

| 경로 | 역할 |
|------|------|
| `bindings/README.md` | bindings 구조 문서 |
| `bindings/Native/` | Rust `cdylib` 공통 Native ABI |
| `bindings/Native/src/lib.rs` | `export-text`, `export-markdown` FFI 구현 |
| `bindings/csharp/RhwpNative.cs` | C# P/Invoke wrapper |

`bindings/Native`는 언어별 폴더가 아니라 모든 언어 binding이 공유하는 공통 ABI 계층이다. C#은 첫 번째 언어별 binding으로 `bindings/csharp`에 배치했다.

## 3. Native ABI

공개 함수:

```rust
rhwp_export_text(input_path, output_dir, page) -> *mut c_char
rhwp_export_markdown(input_path, output_dir, page) -> *mut c_char
rhwp_string_free(ptr)
```

반환값은 UTF-8 JSON 문자열이다.

성공:

```json
{"ok":true,"pageCount":3,"files":["..."],"imageCount":2}
```

실패:

```json
{"ok":false,"error":"..."}
```

`page = -1`은 전체 페이지, `page >= 0`은 0-based 단일 페이지 export로 처리한다.

## 4. C# Binding

`RhwpNative`가 제공하는 공개 API:

```csharp
RhwpNative.ExportText(inputPath, outputDirectory, page)
RhwpNative.ExportMarkdown(inputPath, outputDirectory, page)
```

구현 사항:

- `rhwp_native_ffi` 라이브러리 로드
- UTF-8 null-terminated 문자열 전달
- native 반환 포인터를 UTF-8 managed string으로 변환
- `finally`에서 `rhwp_string_free` 호출
- null 입력과 invalid native 반환값 예외 처리

## 5. 검증 결과

수행 명령:

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
| Windows debug DLL | `bindings/Native/target/debug/rhwp_native_ffi.dll` 생성 확인 |
| C# DLL 로드 | 통과 |
| `RhwpNative.ExportText()` 실제 호출 | 통과 |
| `RhwpNative.ExportMarkdown()` 실제 호출 | 통과 |
| 반환 JSON 수신 | 통과 |
| 입력 받은 출력 폴더에 export 파일 생성 | 통과 |

C# end-to-end 호출 검증은 작업지시자 검증 결과를 반영했다.

## 6. 산출물

| 파일 | 종류 |
|------|------|
| `mydocs/plans/task_m100_403.md` | 수행 계획서 |
| `mydocs/plans/task_m100_403_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_403_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_403_stage2.md` | Stage 2 보고서 |
| `mydocs/working/task_m100_403_stage3.md` | Stage 3 보고서 |
| `mydocs/working/task_m100_403_stage4.md` | Stage 4 보고서 |
| `mydocs/report/task_m100_403_report.md` | 최종 보고서 |
| `bindings/README.md` | bindings 구조 문서 |
| `bindings/Native/Cargo.toml` | Native FFI crate |
| `bindings/Native/src/lib.rs` | Native FFI 구현 |
| `bindings/csharp/RhwpNative.cs` | C# binding |

## 7. 위험 및 대응

| 위험 | 대응 |
|------|------|
| native 반환 문자열 누수 | C# wrapper에서 `finally`로 `rhwp_string_free` 호출 |
| panic이 FFI 경계를 넘어감 | native `ffi_result`에서 `catch_unwind` 적용 |
| UTF-8 경로 처리 실패 | C string을 `CStr::to_str`로 검증 후 실패 JSON 반환 |
| 언어별 binding과 native 구현 결합 | `bindings/Native`와 `bindings/csharp`를 sibling 구조로 분리 |

## 8. 후속 사항

실제 C# 앱을 통한 end-to-end 호출 검증까지 완료되었다.

남은 후속 개선 후보:

1. 반환 JSON schema를 C# 쪽 typed result로 감쌀지 결정
2. 다른 언어 binding 추가 시 `bindings/{language}` sibling 구조 유지
3. 배포 패키지에서 platform별 native library 배치 규칙 정의

## 9. 결론

Task #403의 핵심 범위인 `export-text`, `export-markdown` Native FFI와 C# binding 구현은 완료되었다. Native crate 빌드, 산출물 이름 정합, C# 실제 호출까지 정상 동작을 확인했다.
