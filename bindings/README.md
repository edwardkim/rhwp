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

### 전용 worker에서 PDF 내보내기

`workerSubmit`의 `exportPdf`는 열려 있는 문서의 저장 전 편집을 포함해 지정한 revision을
Print profile로 출력한다. `target`과 `fontGeneration`을 전달하고 `firstPage`는 0부터,
`pageCount`는 1부터 32까지 지정한다. `maxOutputBytes`는 1부터 32 MiB까지 지정한다.
요청을 받으면 이 크기를 worker의 `maxPendingBytes`에서 예약하며, 완료를 poll하면 예약을
반납한다. 활성 IME 조합이 있으면 `unsupported`를 반환한다.

성공 완료의 `kind`는 `exportedPdf`이며 `target`, `fontGeneration`, `firstPage`,
`pageCount`, `buffer`, `byteLength`를 포함한다. 호스트는 `sessionBufferCopy/Release`로
PDF를 가져와 미리보기나 파일 출력에 사용한다. 미리보기를 현재 문서에 표시할 때는 완료의
target 및 font generation을 현재 값과 비교한다. Worker는 출력 파일을 만들거나 문서의
저장 상태를 변경하지 않는다. 대기 중 요청만 취소할 수 있으며 실행을 시작한 요청은
`tooLate`를 반환한다. Worker를 닫으면 아직 받지 않은 완료를 폐기한다.

`livePdfExport`가 활성화된 `native-skia` 빌드에서 실행할 수 있다. 상한은 capabilities의
`maxLivePdfPages`와 `maxLivePdfBytes`로 조회한다.

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
동기 요청의 panic을 포착하면 해당 스레드의 문서·버퍼 handle을 무효화한다.
worker는 종료를 요청하되 handle을 보존하므로 workerStatus.isFinished 확인 후 workerRelease로
해제해야 공유 library를 안전하게 unload할 수 있다. 잘못된 주소를 C 포인터로 넘기는 것은
호출자 계약 위반이다. 입력 메모리는 호출 동안 유효해야 하며 포인터를 보관하지 않는다.

Swift는 `Rhwp.session*`, C#은 `RhwpNative.Session*` 메서드를 제공한다. 외부 실행 예산·파일 I/O·
앱 event loop는 host가 소유한다. native 실제 수명 검사는 빌드한 library를 지정해 실행한다.

