# Task #403 Stage 3 완료 보고서

**제목**: C# P/Invoke wrapper 구현  
**브랜치**: `local/ffi-csharp`  
**이슈**: https://github.com/edwardkim/rhwp/issues/403

---

## 1. 작업 요약

`bindings/csharp/RhwpNative.cs`에 C#에서 Native FFI를 호출하기 위한 얇은 wrapper를 추가했다.

## 2. 공개 API

```csharp
public static string ExportText(string inputPath, string outputDirectory, int page = AllPages)
public static string ExportMarkdown(string inputPath, string outputDirectory, int page = AllPages)
```

`AllPages = -1`로 정의해 native ABI의 전체 페이지 export 규칙과 맞췄다.

## 3. Native 연결

```csharp
private const string NativeLibraryName = "rhwp_native_ffi";
```

P/Invoke 선언:

```csharp
[DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
private static extern IntPtr rhwp_export_text(byte[] inputPath, byte[] outputDirectory, int page);

[DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
private static extern IntPtr rhwp_export_markdown(byte[] inputPath, byte[] outputDirectory, int page);

[DllImport(NativeLibraryName, CallingConvention = CallingConvention.Cdecl)]
private static extern void rhwp_string_free(IntPtr value);
```

## 4. 문자열 처리

C# 문자열은 `Encoding.UTF8.GetBytes()`로 UTF-8 byte array로 변환하고, 마지막에 NUL byte를 추가한다.

null 입력은 native 호출 전에 `ArgumentNullException`으로 차단한다.

## 5. 반환값 처리

`TakeResultString()`에서 native 반환 포인터를 다음 절차로 처리한다.

1. `IntPtr.Zero`이면 `InvalidOperationException`
2. `Marshal.PtrToStringUTF8()`로 managed string 변환
3. invalid UTF-8이면 `InvalidOperationException`
4. `finally`에서 `rhwp_string_free(result)` 호출

이 방식으로 C# 호출자가 native 메모리 수명을 직접 관리하지 않도록 했다.

## 6. 검증 결과

Stage 3에서는 wrapper 코드 작성과 native symbol 이름 정합을 확인했다.

이후 작업지시자 검증으로 네이티브 호출이 정상 동작함을 확인했다.

확인된 항목:

- C# 측 native library 로드
- `RhwpNative.ExportText()` 호출
- `RhwpNative.ExportMarkdown()` 호출
- 반환 JSON 수신
- 입력 받은 출력 폴더 위치에 export 파일 생성

Stage 3 완료.
