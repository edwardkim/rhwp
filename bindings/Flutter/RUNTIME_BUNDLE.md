# Flutter 데스크톱 실행 번들

이 도구는 **이미 빌드한** Flutter desktop runtime과 명시적으로 공급한 Native 라이브러리·owner
dispatcher를 한 디렉터리에 묶는다. SDK·컴파일러·글꼴을 새로 설치하거나 Rust/Flutter를 빌드하지 않는다.
프로젝트의 기존 `native_artifact_provenance.py`에서 사용하는 안정된 파일 descriptor·reparse 판정을
재사용한다. 기존 Qt 전용 `scripts/package-native-host.py`의 실행 번들과 별도 계약이다.

## 기존 참조앱 빌드와 실행

`reference_app.py`는 `example/lib/`의 앱을 복사해 현재 OS의 Flutter runner를 생성하고,
plugin을 등록한 실제 데스크톱 앱과 owner dispatcher를 빌드한 뒤 위 번들 형식으로 묶는다.
준비된 Flutter SDK·OS 개발 도구·pub cache와 명시적으로 공급한 Native 라이브러리가 필요하다.
소스 checkout은 수정하지 않으며, 새로 만든 외부 출력 폴더에 source 사본·runner·로그·번들을 둔다.
Native 라이브러리를 다시 빌드하지 않으므로 해당 파일의 source SHA는 별도로 선언한다.

```bash
python3 -B bindings/Flutter/tool/reference_app.py build \
  --flutter-sdk /SDK/flutter --native /built/librhwp_native_ffi.so \
  --native-source FULL_NATIVE_GIT_SHA --output /build/reference-app
mv /build/reference-app/bundle '/packages/rhwp reference'
python3 -B bindings/Flutter/tool/reference_app.py run --bundle '/packages/rhwp reference'
```

동일한 명령을 해당 Windows/macOS 호스트에서 실행하며 Native 인수는 그 OS의 DLL/dylib를
지정한다. Windows는 해당 아키텍처의 MSVC Native Tools 환경에서 `cl`을 제공해야 한다.
생성한 Linux runner는 `fl_view_new` 이전에 공개 `FlDartProject` API로
`FL_UI_THREAD_POLICY_RUN_ON_SEPARATE_THREAD`를 지정한다. Dart UI와 Native owner 응답을
확인하는 Dart timer를 GTK platform thread의 draw/resize 작업에서 분리한다. Native 문서 작업은
기존 owner dispatcher와 Rust worker에서 수행하며, plugin의 GTK 접근은 platform thread에 남는다.
또한 `fl_dart_project_set_enable_impeller(project, FALSE)`로 Flutter compositor의 Skia 경로를
선택한다. Flutter 3.47.9의 Impeller GLES 경로와 현재 Native GL bridge를 함께 사용한 큰 창
resize에서 관측한 texture sampling 정체를 피하기 위한 호환성 설정이다. Flutter 엔진을 수정하거나
Native renderer·snapshot 수명·publication 제한 시간을 바꾸지 않는다. 이 API를 제공하는 Flutter
Linux SDK가 필요하며 예상한 runner 생성 지점을 찾을 수 없으면 빌드를 거절한다.

P120 Linux 대조에서는 Flutter 3.47.9/Xvfb/Mesa llvmpipe에서
1280×720 → 2800×2000 → 1280×720 → 1600×1200 → 1280×720 순서를 사용했다.
Impeller 경로는 새 프레임 알림 뒤 텍스처 callback이 수 초 지연돼 기존 2초 제한을 넘겼다.
별도 UI 스레드만 사용하거나 software renderer의 스레드 수만 줄여도 재현됐지만,
Skia compositor와 별도 UI 스레드에서는 기존 plugin으로 두 타일의 serial 1–5가 모두
게시됐다. 이 대조의 Native 라이브러리는 `b5064fae1eefff92d60464c51844a38debec09f9`이며,
Native 재생 시간과 engine callback 대기 시간을 구분한 로컬 소프트웨어 GPU 근거다.
실물 GPU·Wayland·다중 모니터·최종 문서 대조를 대신하지 않는다.

