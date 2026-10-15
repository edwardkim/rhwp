# RHWP Bindings

This directory separates the shared native ABI from language-specific bindings.

- `Native/`: Rust `cdylib` crate that exposes the C ABI used by bindings.
- `csharp/`: C# P/Invoke wrapper over the shared native library.
- `swift/`: Swift Package wrapper over the shared native library.

New language bindings are downstream by default. Adding one here requires explicit maintainer
adoption, a stable compatibility contract, and a tested long-term release path.

## Direct PDF (P50)

`rhwp_export_pdf_direct(input_path, output_file, page)`는 기존 direct Skia PDF
renderer를 **print** profile로 호출한다. `page=-1`은 전체 쪽, 그 외에는 0부터 시작하는
쪽 번호다. 입력은 최대 **128 MiB**, 한 번에 선택하는 쪽은 최대 **256쪽**이다.
모든 선택 쪽의 검증과 PDF 생성을 마친 뒤 파일을 만들며, 부모 디렉터리는 이미 존재해야 하고
출력 파일은 새 경로여야 한다. 기존 파일·입력 파일·symlink·hard link를 덮어쓰지 않는다.
지원하지 않는 입력은 오류를 반환하고 compatibility SVG PDF로 우회하지 않는다.
성공 JSON의 `pageCount`는 원본 전체 쪽수, `files`는 생성한 PDF 한 파일이다.
반환한 문자열은 다른 C ABI와 동일하게 `rhwp_string_free`로 해제한다.

기본 빌드도 이 심볼을 내보낸다. null·UTF-8·쪽 번호 인자 검사를 통과하면 파일 I/O 없이
`{"ok":false,"code":"featureDisabled","error":"Direct PDF export requires the native-skia feature."}`를
반환한다. 기능을 사용하려면 공유 target 경로를 지정해 빌드한다.

```sh
CARGO_TARGET_DIR="$PWD/target/pr-review" cargo build --locked -p rhwp-native-ffi --features native-skia
```

C#은 `RhwpNative.ExportPdfDirect(inputPath, outputFile, page)`, Swift는
`Rhwp.exportPDFDirect(inputFile:outputFile:page:)`를 제공한다. C#은 JSON을 그대로 반환하고,
Swift는 기존 `RhwpExportResult`를 반환하거나 `RhwpError.exportFailed`를 던진다.
폰트는 기존 native renderer의 기본 font resolution을 사용한다. 이번 ABI는 별도 font path,
profile 또는 compatibility backend 선택 옵션을 노출하지 않는다.

Linux에서 native ABI를 검증해도 Apple XCFramework build/link가 검증된 것은 아니다.
Apple 산출물과 독립 한컴 PDF 출력 대조는 별도 검증 대상이다.

## 지속 문서 세션 (P51)

Rust의 `native_host::NativeDocumentSession`은 `DocumentCore`와 Skia renderer를 소유한다.
새 문서는 내장 템플릿으로 만들고, 기존 문서는 최대 128 MiB의 바이트에서 연다.
`document()`는 읽기 전용이다. 기본 빌드에서도 문서 수명·쪽 크기 조회를 사용할 수 있고,
화면 출력은 `native-skia`가 필요하다. 기본 화면은 CPU raw RGBA이며 PNG를 경유하지 않는다.

C의 `rhwp_session_capabilities/open/request/close`는 JSON 봉투를 반환한다.
반환 문자열은 항상 `rhwp_string_free`로 해제한다. `open`의 문서 길이가 0이면 새 문서다.
`request`는 길이가 명시된 UTF-8 JSON이며 다음 명령을 지원한다.

```json
{"op":"info"}
{"op":"pageSize","page":0}
{"op":"render","page":0,"scale":1.0}
```

쪽 번호는 0부터, 쪽 좌표는 기존 96-DPI layer 좌표다. scale은 `(0, 64]`이며 frame은
최대 16,777,216픽셀이다. `render`는 buffer handle, byteLength, width/height/stride,
revision, `rgba8Premultiplied`/`srgb` 정보를 반환한다. 각 행은 위에서 아래로 저장한다.
`rhwp_session_buffer_copy`로 caller의 바이트 배열에 복사하고
`rhwp_session_buffer_release`로 해제한다. 너무 작은 대상에는 아무것도 쓰지 않는다.
문서를 닫아도 이미 반환한 buffer는 유지되므로 별도로 해제해야 한다.

