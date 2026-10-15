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
- click/drag, selection/caret overlay, 좌우·상하·Home/End, Backspace/Delete, Enter, Ctrl+A/Z/Y,
  Ctrl+C/X/V를 연결한다. Home/End는 문단 경계다. 셀·inline object 편집은 native API의 거절을 표시한다.
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
실제 스모크에는 P60까지의 worker/editor/IME 문맥 ABI가 포함된 native-skia 라이브러리가 필요하다.
창의 복사·잘라내기·붙여넣기 기능에는 P61 clipboard ABI도 필요하며 위 스모크는 그 기능을 검사하지 않는다.

근거: [QInputMethodEvent](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QInputMethodEvent.html),
[QImage](https://doc.qt.io/qtforpython-6/PySide6/QtGui/QImage.html),
[QWidget](https://doc.qt.io/qtforpython-6/PySide6/QtWidgets/QWidget.html).