```sh
python3 bindings/Native/tests/session_contract.py /path/to/librhwp_native_ffi.so --native-skia enabled
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
snapshot에만 기록한다. 셀에 focus가 있으면 현재 셀 내용과 기존 HWP 커서 메타데이터를 저장하며,
`caretPolicy: "storedCore"`와 `caretSaved: false`를 반환한다. 편집기의 라이브 선택은 유지된다.
HWPX는 기존 serializer의 source settings.xml 또는 기본값 정책을 유지한다.
`caretSaved`, `caretPreservedFromSource`, `caretPolicy`로
실제 경로를 확인한다. caretPreservedFromSource는 원본 settings.xml 바이트가 있을 때만 true이며,
그 안의 CaretPosition 존재나 현재 focus 저장까지 증명하지 않는다. 저장 확인은 편집 history를 지우지 않으며 undo 후에는 revision 기준 dirty다.

## Native worker 검색과 표 접근성

`accessibilityCellLayout`을 제공하는 호스트에서는 접근성 요청에 `includeCellLayout: true`를
넣을 수 있습니다. 표 셀 문단의 `cellLayout`은 해당 셀을 직접 담고 있는 표의 `row`, `column`,
`rowSpan`, `columnSpan`, `rowCount`, `columnCount`를 포함합니다. 행과 열은 0부터 셉니다.
요청을 생략하면 기존 노드 응답을 그대로 반환합니다.

`workerTextSearch`가 활성화된 worker는
`searchText{target,request:{query,caseSensitive,maxMatches,range?}}`를 지원한다.
검색어는 1~256 UTF-8 바이트이며 결과 수는 1~128개다. `range`는 접근성 조회와 같은
`{section,startParagraph,endParagraph}` 형식이며 끝 문단은 범위에 포함되지 않는다.

응답은 조회한 `target`과 `result.matches[]`의 `selection`, `editable`을 반환한다.
본문과 표 셀의 문자 범위를 문서 순서대로 돌려주며, 문자 오프셋은 Unicode scalar 기준이다.
보호된 셀은 선택할 수 있고 `editable: false`로 표시된다. 출력·순회·텍스트 한도에 도달하면
`truncated`가 설정된다. 검색 범위 일부에 선택으로 표현할 수 없는 개체가 있으면
`omittedContent`로 알려준다. 검색은 문서 버전이나 라이브 선택을 바꾸지 않는다.

찾은 범위로 이동할 때는 `setSelection`을 사용한다. 이 범위만 바꿀 때는 같은 `target`과
`selection`으로 `replaceRange{target,selection,text}`를 보낸다. 편집 세션에서 한 번의
편집으로 기록되며 실행 취소는 치환 전 라이브 선택을 복원한다. 이전 revision이나 종료되지
않은 입력기 조합 상태에서 요청하면 오류가 반환된다.

## Reference host (P57)

[Python reference host](Native/examples/README.md)는 표준 라이브러리만으로
open → region render → hit/caret → 한글 IME → undo/redo → save → reopen를 실행한다.
기존 입력을 지정하면 편집 데모를 적용하지 않고 읽기·조회·새 파일 저장만 수행한다.
이는 headless 경로이며 실제 window/event loop, OS IME와 각 플랫폼 packaging 검증은 별도다.
기능을 끈 library의 ABI 검사는 위 session_contract 명령에 `--native-skia disabled`를 사용한다.

## 다중 페이지 뷰 상태 (P58)

`native_host::view::NativeView`는 페이지 논리 크기와 viewport, zoom, deviceScale, pageGap을
받아 연속 페이지 배치·스크롤·가시 영역·좌표 변환을 계산한다. 페이지 좌표는 엔진의 96-DPI
논리 단위이며 viewport/scroll/gap은 창 논리 단위다. 실제 raster 배율은 zoom × deviceScale이다.
포인터를 기준으로 zoom을 바꾸고 caret가 보이도록 최소한 스크롤한다. 범위를 벗어난 scroll은
문서 범위로 제한하며 페이지 사이 빈 공간은 hit 대상이 아니다.

C `rhwp_native_host_request(json, length, data, dataLength)`, Swift `Rhwp.hostRequest`,
C# `RhwpNative.HostRequest`, Python `NativeHost.host_request`로 같은 계산을 사용한다.
P58의 JSON은 `{op:"view",state:{config:{pages:[{width,height}],viewport:{width,height},zoom,
deviceScale,pageGap},scroll:{x,y}},command:{action:"state"}}`다. command action은 configure,
scroll/scrollBy{x,y}, zoomAt{zoom,x,y}, pageAt{x,y}, pointInViewport{page,x,y},
ensureCaret{page,rect:{x,y,width,height},margin}도 지원한다. 이 연산은 binary data를 받지 않는다.

응답은 검증된 state, contentSize, scrollLimit, visible, rasterScale과 선택적 query 결과다.
visible의 pageRegion은 P52 입력 영역이고 pageRect는 페이지 전체의 창 논리 위치다.
RGBA 버퍼의 표시 원점은 pageRect.xy + originXY/deviceScale로 계산한다. 최대 10,000페이지,
논리 extent 2^40, raster 배율 (0,64]로 제한하며 조판/renderer의 페이지 배치는 바꾸지 않는다.

## 전용 문서 worker (P59)

`native_host::runtime::NativeRuntime`은 문서를 전용 스레드 안에서 열고 유지한다. !Send인
core/renderer를 스레드 사이로 이동하지 않는다. 요청과 읽지 않은 완료를 합쳐 기본4개/최대16개,
보관하는 입력·출력 bytes는 기본128 MiB로 제한한다. 문서 IR·history·Skia 내부 작업 메모리까지의
전체 RSS 상한은 아니다. 모든 편집/저장은 FIFO이며 render만 최신으로 병합한다. 거절된 submit은
기존 render를 무효화하지 않고 Rust 호출자에게 command 소유권을 돌려준다.

호스트 JSON 요청은 workerCreate{config}, workerSubmit{worker,command},
workerOpen{worker,replace,password}, workerPoll{worker}, workerCancel{worker,requestId},
workerClose{worker}, workerStatus{worker}, workerRelease{worker}다. workerOpen만 별도 binary data
span으로 문서 바이트를 받는다. worker handle은 생성한 OS 스레드의 소유다.
command는 `{op:"new",replace:null}`, `{op:"info"}`, target을 받는 renderRegion/pageSize/caretRect/
hitTest/selectionRects/beginEdit/setSelection/replace/IME/undo/redo/prepareSave와 documentId/token을
받는 acknowledgeSave/cancelSave다. target은 `{documentId,revision}`으로 재열기의 revision0을
이전 문서와 구별한다. render에는 fontGeneration도 전달한다.

submit은 requestId를 반환하고 poll은 아직 완료가 없으면 completion:null이다. 완료는
`{requestId,ok,kind,...}`이며 operation 오류도 같은 ID로 전달한다. frame/preparedSave의 bytes는
기존 sessionBufferCopy/Release로 소비한다. C ABI 보관 buffer 한도를 넘으면 해당 완료를 오류로
돌려주고 준비된 저장은 CancelSave를 enqueue하여 saveCleanup에 cleanup requestId를 제공한다.
이 cleanup 완료도 poll로 회수한다. 저장 report와 token은 준비된 바로 그 bytes에 대응한다.

cancel은 worker가 아직 요청을 가져가지 않았을 때만 성공한다. 실행 중 Skia/파서 호출을 강제
중단하지 않는다. close는 비차단 종료 신호이며 workerStatus.isFinished 뒤 workerRelease로
수명을 끝낸다. 그 전에 공유 library를 동적으로 unload하면 안 된다. 이미 받은 buffer도 별도로
해제한다. [Python NativeWorker](Native/examples/native_worker.py)는 GUI event loop용 submit/poll과
CLI/test 전용 wait/wait_closed를 구분한다. 실제 OS별 thread 종료·artifact 실행은 별도 검증이다.

## 기본 본문 명령과 IME 문맥 (P60)

`executeCommand`는 expectedRevision과 `{command:"moveLeft",extendSelection:false}` 같은
명령을 받는다. worker에서는 expectedRevision 대신 target을 전달한다. moveRight/Home/End/
moveUp/moveDown, deleteBackward/deleteForward, splitParagraph/joinPreviousParagraph/selectAll을
지원한다. 주소는 scalar지만 collapsed caret의 이동·삭제는 extended grapheme 경계를 따른다.
Home/End는 현재 문단 경계, selectAll은 현재 본문 구역이다. 좌우는 논리 문자열 순서다.
`tableCellParagraphCommands`가 활성화되어 있으면 표 셀에서도 `splitParagraph`와
`joinPreviousParagraph`를 사용할 수 있다. 중첩 표에서는 가장 안쪽 셀 안에서 처리한다.

문단 split/join과 선택 삭제는 한 revision/한 undo 단위이며 실패하면 문서·선택·history를 복원한다.
IME 조합 중 일반 명령은 commit/cancel을 먼저 요구한다. vertical 응답의 preferredX를 다음
vertical 입력에 전달하고 다른 명령 뒤에는 초기화한다.

셀 안에서 `splitParagraph`는 한 문단에 속한 선택을 지운 뒤 그 자리를 나누고 새 문단의
시작에 커서를 둔다. `joinPreviousParagraph`는 커서가 속한 문단을 같은 셀의 바로 앞 문단과
합친 뒤 접합점에 커서를 둔다. 셀의 첫 문단에서 합치기를 요청하면 현재 선택을 반환한다.
편집 대상은 보호되지 않은 셀 안에 있으며, 컨트롤이나 필드 경계가 없는 문단이어야 한다.

`tableCellNavigation` capability를 지원하면 Tab과 Shift+Tab을 각각
`{command:"navigateTableCell",direction:"next"}`와
`{command:"navigateTableCell",direction:"previous"}`로 전달할 수 있다.
현재 위치가 속한 가장 안쪽 표에서 실제 셀의 시작 행과 열을 기준으로 이동한다.
보호된 셀을 건너뛰고, 컨트롤이 없는 문단이 있는 셀로 이동한다.
해당 셀에서 조건을 만족하는 첫 문단의 시작에 커서를 둔다.
표 경계에서는 현재 선택을 반환한다. 이동 결과는 기존 문서 revision과 함께 반환한다.

`imeContext`는 조합 중에도 읽을 수 있는 최대4096 scalar 주변 문맥과 scalar cursor/anchor,
선택 텍스트 및 truncation 정보를 반환한다. worker는 target, 동기 세션은 expectedRevision을
받는다. OS API가 UTF-16을 쓰면 adapter에서 변환하고 surrogate 중간으로 커서를 만들지 않는다.
이 조회 자체는 문서·선택·revision을 바꾸지 않는다.


### Native 텍스트 서식

호스트 기능 목록의 `basicTextFormatting`으로 지원을 확인할 수 있다.
`executeCommand`에서 `{command:"formatCharacters",bold:true,fontSize:1500}`은 선택한 글자의 서식을
바꾼다. `bold`와 `fontSize`는 각각 생략할 수 있으며 하나 이상 필요하다. 크기는
HWPUNIT 단위(1pt = 100 HWPUNIT, 범위 1–409600)다.
`{command:"setParagraphAlignment",alignment:"right"}`는 선택 또는 캐럿이 있는 문단을 정렬한다.
정렬값은 `justify`, `left`, `right`, `center`, `distribute`, `split`이다.

두 명령은 본문 또는 중첩 표의 단일 텍스트 문단을 대상으로 한다. 글자 서식은 비어 있지 않은
선택이 필요하다. 보호된 셀과 편집할 수 없는 컨트롤이 있는 문단은 거부한다. 변경 시 선택을
유지하면서 revision과 undo를 한 번 갱신하며, 현재 서식과 같은 값은 history를 유지한다.

### Native clipboard (P61)

동기 session의 `copySelection`/`cutSelection`/`pasteText`/`pasteHtml`은 `expectedRevision`을 필수로 받습니다. worker에서는 같은 이름에 `target`을 전달합니다. 복사/잘라내기 응답의 `payload`는 `{plainText, html}`이며 `outcome`은 복사 시 null, 잘라내기 시 편집 결과입니다. 붙여넣기는 기존 편집 결과를 반환합니다. OS clipboard 소유권은 host가 관리합니다.

본문은 section 안의 텍스트를 지원하며 출력 payload와 입력은 1 MiB, 한 번의 붙여넣기는 256 문단으로 제한합니다. CRLF/CR을 LF로 정규화하며 하나의 undo로 복원합니다. HTML은 기존 core importer/exporter를 사용하고 object/control/table의 미지원 결과는 rollback합니다. HTML parser의 peak RSS까지 제한하는 계약은 아닙니다. 빈 문단을 HTML exporter가 생략하는 선택은 정확한 plain text와 `html:null`을 반환합니다.

capabilities의 `tableCellTextClipboard`가 `true`면 표 셀은 중첩 경로를 포함해 같은 셀의 한 문단에서 텍스트를 복사하거나 잘라낼 수 있으며 `html:null`을 반환합니다. `pasteText`는 한 줄을 지원합니다. 셀에서 여러 문단이나 HTML을 붙여넣으면 `unsupported`를 반환합니다. 보호된 셀의 텍스트는 읽을 수 있으며 잘라내기와 붙여넣기는 편집 권한을 확인합니다.

P61의 텍스트 편집·clipboard는 전체 대상 문단에 floating object나 필드/책갈피 등 semantic control이 있으면 명시적으로 거절합니다(구조용 SectionDef/ColumnDef는 예외). 문단 합치기와 IME에도 같은 guard를 적용하며 선택 조회와 동일 텍스트 no-op는 보존합니다. HTML은 lexical preflight로 객체 태그를 거절하고 주석을 제거한 뒤 기존 importer에 전달합니다. 일반 HTML sanitizer나 객체/필드 복사 기능은 아닙니다.


### Native 파일 저장·복구 예제 (P62)

`Native/examples/native_file_store.py`의 `NativeFileStore`는 기존 동기 session ABI를 사용하는 host adapter입니다. `open(host, path)`가 읽은 파일 stamp를 기억하고, `save()`는 같은 경로의 외부 변경을 전후 확인한 뒤 같은 파일시스템의 private 임시 파일을 flush/fsync·교체하고 checkpoint를 ack합니다. `save(new_path)`는 이미 있는 다른 파일을 덮어쓰지 않습니다. 손실 보고가 있으면 `accept_loss=True`를 명시해야 합니다.

`autosave(directory)`는 revision/시간 간격을 확인하여 별도 recovery artifact를 쓰고 save token은 취소하므로 원본과 dirty를 유지합니다. `read_recovery(path)`로 길이·버전·digest를 검증한 바이트를 읽어 별도 문서로 열 수 있습니다. 문서 128 MiB, recovery JSON header 64 KiB로 제한합니다. recovery 삭제·목록·사용자 확인 정책은 앱에서 결정합니다.

`PublicationError`의 `published=True`와 `path`는 완성된 파일을 게시한 뒤 directory flush/정리/ack 등이 실패했음을 뜻합니다. 게시된 파일을 삭제하거나 저장 성공으로 표시하지 않고 확인해야 합니다. 외부 변경 검사는 다른 프로세스와의 원자적 compare-and-swap이 아니므로 앱 내부 저장을 직렬화해야 하며 Windows directory durability와 네트워크 파일시스템 보장은 별도 검증 대상입니다.


### Native 접근성 snapshot (P64)

session의 `accessibility` 요청은 `request:{}`를 기본으로 본문·표 셀 문단의 구조 순서 텍스트와 scalar 주소를 반환합니다. `selection`을 명시하면 조회에만 사용하고, 생략/null이면 editor의 현재 선택을 사용합니다. worker는 `accessibility {target,request}`를 받아 현재 선택의 snapshot을 `kind:"accessibility"`로 반환하며 revision을 검사합니다. capability는 `accessibilitySnapshots:true`입니다.

요청은 최대 1000 nodes/UTF-8 text 1 MiB/깊이 32/방문 10000개로 제한합니다. `geometry:true`는 기존 core caret/selection query를 사용하며 최대 128 query/256 selection rect와 개별 문단 64 KiB 등 추가 상한을 적용합니다. `complete`, `truncated`, `limitations`를 확인해야 합니다. 노드 key는 revision+주소로 구성되며 revision 사이 identity를 보장하지 않습니다. worker는 완료를 읽을 때까지 snapshot에 8 MiB byte credit을 예약하며 이는 layout/Skia 전체 RSS 상한이 아닙니다.

`native_scalar_to_utf16`/`native_utf16_to_scalar`는 OS adapter offset 변환을 지원하고 surrogate 내부 주소는 거절합니다. 이 API는 OS 접근성 등록이나 screen reader 지원의 증거가 아닙니다. header/footer·notes·shape text·읽기 순서 추론과 OS adapter 연결은 후속입니다.


### Native Qt 창 예제 (P63)

[Qt 실행 안내](Native/examples/QT_NATIVE_HOST.md)는 선택 설치하는 PySide6 창과 기존 NativeWorker·Skia RGBA 경로를 연결합니다. Linux에서 실제 QWidget offscreen 렌더·한글 QInputMethodEvent·키보드 이동·undo/redo·새 파일 저장/재열기를 확인했습니다. 실제 OS IME 후보창·compositor와 macOS/Windows 실행은 별도 검증 대상입니다.
