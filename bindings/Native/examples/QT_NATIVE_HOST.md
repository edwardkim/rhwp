# Qt native 창 예제

`qt_native_host.py`는 PySide6 창에서 native Skia 문서를 표시하는 작은 참조 앱이다.
WASM·DOM·브라우저를 사용하지 않는다. Rust worker가 문서 core를 소유하며 GUI는
`QTimer`로 완료를 받는다. 입력은 최대 64개 action FIFO이고 문서 요청은 한 번에 하나만 보낸다.
뷰가 바뀌거나 action이 접수되면 실행 중 tiled render에 취소를 요청한다. P77의 타일 경계 취소를
사용하지만, 이미 실행 중인 조판·Skia 호출의 강제 중단을 보장하지 않는다.

```bash
python3 -m venv .venv-qt
.venv-qt/bin/python -m pip install -r bindings/Native/examples/qt-requirements.txt
.venv-qt/bin/python bindings/Native/examples/qt_native_host.py \
  --library /absolute/path/to/librhwp_native_ffi.so
```

`--input sample.hwpx`로 기존 문서를 열 수 있다. 입력 파일을 자동 수정하거나 저장하지 않는다.
`--renderer cpu`가 기본이며 기존 tiled RGBA 화면을 사용한다. `--renderer auto`는 사용 가능한
Native GL 빌드와 Qt context가 있으면 QOpenGLWidget FBO에 직접 그리고, GPU 미지원·context 생성 실패·
device 오류일 때 CPU로 전환한다. `--renderer gl`은 GL을 명시적으로 요구하며 실패를 표시한다.
`offscreen`/`minimal` Qt platform은 GL 창을 지원하지 않으므로 auto는 CPU, gl은 시작 오류다.
`--output new.hwpx`는 다른 이름으로 저장 대화상자의 기본 경로다. HWP/HWPX 저장을 지원하고,
일반 저장은 문서를 열 때 읽은 원본 bytes의 stamp를 사용하여 외부 변경을 확인한다. 파일 읽기와
원자적 파일 교체·flush는 I/O executor에서 수행한다. 심볼릭 링크와 일반 파일이 아닌 경로는 거절한다.
NativeWorkerFileStore의 prepare→동일 bytes/report 승인→파일 게시→ack 순서를 사용한다.
내용 손실 보고는 승인 전에 표시하며, 거절·실패 시 token 정리를 받은 뒤 작업 상태를 해제한다.
이미 게시한 파일은 ack 오류 때문에 삭제하지 않는다. 기존 다른 파일을 선택하면 그 파일을 읽어
stamp를 얻은 뒤 교체 승인을 받고, 게시 직전에 같은 stamp인지 다시 확인한다. `save_new` API는
계속 새 파일에만 쓰며, 원본을 포함하여 이미 있는 파일은 교체하지 않는다.

## 문서 교체와 종료

- 도구 막대의 새 문서·열기와 창 닫기는 이미 접수한 편집 명령 뒤에서 native `info`를 확인한다.
  변경 내용이 있으면 **저장 / 저장하지 않고 계속 / 취소**를 선택한다. 취소와 저장 대화상자
  취소는 현재 문서를 유지한다. Ctrl+N으로 새 문서를 요청할 수 있다.
- 저장 선택 시 파일 쓰기와 저장 token acknowledge를 마친 뒤 문서 상태를 다시 확인한다. 저장 실패,
  저장 완료 확인 실패 또는 저장 중 추가 편집으로 dirty가 남으면 문서 교체·종료를 보류한다.
  원본을 연 문서는 같은 경로에 안전하게 저장하며, 새 문서는 저장 위치를 선택한다.
- 일반적인 저장 중에는 본문을 계속 편집할 수 있다. 저장 중인 파일은 준비 당시 revision이며,
  이후 편집은 dirty로 남는다. 저장 작업 도중 새 문서·열기·닫기 요청은 저장 완료 후 다시 시도하도록 안내한다.