생성한 macOS 개발용 runner는 App Sandbox를 끈다. 이 참조앱은 `.app` 밖의 Native·manifest와
입력한 문서 경로를 사용하며, App Store sandbox/bookmark 접근을 구현하지 않는다.
기본은 release와 GPU 사용이다. `--mode debug`, `--cpu`, `--continuous`, `--trace`로
기존 참조앱의 옵션을 선택한다. 연속 Windows/Metal 표시는 명시적인 `--continuous` opt-in이다.
`build-receipt.json`에는 source 파일 해시·Flutter/engine revision·각 명령의 종료 상태·산출물 해시를
기록한다. Linux 설정은 `linuxUiThreadPolicy: "separate"`, `linuxRenderer: "skia"`와 설정 후
generated runner 해시로 남는다. 이 값은 runner의 선택을 기록하며 실행 환경의 engine switch
override나 실제 compositor 측정을 대신하지 않는다. 성공한 빌드는 앱 실행이나 GPU 표시를
검증했다는 뜻이 아니다.

`run`은 현재 OS/아키텍처와 **전체** manifest를 재검증한 뒤 지정된 실행 파일을 shell 없이 실행한다.
현재 bundle root를 `RHWP_RUNTIME_BUNDLE`로 전달하므로 이전 빌드 경로나 cwd에 의존하지 않는다.
SDK는 실행 시 필요하지 않다. 앱의 종료 코드를 그대로 반환한다. 소스/SDK/빌드 폴더를 지워도
번들에 필요한 데이터·engine·plugin·Native·owner는 보존된다. 시스템 GTK/그래픽 드라이버와
플랫폼 loader 의존성은 해당 OS에서 제공해야 하며, OS 서명·installer·자동 업데이트는 별도 작업이다.

실행 후 참조앱에서 파일 열기·쪽 이동·확대·축소를 확인한다. GPU 오류·한도 초과 등으로
CPU fallback 또는 퇴역 상태가 되면 `GPU 다시 연결`로 같은 문서의 새 controller를 만든다.
정상 GPU에는 복구 버튼을 활성화하지 않는다. `문서 닫기`로 retirement를 확인한 뒤 창을 닫는다. 실제 화면·DPR·복구·종료 결과는 빌드 receipt와 구분해 기록한다.

launcher의 portable 음성 대조는 다음 명령으로 실행한다. 작은 실제 ELF child 검사는 이동한 root와
종료 코드 전달만 확인하며, rhwp/Flutter/GPU 실행 증거로 세지 않는다.

```bash
python3 -B bindings/Flutter/test/reference_app_contract.py
```

## 생성과 검증

```bash
python3 -B bindings/Flutter/tool/runtime_bundle.py create \
  --runtime /built/flutter-runtime --executable runner \
  --native /built/librhwp_native_ffi.so --dispatcher /built/librhwp_flutter_owner.so \
  --notices /source/rhwp --platform linux --architecture x86_64 \
  --runtime-source FULL_RUNTIME_GIT_SHA --native-source FULL_NATIVE_GIT_SHA \
  --dispatcher-source FULL_DISPATCHER_GIT_SHA --output /packages/new-runtime
python3 -B bindings/Flutter/tool/runtime_bundle.py verify --bundle /packages/new-runtime
```

`--runtime`는 Linux/Windows Flutter build의 bundle 디렉터리다. macOS는 `.app`를 포함하는 상위
디렉터리를 공급하고 `--executable App.app/Contents/MacOS/App`를 지정한다. macOS 내부의 상대
framework link는 보존하지만 절대 경로·`..`·외부 이탈·순환 link는 거절한다. 코드 서명·notarization은
생성하지 않는다. 실제 서명·배포 검증은 플랫폼 작업이며 번들 해시는 서명을 대신하지 않는다.