문서와 buffer handle은 **생성한 OS 스레드에서만** 사용한다. 다른 스레드·닫힌 handle은 오류다.
요청은 동기 호출이며 재진입을 지원하지 않는다. thread마다 문서 8개, 보관 buffer 64개/128 MiB,
요청 JSON 1 MiB로 제한한다. 이 제한은 파서·Skia의 전체 peak RSS 상한을 뜻하지 않는다.
panic을 포착하면 해당 스레드의 handle을 모두 무효화한다. 잘못된 주소를 C 포인터로 넘기는 것은
호출자 계약 위반이다. 입력 메모리는 호출 동안 유효해야 하며 포인터를 보관하지 않는다.

Swift는 `Rhwp.session*`, C#은 `RhwpNative.Session*` 메서드를 제공한다. 외부 실행 예산·파일 I/O·
앱 event loop는 host가 소유한다. native 실제 수명 검사는 빌드한 library를 지정해 실행한다.

```sh
python3 bindings/Native/tests/session_contract.py /path/to/librhwp_native_ffi.so
```

이 프로브의 Linux 통과는 Apple/Windows 패키지 실행이나 한컴 시각 정합 검증을 대신하지 않는다.

## 화면 영역 출력 (P52)

`{"op":"renderRegion","page":0,"x":20.5,"y":40,"width":320,"height":240,"scale":1.5}`는
전체 페이지와 동일한 원점의 제한된 scratch surface에 그린 뒤 해당 영역을 복사한다.
`scale`은 host의 zoom × device pixel ratio다. 정수 원점 이동도 Skia의 antialiasing을
바꿀 수 있으므로, 작은 surface에 평행 이동해 그리던 방식 대신 정확한 full-page crop을 사용한다.
왼쪽/위는 floor, 오른쪽/아래는 ceil로 전체 페이지 픽셀 격자에 맞추며 `originX/originY`를 반환한다.
버퍼 픽셀 `(px, py)`의 페이지 좌표는 `((originX+px)/scale, (originY+py)/scale)`다.
음수/페이지 밖 영역은 허용하고 원본 페이지 픽셀 경계 밖은 배경색으로 채운다.
좌표는 ±2²⁴픽셀, 출력 크기·픽셀 수는 기존 frame 한도로 제한한다.
출력 한도와 별개로 scratch는 축당 16,384픽셀·총 16,777,216픽셀 이하이고,
scratch N32와 출력 RGBA의 합계는 128 MiB 이하다. 초과하면 할당 전에
`limitExceeded`를 반환한다. 이는 두 bitmap의 상한이며 Skia 중간 surface·캐시·전체 RSS 상한은 아니다.
따라서 큰 페이지의 작은 영역도 scratch 상한 때문에 거절될 수 있다. 전체 페이지와 겹치지 않는
요청은 scratch 없이 배경색만 반환하지만, 먼저 geometry·출력 한도를 검증하고 출력 바이트에도
128 MiB 상한을 적용한다. 화면 밖 subtree의 replay 생략이나 GPU 렌더링은 하지 않는다.

## 폰트와 frame 유효성 (P53)

`setFonts`는 `paths`의 TTF/OTF/TTC 파일과 선택적 `environment` 선언을 함께 교체한다.
파일당 32 MiB, 입력 합계·보관 복사본 각각 64 MiB, 256파일/collection당 64 face로 제한한다.
검증한 바이트를 직접 renderer에 전달하고 실패하면 이전 설정을 보존한다. 경로가 달라도 같은
바이트·순서·선언이면 no-op이다. `fontState`는 내용 fingerprint와 공급 source metadata를 반환한다.
이는 시스템 fallback 설치·실제 글리프 coverage를 증명하지 않는다.

frame의 `revision`과 `fontGeneration`을 함께 보관하고 `isFrameCurrent`에 전달해 폐기 여부를 판단한다.
폰트 변경은 문서 revision을 올리지 않고 fontGeneration을 올린다. `clearCaches`는 보관한 Skia
리소스와 통계를 비우며 `cacheStats`는 picture/glyph/image별 사용량을 반환한다. 각 cache는
64항목/8 MiB 한도이며 전체 process 메모리 상한은 아니다.

## 입력 위치와 좌표 (P54)

