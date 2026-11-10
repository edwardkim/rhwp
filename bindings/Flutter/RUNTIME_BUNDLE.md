# Flutter 데스크톱 실행 번들

이 도구는 **이미 빌드한** Flutter desktop runtime과 명시적으로 공급한 Native 라이브러리·owner
dispatcher를 한 디렉터리에 묶는다. SDK·컴파일러·글꼴을 새로 설치하거나 Rust/Flutter를 빌드하지 않는다.
프로젝트의 기존 `native_artifact_provenance.py`에서 사용하는 안정된 파일 descriptor·reparse 판정을
재사용한다. 기존 Qt 전용 `scripts/package-native-host.py`의 실행 번들과 별도 계약이다.

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
