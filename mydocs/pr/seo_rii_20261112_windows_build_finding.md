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

작업지시자가 “메인테이너 보정·필수 재검증 승인 (권장)”으로 위 보정과 실제 재검증을 승인했다. Windows CMake의 공통 경로 보정을 적용하고 실제 runner·두 타일·이동 참조앱을 검증한다. 기존 macOS 수용 결정은 유지하며 원격 source push·PR·merge는 이번 범위에 포함하지 않는다. [승인 범위](assets/seo-rii-20261112/windows-correction-approval.json)를 보존했다.
