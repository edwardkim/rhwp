# Native 참조 host 사용법

`native_host.py`는 Native C ABI를 사용하는 **창 없는 Python 표준 라이브러리 예제**다.
WASM·브라우저·DOM 없이 문서 열기, RGBA 렌더, 본문 편집·조합, 저장·재열기를 실행한다.
Python 3.9 이상과 `native-skia` 기능으로 빌드한 공유 라이브러리가 필요하다.
예제는 라이브러리를 자동 탐색하거나 빌드하지 않으므로 `--library`를 지정한다.

## 라이브러리 준비

저장소 루트에서 기존 공용 target을 재사용한다. worktree에서도 기본 작업공간의 같은 절대 경로를 쓴다.
다른 Cargo 작업이 끝난 뒤 순차 실행한다. 자세한 환경은
[개발 환경 안내](../../../mydocs/manual/dev_environment_guide.md)를 따른다.

```sh
cargo build --locked -p rhwp-native-ffi --features native-skia \
  --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review
```

Linux 출력은 `target/pr-review/release-test/librhwp_native_ffi.so`다.
macOS는 `.dylib`, Windows는 `rhwp_native_ffi.dll`을 사용한다.
예제의 `/path/to/`와 target 경로는 실제 작업공간과 운영체제에 맞게 바꾼다.
호출 시에는 라이브러리와 입력·출력 경로를 절대 경로로 지정하면 현재 디렉터리에 영향을 받지 않는다.

## 새 문서 편집·저장

```sh
python3 /path/to/rhwp/bindings/Native/examples/native_host.py \
  --library /path/to/librhwp_native_ffi.so \
  --output /path/to/new-document.hwpx
```

빈 문서를 만들고 페이지 영역을 RGBA로 렌더한 뒤 hit-test·caret를 조회한다.
본문 편집에서 `ㄱ → 가 → 각` 조합을 한 번 확정하고 undo/redo를 실행한다.
HWPX 버퍼를 복사·해제하고 **새 파일**을 기록한 뒤 저장 revision을 acknowledge한다.
세션을 닫고 다시 열어 `rhwp_read_text`로 내용을 확인한다.
JSON 요약에는 렌더 메타데이터, 저장 진단과 재열기 결과가 포함된다.
픽셀 해시는 산출물의 신원이며 한컴과의 시각 일치 증거는 아니다.

## 기존 문서 조회·사본 저장

```sh
python3 /path/to/rhwp/bindings/Native/examples/native_host.py \
  --library /path/to/librhwp_native_ffi.so \
  --input /path/to/source.hwp \
  --output /path/to/new-copy.hwpx
```

`--input`을 지정하면 그 문서에서 편집 데모를 실행하지 않는다.
인라인 객체처럼 지원하지 않는 상호작용 대상은 명시적으로 보고하고 viewer 세션을 유지한다.
렌더·포맷 오류는 실패로 반환한다. 입력을 읽기만 하고 출력은 배타적으로 새로 만든다.
기존 대상·심볼릭 링크·원본 파일을 덮어쓰지 않는다. 이 예제 CLI는 암호 입력을 제공하지 않는다.

## 버퍼·저장·좌표 계약

- 파일 I/O는 host 책임이다. 파일 기록·flush·close가 끝난 뒤에만 저장 token을 acknowledge한다.
  기록 실패 시 token을 취소하고 이번 호출이 만든 미완성 파일만 제거한다.
  acknowledge 실패 시 완성된 출력은 보존한다.
- 이 예제의 새 파일 저장은 기존 파일 교체나 crash recovery journal 구현이 아니다.
  교체·복구 adapter는 `native_file_store.py`와 상위 bindings 안내를 참고한다.
- Native 결과 문자열은 `rhwp_string_free`로 해제한다. 렌더·저장 버퍼는 복사한 뒤
  `rhwp_session_buffer_release`로 해제한다. 실패 경로에서도 `finally`에서 세션을 닫는다.
- 세션 handle은 thread 소유이므로 모든 호출은 같은 thread에서 실행한다.
- `originX`/`originY`는 배율 적용 후 페이지 픽셀 좌표다. hit-test·caret는 배율 적용 전의
  페이지 조판 좌표다. 이 예제의 영역 배율은 1이므로 조판 좌표를 그대로 전달한다.

## 검증 명령

라이브러리 없이 파일 기록·정리·checkpoint 순서의 mock 계약만 확인한다.

```sh
python3 /path/to/rhwp/bindings/Native/tests/native_host_smoke.py --mock
```

실제 Native ABI에서 빈 문서 편집·저장·재열기와 입력 문서 조회·사본 저장을 확인한다.

```sh
python3 /path/to/rhwp/bindings/Native/tests/native_host_smoke.py \
  --library /path/to/librhwp_native_ffi.so
```

mock 통과와 실제 라이브러리·대상 플랫폼 실행은 구분해 보고한다.
실행하지 않은 macOS/Windows SDK·OS UI·한컴 시각 검증을 완료로 해석하지 않는다.

## 지원 범위

이 예제의 편집 데모는 지원되는 본문 텍스트 범위다. cell 편집·inline control affinity,
OS 키 바인딩·clipboard transport·접근성 등록·창·GPU 표시·OS IME 후보창을 구현하지 않는다.
조합 이벤트는 코드가 직접 공급한다. HWPX는 현재 editor focus 대신 원본/기본 caret 정책을
보존할 수 있으므로 `caretSaved`·`caretPreservedFromSource`를 확인한다.
`save.report`의 내용 손실 진단은 실제 앱의 저장 UI에서 판단할 정보로 전달해야 한다.