위치는 `{section, paragraph, scalarOffset, cellPath}`다. section/paragraph는 0부터 시작하고
scalarOffset은 Unicode scalar 수이며 UTF-8 byte나 UTF-16 code unit 수가 아니다.
cellPath의 `{control, cell, paragraph}` 항목은 바깥 문단에서 안쪽 표 셀까지 차례로 내려간다.
선택은 `{anchor: position, focus: position}`이며 역방향도 보존한다.

`hitTest{page,x,y}`, `caret{position}`, `selectionRects{selection}`은 기존 core의 페이지 좌표를
반환한다. `moveVertical{position,direction:"up"|"down",preferredX}`는 다음 위치와 preferredX를
반환한다. 화면 위치는 P52 원점/scale 식으로 먼저 페이지 좌표로 변환한다. 유효한 위치만 있고
실제 caret geometry가 없으면 null이며, 임의의 (0,0) 사각형을 만들어내지 않는다.
본문·중첩 표 셀을 조회하며 textbox/caption/note/header/footer, inline control affinity와
서로 다른 container를 가로지르는 선택은 명시적으로 unsupported 오류를 반환한다.

## 본문 편집, IME와 undo (P55)

`beginEdit{selection,expectedRevision}`로 기존 viewer 세션을 편집 세션으로 전환한다.
지원하지 않는 위치로 전환에 실패해도 열린 문서는 유지된다. 편집 요청은 expectedRevision을
필수로 받고 오래된 요청을 거절한다. `setSelection`, `replaceSelection{text}`와 `undo`, `redo`,
`clearHistory`는 모두 같은 revision 계약을 사용한다. 반환 `outcome`에 다음 revision, selection,
compositionId, documentChanged/historyChanged가 있다.

`beginComposition` → `updateComposition{compositionId,text}` 반복 → `commitComposition` 또는
`cancelComposition` 순서로 호출한다. 각 요청에 최신 expectedRevision도 전달한다. preedit는
같은 원본 선택을 대체하고 commit은 한 undo 단위다. 취소된 과거 조합의 이벤트는 ID로 거절한다.
`focusLost{policy:"commit"|"cancel",expectedRevision}`로 앱의 focus 정책을 명시한다.
조합 중 선택 이동·일반 편집·undo·폰트 교체는 먼저 commit/cancel해야 한다.

이 단계는 본문 scalar 텍스트만 편집한다. 셀 편집·inline object affinity·줄/문단 삽입·탭은
별도 명령이 필요하며 거절한다. history는 기본 16작업의 snapshot pair이고 caller의 기존
snapshot을 지우지 않는다. snapshot 수 제한은 전체 메모리 상한이 아니다. core의 event log
정책은 유지하며 OS 키보드/IME 후보창은 앱이 caret geometry를 이용해 연결한다.

## 저장 checkpoint (P56)

`prepareSave{format:"hwp"|"hwpx"}`는 저장 bytes의 buffer/byteLength, 그 bytes에 해당하는
content-loss `report`, revision과 일회용 token을 반환한다. host가 보고를 확인하고 실제 파일
쓰기를 성공시킨 뒤에만 `acknowledgeSave{token}`을 호출한다. 실패·취소는 `cancelSave{token}`이다.
이 API는 파일 I/O를 하지 않으며 경로·동시 저장·원자적 파일 교체는 host 책임이다.

새 문서는 dirty이고 성공적으로 연 문서는 clean이다. 준비만으로 clean이 되지 않으며 저장 도중
새 편집이 생기면 옛 token을 성공 확인해도 dirty를 유지한다. 최신 성공한 prepare만 유효하며
취소/확인 후 token은 재사용할 수 없다. frame처럼 저장 buffer도 copy 후 별도로 release한다.
보관 한도로 buffer를 반환하지 못하면 새 checkpoint를 취소하며 이전 superseded token은 복원하지 않는다.

편집 세션은 활성 IME를 먼저 commit/cancel해야 저장할 수 있다. HWP는 현재 본문 focus를 export
snapshot에만 기록한다. HWPX는 기존 serializer의 source settings.xml 또는 기본값 정책을 유지하며
현재 focus 저장을 보장하지 않는다. `caretSaved`, `caretPreservedFromSource`, `caretPolicy`로
실제 경로를 확인한다. caretPreservedFromSource는 원본 settings.xml 바이트가 있을 때만 true이며,
그 안의 CaretPosition 존재나 현재 focus 저장까지 증명하지 않는다. 저장 확인은 편집 history를 지우지 않으며 undo 후에는 revision 기준 dirty다.