- 제목에는 파일명과 변경 표시(`*`)를, 상태 영역에는 dirty와 문서 작업 진행 여부를 표시한다.
  문서 전환 중 새 입력과 관련 action을 비활성화하고, 실행 취소·다시 실행은 완료된 native capability를 따른다.
- 파일 읽기에는 문서 교체 의도를 묶는다. 읽는 동안 더 새로운 열기 요청이 접수되면 늦게 끝난 이전 읽기를
  반영하지 않는다. 읽기 뒤 문서 revision이 달라지면 교체를 보류한다. 실행 중 native open 자체의 강제 취소를
  주장하지 않으며 이 단계에서는 추가 교체 요청을 받지 않는다.
- 접수된 GUI action은 문서 세대와 document ID를 보유한다. 교체 전 action의 지연 target 계산이나 callback을
  새 문서에 적용하지 않는다. 기존 문서에 이미 접수한 편집은 dirty 확인 이전에 완료한다.
- 일반 입력 큐의 64개 상한과 별도로 저장 ack/cancel용 한 슬롯을 예약한다. 입력이 가득 차도
  파일 게시 이후 checkpoint 또는 token 정리가 멈추지 않으며 편집 명령끼리의 순서는 유지한다.
- native 조합이 남아 있으면 확정·취소 후 문서 전환을 다시 요청하도록 안내한다.
  파일 I/O가 진행 중이면 worker를 먼저 닫지 않는다.

## 표준 명령과 드래그 선택

- 파일·편집·보기 메뉴와 도구 막대는 같은 `QAction`을 사용한다. `QKeySequence.StandardKey`의
  플랫폼별 전체 키 목록을 등록하고 `QKeyEvent.matches`로 처리하므로 Ctrl/Cmd 및 Shift+Insert 같은
  대체 키도 Qt 설정을 따른다. 저장과 다른 이름으로 저장은 별도 action이며, 안전 저장 adapter를
  연결한 호스트의 `save_document`가 있으면 저장 action이 해당 함수를 사용한다.
- AltGr/Ctrl+Alt의 문자는 `ShortcutOverride`부터 편집기로 전달한다. 조합 중 Undo 또는 Escape는
  현재 조합만 취소하고, 확대·축소는 허용한다. 다른 문서·편집 명령은 조합 확정·취소를 안내하며
  미확정 글자를 저장하거나 clipboard 작업에 섞지 않는다. 접수됐지만 아직 완료되지 않은 IME
  action도 이 판단에 포함한다.
- 메뉴·팝업으로 이동하는 focus 이벤트는 조합을 취소하지 않는다. 다른 위젯·앱으로 focus가
  이동하면 Qt 입력기를 reset하고 native 조합을 FIFO로 취소한다. 이 정책은 Qt adapter 수준의
  동작이며 실제 OS 입력기·후보창 검증을 대신하지 않는다.
- 드래그 포인터가 viewport 밖으로 나가면 40ms 타이머에서 축마다 회당 최대 32 logical pixel씩
  스크롤하고, native view가 제공한 가장 가까운 보이는 페이지를 다시 hit-test한다. 이전 hit가
  끝나기 전에는 다음 스크롤을 시작하지 않으며, GUI action이 4개 이상 대기하면 재시도한다.
  원래 64개 FIFO 상한을 유지하고 편집 명령을 버리거나 native 조판을 UI에서 실행하지 않는다.
- 드래그 release·focus 이동·문서/폰트/revision 교체·문서 전환·종료 시 타이머를 정리한다.
  release 직후 대기 중인 최초 hit도 anchor를 보존한 뒤 마지막 선택 지점을 적용한다.
- IME 문맥·캐럿·선택 geometry는 요청 당시의 문서 세대·revision·폰트·선택·조합 커서와
  완료 시점을 비교한다. 이전 상태의 결과는 후보창 좌표에 반영하지 않으며, 현재 페이지 좌표는
  최신 scroll/zoom 변환으로 표시한다. 주변 문자열 치환·재변환은 여전히 별도 Native API 범위다.