새 출력만 배타적으로 만들며 기존 디렉터리·파일·link를 덮어쓰지 않는다. runtime 전체 파일과
Native·dispatcher·`LICENSE`·`THIRD_PARTY.md`·`THIRD_PARTY_LICENSES.md` 및 공급된 `licenses/`를
해시 원장에 넣는다. Flutter runtime 자체의 license asset도 그대로 포함된다. 공급자는 실제 의존성의
배포 고지가 갖춰진 입력을 제공해야 하며 도구는 고지 내용의 라이선스 적합성을 판정하지 않는다.
개인·시스템 글꼴을 자동 탐색하거나 복사하지 않는다.

`rhwp-runtime.json`은 다음을 기록한다.

- schema 1과 `rhwp-flutter-desktop` 종류, Linux/Windows/macOS 및 x86_64/arm64 target.
- runtime·Native·dispatcher 각각의 **호출자가 선언한** 40자리 Git source SHA. source와 binary의
  빌드 대응을 해시만으로 입증하거나 모든 바이너리를 같은 commit에서 빌드했다고 추정하지 않는다.
- 상대 실행 파일·라이브러리 경로와 모든 member의 크기·SHA-256·실행 비트 또는 상대 link target.
- Native ABI 1, dispatcher ABI 2 요구. 이 숫자는 실제 로딩 시 기존 Native owner 경로에서 다시 검사한다.

최대 8,192개 member, 파일당 512 MiB, 전체 2 GiB, JSON 1 MiB, 깊이 32를 제한한다. empty asset과
empty directory는 보존한다. ELF/PE/Mach-O의 실제 machine/type metadata로 target을 대조하고
fat Mach-O slice도 확인한다. 헤더 검사는 동적 의존성의 설치나 실행 가능성을 입증하지 않는다.

`create`와 `verify`는 코드를 실행하거나 라이브러리를 열지 않는다. manifest와 해시를 함께
바꿀 수 있는 사람은 다른 코드를 넣을 수 있다. 신뢰할 수 있는 배포 출처·디렉터리 권한이 필요하며
이 API는 공격자가 동시에 수정하는 디렉터리를 sandbox로 바꾸지 않는다.

## Dart 참조 호스트 연결

```dart
final bundle = await NativeRuntimeBundle.load(Directory(bundlePath));
final transport = await bundle.connect();
final document = await NativeDocument.connect(transport);
```

호스트의 시작 시 한 번 사용한다. `load`는 현재 OS/ABI와 전체 member를 대조한 뒤 상대 경로를
현재 root로 해석한다. `connect`는 다시 검증한 뒤 기존 `NativeOwnerTransport.connect`를 사용한다.
기존 owner thread/TLS·동일 Native module·ABI·worker 계약을 우회하지 않는다. Native는
`abiVersion:1`, `nativeSkia:true`, `worker:true`, `hostRequests:true`, dispatcher는 snapshot cleanup이
있는 ABI 2여야 한다. 실제 공급 코드의 로딩은 `connect`를 호출하는 명시적인 동작이다.

참조 example은 `RHWP_RUNTIME_BUNDLE`가 지정되면 이 경로를 쓰고, 미지정이면 기존
`RHWP_NATIVE_LIBRARY`·`RHWP_OWNER_LIBRARY` 연결을 유지한다. OS 환경 변수를 먼저 읽고
compile-time dart define은 기본값으로 사용하므로 이동한 bundle root를 다시 컴파일하지 않고
지정할 수 있다. 이는 제품 앱 UI나
플랫폼 plugin의 자동 설치 기능이 아니다. 실제 Flutter runner는 빌드 시 plugin 등록을 포함해야 한다.

환경 변수·dart define이 없으면 `NativeRuntimeBundle.forExecutable()`이 실행 파일의
`app/` 상위 bundle을 찾는다. 따라서 이동한 `app/rhwp_reference_app`, Windows exe 또는
macOS `.app`를 직접 실행할 수도 있다. 확인 범위는 Linux/Windows 두 단계, macOS 앱 구조를
포함해 최대 다섯 단계이며, manifest가 지정한 runtime과 실제 실행 파일의 동일성을 확인한다.
manifest가 없을 때만 기존 개발용 Native/owner 경로를 사용한다. manifest가 존재하지만
잘못됐거나 다른 실행 파일을 가리키면 오류를 표시하며 다른 라이브러리로 자동 대체하지 않는다.

