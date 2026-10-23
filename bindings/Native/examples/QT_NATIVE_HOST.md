# Qt native 창 예제

`qt_native_host.py`는 PySide6 QWidget에서 native Skia 픽셀을 표시하는 작은 참조 앱이다.
WASM·DOM·브라우저를 사용하지 않는다. Rust worker가 문서 core를 소유하며 GUI는
`QTimer`로 완료를 받는다. 입력은 최대 64개 action FIFO이고 문서 요청은 한 번에 하나만 보낸다.
이미 실행 중인 조판·Skia 작업의 강제 취소를 보장하지 않는다.

```bash
python3 -m venv .venv-qt
.venv-qt/bin/python -m pip install -r bindings/Native/examples/qt-requirements.txt
.venv-qt/bin/python bindings/Native/examples/qt_native_host.py \
  --library /absolute/path/to/librhwp_native_ffi.so
```

`--input sample.hwpx`로 기존 문서를 열 수 있다. 입력 파일을 자동 수정하거나 저장하지 않는다.
`--output new.hwpx`는 저장 대화상자의 기본 경로다. 저장은 새 파일에만 허용하며 기존 파일·심볼릭
링크를 덮어쓰지 않는다. bytes 복사와 buffer release는 GUI 스레드에서 수행하고, 파일 쓰기·flush·close는
별도 I/O executor에서 수행한 뒤 native 저장 token을 acknowledge한다. 문서를 여는 파일 읽기도 I/O
executor를 사용한다. 파일 쓰기가 실패하면 cancel하며, 이미 성공한 파일은 ack 오류 때문에 삭제하지 않는다.

## 표시와 입력 범위

- 여러 쪽을 세로로 표시한다. 참조 UI는 최대 512쪽을 읽는다. P58 view 계약이 zoom·DPR·scroll·좌표
  변환을 담당한다. wheel은 스크롤, Ctrl+wheel은 포인터 기준 확대·축소다.
- 화면에 보이는 region만 페이지별로 순서대로 렌더한다. native frame은 복사·release 후
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
- Qt `QInputMethodEvent` preedit→update→commit을 native composition에 연결한다. Focus loss는 cancel이다.
  surrogate pair 중간 offset과 주변 문자열 replacementStart/replacementLength는 안전하게 거절한다.
  주변 문맥·cursor·anchor는 마지막 완료된 native snapshot에서 읽고 scalar→UTF-16으로 바꾼다.
  bounded 문맥 밖의 anchor와 absolute position은 완전한 문서 위치라고 주장하지 않는다.
- preedit cursor와 밑줄 색을 지원하며 조합 범위에는 점선 밑줄을 표시한다. Qt IME의 임의 TextFormat
  글꼴/배경, 복잡한 clause별 장식, 주변 문자열 재변환은 아직 구현하지 않았다. OS별 실물 IME 검증은 별도다.
- 종료는 native shutdown 신호를 보낸 뒤 worker가 끝날 때까지 Qt event loop를 유지한다.
  진행 중인 파일 작업은 먼저 마쳐야 한다. 실행 중 작업을 취소 완료로 표시하지 않는다.

## 검사

```bash
QT_QPA_PLATFORM=offscreen .venv-qt/bin/python -B bindings/Native/tests/native_window_smoke.py
QT_QPA_PLATFORM=offscreen .venv-qt/bin/python -B bindings/Native/tests/native_window_smoke.py \
  --library /absolute/path/to/librhwp_native_ffi.so \
  --screenshot /new/path/qt-window.png
```

첫 명령은 UTF-16 경계, QImage 소유권, IME replacement 거절, 큐 상한의 Qt adapter 검사다.
라이브러리를 지정하면 실제 blank→region 표시→합성 Qt 조합 ㄱ/가/각→commit→좌우 키 이동→undo/redo→HWPX 새 파일
저장→재열기를 검사한다. 이 검사는 OS 한글 입력기·후보창, 한컴 시각 일치 또는 플랫폼 배포 패키지 검증을
대체하지 않는다. `--screenshot`은 선택 사항이며 실행 중인 QWidget을 새 PNG 파일로 남긴다.
실제 스모크에는 P61까지의 worker/editor/IME 문맥·clipboard ABI가 포함된 native-skia 라이브러리가 필요하다.
Ctrl+A/C/X/V로 Qt clipboard의 복사·잘라내기·붙여넣기와 한 번의 undo/redo도 검사한다.
다중 문단 붙여넣기로 여러 쪽을 만든 뒤 undo하여 페이지 크기 배치·뷰가 함께 갱신되는지도 검사한다.

근거: [QInputMethodEvent](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QInputMethodEvent.html),
[QImage](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QImage.html),
[QWidget](https://doc.qt.io/qtforpython-6/PySide6/QtWidgets/QWidget.html).
