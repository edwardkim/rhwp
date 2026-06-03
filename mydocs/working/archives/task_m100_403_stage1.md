# Task #403 Stage 1 완료 보고서

**제목**: bindings 구조 확정  
**브랜치**: `local/ffi-csharp`  
**이슈**: https://github.com/edwardkim/rhwp/issues/403

---

## 1. 작업 요약

AI/서버 워크로드에서 console 프로세스 실행 없이 RHWP export 기능을 호출할 수 있도록 `bindings` 하위 구조를 분리했다.

기존 `src/lib.rs`와 WASM API는 변경하지 않고, FFI 전용 코드를 별도 디렉터리에 둔다.

## 2. 확정 구조

| 경로 | 역할 |
|------|------|
| `bindings/README.md` | bindings 구조와 확장 규칙 |
| `bindings/Native/` | 모든 언어 바인딩이 공유하는 Rust C ABI crate |
| `bindings/csharp/` | C# P/Invoke wrapper |

구조 원칙:

- `bindings/Native`는 언어별 구현이 아니라 공통 네이티브 ABI 계층이다.
- `bindings/csharp`는 C# 전용 wrapper만 포함한다.
- 이후 Python, Java, Node 등 언어별 구현은 `bindings/{language}` sibling 폴더로 추가한다.

## 3. 변경 파일

| 파일 | 내용 |
|------|------|
| `bindings/README.md` | `Native/`, `csharp/` 역할 문서화 |
| `bindings/Native/.gitignore` | native crate의 `target/` 제외 |
| `bindings/Native/Cargo.toml` | FFI crate 선언 |
| `bindings/csharp/RhwpNative.cs` | C# wrapper 위치 확정 |

## 4. 계획서 대비 정정

원 수행계획서에는 `src/bindings/...` 경로가 언급되어 있으나, 실제 구현은 독립 bindings 루트인 `bindings/...`를 사용한다.

정정된 산출물:

| 구분 | 경로 |
|------|------|
| Native FFI | `bindings/Native/src/lib.rs` |
| C# binding | `bindings/csharp/RhwpNative.cs` |

## 5. 결과

Stage 1 완료. bindings 디렉터리의 책임 경계가 확정되었고, 언어별 구현을 sibling 폴더로 확장할 수 있는 구조가 마련되었다.
