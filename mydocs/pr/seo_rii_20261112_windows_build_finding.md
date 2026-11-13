# Windows Flutter junction 헤더 경로 진단과 보정안

## 재현과 원인

후보 `1e819721fe789b6b3b5c6db600635245039c9f98`의 실제 Windows D3D Native 빌드는 성공했다. 같은 소스·Flutter 3.47.9·MSVC에서 plugin 3개 단위와 등록 코드의 엄격 컴파일 4건도 통과했다. 그러나 `build_desktop_probe.py --tile-group`이 생성한 실제 runner는 `NativeApi.h:3`의 `rhwp_native_ffi.h` 검색에서 C1083으로 실패했다.

실제 생성 `.vcxproj`는 `.plugin_symlinks/rhwp_native_reference/windows/../../swift/...`를 include 경로로 사용했다. CMake의 기존 `get_filename_component(... REALPATH)`와 `file(REAL_PATH)` 모두 Windows junction 별칭을 유지했다. `IS_SYMLINK`·`READ_SYMLINK`로 확인한 실제 target은 NT namespace `\??\C:\...\source\bindings\Flutter\`다. 이 target을 정상 Windows 경로로 변환하면 실제 공통 헤더의 존재가 확인된다. 헤더를 빌드 전에 누락한 문제나 Native C ABI 컴파일 오류가 아니다.

해당 `REALPATH` 도입은 `a92974dca48daad1d639d3d5f46267a5093ff2af`다. #8727의 Windows bridge → 통합 #8739 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd` → 후속 `91dea56707`의 close registry 검사 추가까지 추적했다. 이전 독립 검증은 실제 Windows Native D3D12·SDK 단위 검사였으며 최종 Flutter 소비자 sampling·지속 표시의 완료를 주장하지 않았다. 이번에는 그 실제 runner의 junction 경계를 처음 실행해 실패를 확인했다. 최신 devel과 세 원 PR이 공유하는 Windows CMake 설정이며 #8953의 참조앱과 #8954의 타일 collector 실행에 함께 영향을 준다. macOS 실기 부재의 이번 수용 결정과 별개인 실제 Windows 실패다.

## 제한 보정·검증 제안

- Windows CMake에서 Flutter package junction target을 명시적으로 해석한다. NT drive/UNC prefix와 상대 symlink target을 정규화하고 기존 공통 헤더를 그대로 사용한다. 헤더 사본·환경 변수 우회·검사 완화는 추가하지 않는다.
- 일반 직접 checkout과 실제 junction 경로를 대조하고, 실제 Windows runner compile/link → 두 타일의 최초·편집·창 resize·renderer recovery·자원 종료를 재실행한다.
- Windows 참조앱을 빌드해 이동 경로의 실제 실행도 확인한다. 해당 설정의 검증 뒤 관련 Python·Dart 검사와 Linux 정상 대조를 다시 확인한다. Native Rust/WASM 입력은 보정하지 않는다.
- 구체적인 patch는 로컬 `output/pr-review/seo-rii-20261112/proposed-windows-junction-correction.patch`에 준비했다. 아직 제품 소스에 적용하지 않았고 원격 변경도 수행하지 않았다.

## 판정

#8953·#8954의 Windows 필수 실제 경로는 보정·재검증 전까지 머지 보류다. 메인테이너가 이 기존 빌드 결함의 보정을 맡는 범위 승인을 요청한다. Mac 수용 결정을 다시 묻지 않는다.

## 2026-11-13 승인

작업지시자가 “메인테이너 보정·필수 재검증 승인 (권장)”으로 위 보정과 실제 재검증을 승인했다. Windows CMake의 공통 경로 보정을 적용하고 실제 runner·두 타일·이동 참조앱을 검증한다. 기존 macOS 수용 결정은 유지하며 원격 source push·PR·merge는 이번 범위에 포함하지 않는다. [승인 범위](assets/seo-rii-20261112/windows-correction-20261113/validation-record.md)를 보존했다.

## 실제 Windows 참조앱에서 추가 확인한 파일 metadata 경계

