# Task #403 Stage 4 완료 보고서

**제목**: Native crate 빌드 검증  
**브랜치**: `local/ffi-csharp`  
**이슈**: https://github.com/edwardkim/rhwp/issues/403

---

## 1. 검증 명령

```bash
cd bindings/Native
cargo check
cargo build
```

## 2. 결과

| 검증 | 결과 |
|------|------|
| `cargo check` | 통과 |
| `cargo build` | 통과 |
| debug DLL 생성 | `bindings/Native/target/debug/rhwp_native_ffi.dll` 확인 |

Windows debug 산출물:

```text
rhwp_native_ffi.dll
rhwp_native_ffi.dll.lib
rhwp_native_ffi.dll.exp
rhwp_native_ffi.pdb
rhwp_native_ffi.d
```

## 3. 확인 사항

`bindings/Native/Cargo.toml`의 상대 경로 의존성:

```toml
rhwp_core = { package = "rhwp", path = "../.." }
```

새 위치 기준으로 루트 `rhwp` crate를 정상 참조한다.

라이브러리 이름:

```toml
[lib]
name = "rhwp_native_ffi"
crate-type = ["cdylib"]
```

C# wrapper의 `NativeLibraryName = "rhwp_native_ffi"`와 일치한다.

## 4. 추적 제외

| 대상 | 처리 |
|------|------|
| `bindings/Native/target/` | `bindings/Native/.gitignore`로 제외 |
| `bindings/Native/Cargo.lock` | 루트 `.gitignore`의 `Cargo.lock` 규칙으로 제외 |

## 5. Native 호출 검증

작업지시자 검증으로 네이티브 호출이 정상 동작함을 확인했다.

| 검증 | 결과 |
|------|------|
| C# 테스트 앱에서 DLL 로드 | 통과 |
| `RhwpNative.ExportText()` 실제 호출 | 통과 |
| `RhwpNative.ExportMarkdown()` 실제 호출 | 통과 |
| 반환 JSON 수신 | 통과 |
| 입력 받은 출력 폴더에 파일 생성 | 통과 |

Stage 4 완료. Native crate 빌드, 산출물 이름 정합, C# 경유 네이티브 호출까지 확인되었다.