## 타일 화면과 캐시

- `native_viewport_tiles.py`가 Native Skia와 같은 512px 페이지 픽셀 격자로 보이는 영역만 계획한다.
  각 타일을 `renderTiledRegion`으로 요청하고, 마지막 타일은 확대된 페이지 끝의 올림 값을 포함한다.
  논리 좌표 나눗셈으로 출력에 경계 픽셀이 추가되더라도 실제 `originX/Y`·크기로 배치하고 타일과 페이지의
  교집합에 clip한다. 전체 페이지 exact-crop API와 동일한 antialiasing 결과를 주장하지 않는다.
- 한 뷰는 최대 64개 타일·64개 페이지다. 비정상 수치·정확한 픽셀 좌표 범위 초과와 초과 타일 수를
  격자 순회 전에 거절한다. UI의 기존 문서 최대 512쪽 제한도 유지한다.
- 화면 QImage는 80 MiB·80개 entry 상한의 LRU만 소유한다. 현재 화면 목록에는 image 참조를 보관하지 않는다.
  같은 문서 세대·document ID·revision·font generation·page·raster scale·DPR·타일 영역이면 스크롤 후 재사용한다.
  선택·캐럿·조합 장식은 별도 overlay이며 선택 이동만으로 타일 캐시를 지우지 않는다.
- 문서별 Native raster cache 상한은 `setRasterCacheLimit`으로 32 MiB를 지정한다. 한 뷰 렌더가 끝나면
  `rasterCacheStats`를 조회하고 상태 영역에 화면 image와 Native 보유 바이트를 각각 표시한다. 이 수치는
  문서 모델·폰트·리소스·Skia 중간 작업과 프로세스 전체 RSS를 포함하지 않는다.
- 새로운 뷰와 접수된 action은 이전 render request ID에 `workerCancel`을 보낸다. 실제 완료를 받을 때까지
  FIFO 슬롯을 유지하므로 편집 순서를 바꾸지 않는다. 취소·superseded skip은 오류로 표시하지 않는다.
  stale 문서·revision·font·view 결과는 CPU로 복사하지 않고 native buffer를 바로 해제한다.
- 확대율·DPR이 바뀐 화면에는 해당 scale의 타일이 필요하다. 보유된 같은 scale의 타일은 재사용하며,
  이 경로는 CPU RGBA 출력이다.

## GL 화면과 소유권

- `qt_gpu_view.py`는 P89 worker의 `exportRenderSnapshot`으로 얻은 불변 snapshot ID와 P90
  `native_gl.NativeGlPresenter`를 연결한다. `paintGL`은 문서 core·조판·RGBA buffer를 호출하지 않는다.
  worker가 새 revision/font를 반영하면 이전 snapshot을 해제하고 다시 export한다. 화면 이동·zoom·DPR·
  resize·FBO/context 교체는 별도 view/device generation을 올리며 같은 source snapshot을 재사용한다.
- NativeView의 visible page·pageRect·zoom·DPR을 사용해 물리 픽셀 위치와 scale을 정한다. 가시 페이지
  snapshot이 모두 준비된 뒤 한 frame을 replay한다. 최대 16쪽의 snapshot만 보유하며 화면 밖 페이지·
  문서 교체·종료·CPU 전환 및 늦은 completion의 snapshot을 `snapshotRelease`로 해제한다.
  export도 기존 FIFO와 한 개의 worker 요청 슬롯을 사용한다.