CMake 보정 후 실제 Windows 두 타일·편집·resize·복구·종료는 변경 없는 collector에서 성공했다. 이어서 #8953 참조앱의 release 빌드도 성공했지만, 패키지의 정규 파일 읽기에서 `bundle input changed before read`로 중단됐다. 동일 `.exe`의 `lstat.st_mode=0o100777`, 열린 fd의 `fstat.st_mode=0o100666`을 실제 Windows Python 3.12.3에서 대조했다. inode·device·size·mtime·nlink는 같았다. ctime의 다른 권위는 기존 `54d987cfb5962e7cf6e49d8460a96bef7143f0e8`의 birthtime 정규화로 이미 처리한다.

[CPython 3.12.3 원문](https://github.com/python/cpython/blob/v3.12.3/Modules/posixmodule.c#L1843-L1856)도 경로의 `.exe`·`.bat`·`.cmd`·`.com` 확장자로 실행 비트를 합성하며 실제 ACL 실행 권한을 측정하는 값으로 사용하지 않는다고 설명한다. Native DLL만 읽을 때 드러나지 않던 경계가 새 runtime `.exe` 패키징에서 실행됐다.

승인한 Windows 참조앱 검증을 마무리하기 위해 공통 `_file_identity`에서 Windows의 이 합성 실행 비트만 정규화한다. 파일 종류·읽기/쓰기 비트·device·inode·size·mtime·birthtime·nlink·reparse 및 hash 거부 계약은 유지한다. 원시 path/fd metadata는 각각의 권위에서 전후 변경을 계속 거부하도록 확인한다. POSIX 실행 권한은 정규화하지 않는다. 이 보정은 Windows 실행 허용 정책이나 보안 문턱의 예외가 아니며 기존 정상/변조 반례와 실제 `.exe`를 함께 재검증한다.

## 실제 이동 번들의 Windows 창 종료

소스 `7e576a18ca`의 실제 release 번들은 이동 후 두 문서 열기·1→6쪽 이동·문서 닫기를 통과했다. 그러나 WM_CLOSE 후 창이 먼저 없어지고 프로세스가 아직 종료 중인 구간에 캡처를 요청한 검증 하네스의 race가 있었다. 종료 대기를 별도로 고쳐 실제 프로세스를 확인하자 `0xc000041d`가 재현됐다. 이전 종료를 성공으로 판정하지 않는다.

Windows Application Error는 `flutter_windows.dll`의 access violation `0xc0000005`, RVA `0x1551f`를 기록했다. 실제 DLL export에서 `FlutterDesktopMessengerSetCallback`은 RVA `0x154f9`이며, plugin 소멸자의 무조건적인 `SetMethodCallHandler(nullptr)` 호출과 연결된다. 고정 Flutter SDK의 `PluginRegistrar::~PluginRegistrar`는 plugin부터 제거하지만 `BinaryMessengerImpl`은 C messenger의 비소유 참조다. 엔진 종료 뒤 unregister는 이미 무효화한 engine을 호출한다. 원 소스 `a92974dca48d` 이후의 plugin lifetime 경로를 함께 대조했다.

C SDK의 AddRef/Release·Lock/Unlock·IsAvailable 계약을 사용해 messenger 수명을 소유하고, 소멸자에서는 살아 있는 engine일 때만 handler를 제거한다. texture owner의 기존 close·retirement 처리는 유지한다. 승인된 실제 Windows 종료 검증을 완성하기 위한 공통 plugin 보정이며, raw pointer를 잡아 두거나 오류 종료를 정상 종료로 허용하지 않는다. 실제 runner를 다시 만들어 최초·편집·resize·복구·자원 종료와 이동 참조앱의 WM_CLOSE exit 0을 재검증한다.

## 최종 재검증 완료

`4ab994ee67`에서 실제 Windows reference release·타일 runner를 새로 빌드했다. 타일 collector 4단계는 `windows-two-tile-consumer-passed`·exit 0이며, 이동 번들은 실제 두 문서 열기·1→6쪽 이동·문서 닫기·WM_CLOSE exit 0을 확인했다. 최초 C1083·`.exe` metadata 거부·messenger 종료 fault를 모두 해소했다. 관련 Linux 검사 50개와 Windows portable bridge도 통과했다. 실제 SDK·DPR·카운터·module mapping·전쪽 Native/WASM 결과는 [최종 검증 원장](assets/seo-rii-20261112/windows-correction-20261113/README.md)에 연결했다. 원격 처리는 아직 수행하지 않았다.