## 재현 검사

```bash
python3 -B bindings/Flutter/test/runtime_bundle_contract.py
cd bindings/Flutter
flutter test --no-pub test/native_runtime_bundle_test.dart
python3 -B test/runtime_bundle_ffi_contract.py --dart /SDK/flutter/bin/dart
# 실제 신뢰한 runtime/Native 입력을 검증·로딩할 때만 실행한다.
dart run tool/runtime_bundle_smoke.dart /packages/relocated-runtime
```

portable 검사는 ELF/PE/Mach-O **비실행 헤더 fixture**를 다룬다. FFI 검사는 Linux에서 현재
`owner_dispatcher.c`와 작은 C ABI fixture를 실제로 compile한 뒤 공백을 포함한 위치로 옮겨 Dart에서
연결·종료한다. 미지원 capability·잘못된 owner ABI도 실제 로딩 경로로 거절한다. 실제 rhwp 렌더러,
Flutter runner 실행, Windows/macOS loader·GPU·OS 서명 검증으로 표현하지 않는다. 실제 공급 Native와
Flutter runner의 별도 검증 source·hash를 기록한 후 사용할 수 있는 연결 기반이다.

추가 실제 Linux smoke에서는 `native-skia-gl` Native를 `devel@b5064fae1eef`에서
빌드한 library SHA-256
`59e1602dbf09bc0c8094ab5955c809689567d3cd0062bfdd086e480ea1d3e359`와
현재 owner dispatcher를 사용했다. 기존 Flutter runtime `26897412c355`를 공급해
17개 파일273,163,585bytes를 묶고 공백 경로로 이동한 뒤 초기 dependency 복사본을
제거했다. 이동한 library를 실제 로딩해 새 문서·64×64 RGBA 두 번 일치·snapshot의
full stamp 검사/해제·owner 종료를 실행했고 Native resource counter9개는 모두0으로
돌아왔다. 이 실행은 C fixture가 아니며 runtime runner 실행·GPU 표시 검증은 아니다.
각 role의 source는 독립적으로 선언하고 기존 runtime 재사용을 새 빌드로 계산하지 않는다.

P118에서는 runtime source `0ebb5b1dfa7290d275c493d8be647dd72891b75a`의 기존 참조앱을
Flutter 3.47.9 release로 새로 빌드했다. 위 `b5064fae1eef` Native와 현재 dispatcher를 사용한
32개 member·207,053,122 bytes 번들을 공백이 있는 경로로 이동하고 원래 runner/source 경로를
치운 뒤, bundle 환경 변수 없이 `/tmp`에서 실행 파일을 직접 열었다. 실제 GTK 창과
runner·engine·plugin·Native·owner의 `/proc` 매핑이 모두 이동한 번들의 파일임을 확인했다.
기존 `desktop-consumer.hwpx`와 radial 6쪽 fixture를 열어 표시한 뒤 `문서 닫기`의 완료 화면을
확인했고, 정상 `WM_DELETE_WINDOW`로 프로세스 종료 코드 0을 받았다. Native를 현재 devel에서
다시 빌드한 결과나 물리 GPU 검증으로 계산하지 않는다. 환경은 Mesa 22.3.6/llvmpipe이며 Flutter
engine 로그는 Impeller OpenGLES, 문서 producer는 공급한 Native Skia GL 경로다.

Dart 278개 검사(전체 실행 277개 통과 후 sparse checkout에 없던 fixture 1개를 정확한 Git 입력으로
보충해 재실행), analyzer·format, launcher 8개와 기존 bundle 11개 계약 검사를 통과했다.
복구는 기존 controller/session 회귀로 검증했고 이 앱 실행에서 강제 GPU 복구 성공을 관측한 것으로
표시하지 않는다. 매우 큰 GTK 창 resize에서는 frame 크기 전환 timeout도 관측했다. 실제 macOS·Windows
앱 실행과 복구·DPR·GPU 종료 계수 수집은 각 플랫폼의 후속 실행 증거로 남긴다.