- Qt가 소유한 current OpenGL 3.3 context와 RGBA8·samples=0·stencil=0/8 FBO를 잠깐 빌린다.
  Qt는 swap/present를 담당한다. Native replay 전후 `QPainter.beginNativePainting/endNativePainting`을
  사용하며 Skia의 GL state reset 뒤 Qt로 제어를 돌려 캐럿·선택·IME overlay를 그린다.
  CPU와 GL은 QObject 기반이 없는 `CanvasInputMixin`을 공유한다. 접근성 adapter는 공통
  `rhwp_native_canvas` marker와 `window` 인터페이스를 사용할 수 있다.
- context의 `aboutToBeDestroyed`와 창 종료에서 makeCurrent 후 presenter를 release한다. current가
  되지 않으면 GL을 호출하지 않는 abandon을 사용한다. callback 객체와 Qt context는 native handle의
  수명 동안 wrapper에 보유한다. context 재생성은 새 device generation을 사용한다.
- GPU frame은 strict replay를 scratch surface에서 마친 뒤 Qt FBO에 게시한다. 문서 이미지 decode·
  font/text 계약·한도 오류는 진단하며 CPU의 placeholder 경로로 바꾸지 않는다. Native의 실패 시
  target 보존과 별개로, Qt는 성공한 현재 frame이 없으면 FBO 전체를 불투명 배경으로 가린다.
  문서·revision·font·view·target 세대가 바뀌거나 snapshot이 준비 중이면 이전 frame metadata를
  폐기하며, strict 실패·종료 상태도 같은 배경을 표시한다. `PartialUpdate`가 보존한 이전 문서·
  overlay 픽셀을 새 문서의 화면으로 노출하지 않는다. stale frame은 최신 source/view를 기다리는
  경로이며 device 오류와 구분한다.
- 정상 GL 경로에는 whole-frame CPU readback이 없다. Native `wholeFrameReadbacks` 계측으로 확인한다.
  테스트의 대기·실패 화면 검사와 선택적 `grabFramebuffer`는 검증용 readback이며 이 계측의 범위 밖이다. GL_VENDOR/RENDERER/
  VERSION을 함께 보고하고 llvmpipe 등은 software로 표시한다. 그 외 드라이버도 하드웨어 실행이라고
  자동 확정하지 않는다. 이 예제는 외부 texture export나 Flutter texture registration을 구현하지 않는다.

context 정리와 FBO 수명은 [QOpenGLWidget](https://doc.qt.io/qt-6/qopenglwidget.html),
state handoff는 [QPainter](https://doc.qt.io/qt-6/qpainter.html#beginNativePainting) 계약을 따른다.

## 문서 읽기 접근성

- `native_accessibility.py`는 문서 Canvas에 `QAccessibleWidget` + `QAccessibleTextInterface` factory를 등록한다.
  Qt 제목·도구 막대의 기본 접근성과 별개로 본문을 `QAccessible.Role.Document`의 text interface로 노출한다.
  PySide6 6.10.2에서 Python subclass·factory·Qt `textInterface()` 호출을 실제로 검사한다.
- 기존 worker `accessibility` 명령으로 최대 256개 node·64 KiB UTF-8 텍스트·깊이 8·geometry query 64개·선택
  rect 64개인 snapshot을 비동기로 받는다. 문서 세대·document ID·revision·font generation·선택에 묶고,
  늦은 응답을 폐기한다. 같은 revision의 선택 이동은 텍스트를 지우지 않고 선택만 알 수 없는 상태로 두었다가
  새 snapshot으로 갱신한다. 문서 변경·교체·종료에는 이전 텍스트와 geometry를 폐기한다.
- 본문과 표 셀 텍스트를 Native snapshot의 구조 순서로 연결하며 문단 사이에 명시적인 개행을 넣는다.
  Qt offset은 UTF-16 code unit, Native 주소는 Unicode scalar다. surrogate pair 중간 조회를 텍스트 조각이나
  Native 주소로 변환하지 않는다. bounded snapshot 밖의 cursor·선택은 `-1`로 알 수 없음을 표시한다.
  시각적 읽기 순서, 전체 문서 absolute offset, 표의 행·열 접근성 tree를 주장하지 않는다.
- `complete`·`truncated`·Native limitations는 문서 Description에, node role·text-complete·geometry status는
  text attributes에 보존한다. 읽기 전용 접근성 adapter이며 접근성 API의 편집·선택 변경·자동 스크롤은 지원하지 않는다.
  기존 키보드·IME 편집은 계속 Native editor를 사용한다.
- 완전한 단일 문자 선택의 snapshot geometry는 즉시 재사용한다. 나머지 `characterRect`는 실제 문자 rect가
  없으면 빈 QRect를 반환하고 기존 `selectionRects` 요청을 예약한다. 예약 8개·완료 cache 64개로 제한하며,
  완료 후 Qt `LocationChanged`를 알린다. 문단 start/end caret에서 임의 문자 폭을 추정하지 않는다.
  조회 시 native 호출 없이 완료된 NativeView의 page placement·zoom을 화면 좌표로 변환한다.
- Qt query는 native worker/파일 I/O를 동기로 호출하지 않는다. `offsetAtPoint`는 캐시된 문자 geometry만
  사용한다. Char boundary는 Qt grapheme finder, Paragraph boundary는 구조적 문단 경계를 사용한다.
  실제 조판 line·word·sentence 경계는 미지원으로 반환한다. PySide6 6.10.2의 기본 boundary method를
  Python에서 base-call할 때 생기는 out-parameter 바인딩 오류를 unsafe 포인터로 우회하지 않는다.
- Qt는 위젯 생성 도중 서로 다른 `QMetaObject`마다 접근성 interface를 캐시에 등록할 수 있다.
  adapter는 실제 metaobject별로 다른 interface를 만들고 같은 metaobject의 조회만 재사용한다.
  provider는 weak reference로 연결하며, Qt 캐시가 위젯 파괴 시 interface를 각각 한 번 해제한다.
  factory 반환을 별도로 `registerAccessibleInterface`로 중복 등록하거나 UI 객체를 전역 보관하지 않는다.
  실제 GL 검사에는 접근성 텍스트 조회, 세 번의 위젯 파괴·재생성, cache ID 무효화와 Python UI wrapper
  회수를 포함한다. 구현 근거는 [Qt 6.10.2 접근성 캐시](https://github.com/qt/qtbase/blob/v6.10.2/src/gui/accessible/qaccessiblecache.cpp)와
  [PySide 6.10.2 factory glue](https://github.com/pyside/pyside-setup/blob/v6.10.2/sources/pyside6/PySide6/glue/qtgui.cpp)다.
- 이 검사는 실제 Qt 접근성 interface/event API 연결의 근거다. 실물 OS의 accessibility bridge와
  screen reader qualification은 별도이며, offscreen 검사만으로 완료했다고 주장하지 않는다.

## 표시와 입력 범위

- 여러 쪽을 세로로 표시한다. 참조 UI는 최대 512쪽을 읽는다. P58 view 계약이 zoom·DPR·scroll·좌표
  변환을 담당한다. wheel은 스크롤, Ctrl+wheel은 포인터 기준 확대·축소다.
- 화면에 보이는 타일만 페이지별로 순서대로 렌더한다. 유효한 native frame은 복사·release 후
  `Format_RGBA8888_Premultiplied` QImage로 다시 소유한다. 이전 revision/font generation/view의
  결과는 표시하지 않는다. GUI에서 레이아웃을 재계산하지 않는다.
  페이지 크기 조회 도중 문서 revision이 바뀌면 이전 조회를 버리고 새 정보로 다시 시작한다.
  한 revision의 전체 페이지 크기를 받은 뒤 뷰에 한 번에 게시하며, 준비 중인 geometry를 IME·overlay에 사용하지 않는다.
  문서·폰트·선택 상태가 바뀌면 이전 캐럿·선택·조합 범위의 geometry를 폐기한다.
  캐럿 조회와 전체 페이지 뷰 갱신 뒤 native `ensureCaret`로 편집 위치를 화면 안에 드러냅니다.
  페이지 크기 조회 자체의 실패는 로딩 상태를 해제합니다. 다른 편집 명령의 실패가 진행 중인 페이지 조회를 취소하지 않습니다.
  문서 교체 시 이전 문서의 선택·캐럿·조합 장식과 이동 기준을 지운다.
- 페이지 크기 조회 중 click/drag hit test를 보류하며, 완성된 뷰의 좌표만 사용합니다.
  click/drag, selection/caret overlay, 좌우·상하·Home/End, Backspace/Delete, Enter, Ctrl+A/Z/Y,
  Ctrl+C/X/V를 연결한다. Ctrl+Alt 또는 AltGr의 출력 문자는 shortcut으로 소비하지 않고 본문에 입력합니다. Home/End는 문단 경계다. 셀·inline object 편집은 native API의 거절을 표시한다.
- Qt `QInputMethodEvent` preedit→update→commit을 native composition에 연결한다. 다른 위젯·앱으로
  focus가 이동하면 취소하며, 메뉴·팝업 focus는 위 표준 명령 정책을 따른다.
  surrogate pair 중간 offset과 주변 문자열 replacementStart/replacementLength는 안전하게 거절한다.
  주변 문맥·cursor·anchor는 마지막 완료된 native snapshot에서 읽고 scalar→UTF-16으로 바꾼다.
  bounded 문맥 밖의 anchor와 absolute position은 완전한 문서 위치라고 주장하지 않는다.
- preedit cursor와 밑줄 색을 지원하며 조합 범위에는 점선 밑줄을 표시한다. Qt IME의 임의 TextFormat
  글꼴/배경, 복잡한 clause별 장식, 주변 문자열 재변환은 아직 구현하지 않았다. OS별 실물 IME 검증은 별도다.
- 종료는 native shutdown 신호를 보낸 뒤 worker가 끝날 때까지 Qt event loop를 유지한다.
  진행 중인 파일 작업은 먼저 마쳐야 한다. 실행 중 작업을 취소 완료로 표시하지 않는다.

## 검사

```bash
python3 -B bindings/Native/tests/native_viewport_tiles_contract.py
python3 -B bindings/Native/tests/native_accessibility_contract.py
QT_QPA_PLATFORM=offscreen .venv-qt/bin/python -B bindings/Native/tests/native_window_smoke.py
QT_QPA_PLATFORM=offscreen .venv-qt/bin/python -B bindings/Native/tests/native_gpu_window_smoke.py
QT_QPA_PLATFORM=offscreen .venv-qt/bin/python -B bindings/Native/tests/native_window_smoke.py \
  --library /absolute/path/to/librhwp_native_ffi.so \
  --screenshot /new/path/qt-window.png
```

GL 실제 검사는 Native GL 기능을 포함한 라이브러리와 xcb/X 서버가 필요하다. headless 환경에서는
별도로 실행한 Xvfb의 display를 지정한다. 아래 명령은 synthetic controller 검사 뒤 실제 GL 창에서
snapshot 재사용·zoom·resize·편집과 readback 계측을 검사한다. 새 문서의 export 완료를 지연한 화면과
실제 손상 PNG의 strict 실패 화면도 배경으로 가려지는지 세 번의 검증용 readback으로 확인한다.
추가 `--diagnostic-readback`은 선택 사항이다.

```bash
DISPLAY=:99 QT_QPA_PLATFORM=xcb .venv-qt/bin/python -B \
  bindings/Native/tests/native_gpu_window_smoke.py --library /absolute/path/to/lib-with-native-gl.so
```

offscreen synthetic 검사의 성공만으로 실제 GPU·드라이버·물리 장치 성공을 주장하지 않는다.

첫 명령은 Qt 없이 실행하는 격자·분수 끝점·계획 상한·LRU 바이트/수명 계약 검사다.
Qt 명령은 UTF-16 경계, QImage 소유권, IME replacement 거절, 큐 상한의 adapter 검사다.
라이브러리를 지정하면 실제 blank→tile 표시→합성 Qt 조합 ㄱ/가/각→commit→좌우 키 이동→undo/redo→HWPX 새 파일
저장→재열기를 검사한다. 이 검사는 OS 한글 입력기·후보창, 한컴 시각 일치 또는 플랫폼 배포 패키지 검증을
대체하지 않는다. `--screenshot`은 선택 사항이며 실행 중인 QWidget을 새 PNG 파일로 남긴다.
실제 스모크에는 P74의 tiled worker/cache ABI와 P77의 협력 취소를 포함한 native-skia 라이브러리가 필요하다.
Ctrl+A/C/X/V로 Qt clipboard의 복사·잘라내기·붙여넣기와 한 번의 undo/redo도 검사한다.
다중 문단 붙여넣기로 여러 쪽을 만든 뒤 undo하여 페이지 크기 배치·뷰가 함께 갱신되는지도 검사한다.
문서 수명 검사는 dirty 닫기 취소, 기존 입력 뒤 dirty 확인, 저장 대화상자 취소, 쓰기·ack·executor 실패,
저장 중 추가 편집, 늦은 파일 읽기와 이전 action 세대를 포함한다. 실제 native 스모크도 dirty 닫기 취소와
저장 bytes 준비 이후 추가 편집을 실행하여 저장본과 현재 문서를 각각 확인하고, 변경 폐기 후 새 문서를 연다.
`native_worker_file_store_contract.py`는 동일 payload의 손실 승인, 외부 변경, publication 이후 ack 실패와
token cleanup을 검사하며, Native 라이브러리를 인자로 주면 실제 HWP/HWPX 저장·재열기도 실행한다.
타일 검사에는 실제 origin을 반영한 paint, 겹치는 스크롤의 image 재사용, overlay 보존, stale buffer 무복사 해제와
render 취소 중 편집 FIFO 유지가 포함된다. 실제 native 스모크에서도 Native cache 통계·상한, 겹치는 스크롤의
추가 buffer 복사 없음과 실행 중 render 취소 요청 뒤 기존 화면 복구를 검사한다.
접근성 계약은 UTF-16·scalar·문단/셀 주소 변환, snapshot/geometry 요청 상한과 stale 응답 거절을 검사한다.
Qt 검사에는 provider 연결 전 factory query, 실제 text interface의 emoji/선택/커서/경계 조회,
getter의 native 호출 금지와 비동기 문자 geometry가 포함된다. 실제 Native 스모크도 본문 읽기·선택·geometry·
undo·재열기·새 문서·닫기 뒤 semantic 상태를 Qt 접근성 API로 확인한다.

근거: [QInputMethodEvent](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QInputMethodEvent.html),
[QImage](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QImage.html),
[QWidget](https://doc.qt.io/qtforpython-6/PySide6/QtWidgets/QWidget.html).

## 자동 복구

`--recovery-dir <전용 폴더>`로 자동 복구를 활성화한다. 기본 간격은 30초이며
`--recovery-interval`로 1초~24시간 범위에서 바꿀 수 있다. 조합·입력·저장·파일 I/O 중에는
시작하지 않고 같은 revision은 중복 보관하지 않는다. 복구 파일을 생성해도 원본 문서의
수정 상태와 저장 checkpoint는 유지한다.

툴바의 ‘복구 파일 열기’는 확인한 payload를 저장되지 않은 새 문서로 연다.
복구 metadata의 원본 경로는 자동 저장 위치로 사용하지 않는다.
보관 한도에 도달하면 기존 파일을 삭제하지 않고 새 보관을 중단한다.
세부 계약과 남은 OS qualification은 `mydocs/tech/native_worker_recovery.md`를 따른다.
