# 실제 Linux Flutter consumer 검증

이 경로는 GTK Flutter runner와 Linux GL plugin을 실제로 실행하고 X11 root의 표시 pixel을
동일 immutable snapshot의 Native CPU 출력과 대조한다. Flutter test screenshot, fake registrar,
별도 GL context의 readback을 consumer 표시 증거로 대신하지 않는다. 제품 example은 변경하지 않고
별도 디렉터리에 검증 runner를 생성한다.

준비된 Linux Flutter SDK와 Python 3, GCC/Clang·CMake·Ninja, GTK 3·epoxy 개발 패키지가 필요하다.
Native GL 기능과 full-stamp presentation ABI를 제공하는 실제 Native `.so`를 지정한다.
`--native-source-head`는 이 바이너리의 선언된 source SHA이며, 이 도구가 Native를 새로 빌드하거나
그 선언 자체를 검증하는 것은 아니다. 영수증에 실제 바이너리 SHA-256과 `nativeBuildVerified:false`를
함께 남긴다.

## SDK·GL feature·표시의 순차 실행

`tool/check_linux_desktop_sdk.py`는 Linux x86_64에서 기존 도구를 순서대로 실행한다.
준비된 Flutter SDK, C++ compiler, `pkg-config`의 GTK3/epoxy 개발 파일이 필요하며
SDK·패키지를 설치하거나 저장소 feature default를 변경하지 않는다. 출력은 checkout 밖의
새 경로이고 원본 소스는 커밋된 clean 상태여야 한다.

```bash
# 실제 public headers/engine으로 plugin 전체를 compile/link한다. Rust/Native는 실행하지 않는다.
python3 -B bindings/Flutter/tool/check_linux_desktop_sdk.py \
  --flutter-sdk /SDK/flutter --compile-only --output /private/linux-sdk-compile

# 새 고유 Cargo target에서 native-skia-gl을 빌드하고 두 Flutter probe를 빌드한다.
# --build-only를 생략하면 실제 X11 single 32회·두 타일 3회 수집/종료까지 수행한다.
python3 -B bindings/Flutter/tool/check_linux_desktop_sdk.py \
  --flutter-sdk /SDK/flutter --build-only --output /private/linux-sdk-build
```

Native 재빌드를 피하려면 `--native-artifact-root /artifact/linux-gl
--native-source-head FULL_GIT_SHA`를 함께 지정한다. artifact는 기존 검증기와 같은
bounded 디렉터리이며 `librhwp_native_ffi.so`, `native-build-receipt.json`,
`linux-sdk-receipt.json`을 포함해야 한다. 전체 Cargo cache를 artifact로 공급하지 않는다.
정확한 source/Native build input 동등성, `.so` hash, 성공한 `native-skia-gl`·
`x86_64-unknown-linux-gnu` build command를 검증한다. Linux의 `cargo +1.93.1 rustc
--lib --crate-type cdylib` host-default build는 성공한 실제 rustc host 명령·원본 stdout과
ELF64/DYN/x86_64 metadata를 추가로 대조해야 한다. CPU-only feature, 실패 명령,
다른 host/ELF/source와 숨겨진 command target override는 거절한다.
이후 별도 child에서 실제 `.so`를 열어
Native worker/Skia·full-stamp snapshot·CPU oracle·GL host-FBO capability를 확인한다.
CPU library와 CPU feature receipt는 GPU 성공 근거로 인정하지 않는다.

`linux-sdk-receipt.json`은 실제 SDK compile/link, Native build provenance, Native GL
feature probe, 두 Flutter runner build, single/tile compositor sampling을 별도 flag로 기록한다.
`--compile-only` 성공은 `linux-sdk-compiled`, `--build-only`는 `linux-desktop-built`이며
표시 검증 flag는 false다. capability probe는 context 생성·GPU replay·sampling 증거가 아니다.
실제 두 수집기가 각각 성공해야 `linux-desktop-consumer-passed`로 기록하며,
그 경우에도 consumer GPU fence·hardware acceleration은 false를 유지한다.
전체 수집에 `--memory-pressure-cycles 2`를 더하면 single 32회와 기존 두 타일 3단계 모두에서
실제 binding→Native trim·RSS·idle·후속 표시·종료를 확인한다. 통합 실행기는 0..2회만 받고,
측정을 생략하는 `--compile-only`·`--build-only`와 양수 압박 횟수의 조합은 거절한다.
두 경로의 압박 검증 flag와 영수증을 각각 기록하며, 회수량 0은 observedNoop으로 보존한다.
자세한 범위와 개별 수집기 옵션은 [메모리 검증 가이드](GPU_MEMORY_ACCEPTANCE.md)를 따른다.
각 child의 private log·PID·timeout·exit와 source/SDK/engine/artifact hash를 기록하고,
실패 시 이미 통과한 단계와 아직 입증하지 못한 단계를 구분한다.

`test/linux_sdk_runner_contract.py`의 executor·비실행 ELF fixture와
`test/native_artifact_provenance_contract.py`는 orchestration/거절만 검증한다.
실제 compiler·Native·Flutter/GTK·GPU 실행의 대체가 아니다. 이전 source/Native의
실제 SDK·llvmpipe 실행도 보존하되, 새 source의 실행으로 다시 계산하지 않는다.

## 개별 probe 빌드와 수집

```bash
python3 -B bindings/Flutter/tool/build_linux_probe.py \
  --flutter-sdk /SDK/flutter \
  --native /artifacts/librhwp_native_ffi.so \
  --native-source-head <40-character-native-source-sha> \
  --output /private/rhwp-linux-build
```

빌드 영수증은 원본/복사본 Dart·C·C++·CMake 입력, 실제 SDK header, Flutter/engine revision,
Native·dispatcher·plugin·engine·runner·공개 입력 및 Debug kernel/data 전체 bundle의 hash를 고정한다.
첫/마지막 표시 중 `/proc/<pid>/maps`의 실제 ELF device/inode도 대조하므로 `LD_LIBRARY_PATH`가
bundle 밖의 같은 이름의 다른 engine/plugin을 선택한 실행은 통과하지 않는다.
출력은 checkout 외부에 두며 기존 경로를 덮어쓰지 않는다. 각 명령은 시작부터 private log·PID·종료
상태를 기록하고 deadline을 넘으면 소유 process group을 정리한다. 원본 소스는 빌드 중 유지한다.

실행에는 다른 창이 겹치지 않는 X11 display가 필요하다. 물리 GPU가 없는 머신에서도 별도로 소유한
Xvfb와 Mesa software GL로 실제 GTK/Flutter compositor를 실행할 수 있다. 그 결과는 하드웨어 GPU
가속이나 Wayland compositor qualification으로 확대하지 않는다. 수집기의 지원 범위는 검증용
undecorated top-level client와 24/32-bit little-endian X11 RGB visual이다.

```bash
DISPLAY=:99 python3 -B bindings/Flutter/tool/qualify_linux_texture.py \
  --runner /private/rhwp-linux-build/runner/build/linux/x64/debug/bundle/rhwp_texture_qualification \
  --native /artifacts/librhwp_native_ffi.so \
  --dispatcher /private/rhwp-linux-build/librhwp_flutter_owner.so \
  --fixture /private/rhwp-linux-build/source/bindings/Flutter/test/fixtures/desktop-consumer.hwpx \
  --build-receipt /private/rhwp-linux-build/build-receipt.json \
  --frames 32 --output /private/rhwp-linux-capture
```

첫 표시→실제 편집→resize 이후 viewport 위치를 바꿔 최대 64회, 기본 32회의 서로 다른 publication을
관측한다. 매번 nonce·실행 PID·문서/revision/font/view/device/DPR 전체 stamp를 확인하고 Native CPU와
실제 root 화면의 pixel을 비교한다. 균일한 내부 영역은 채널 오차 1 이하이며, 나머지 차이는 CPU
경계의 1 pixel 영역 안에 있어야 한다. 빈 cover·오래된 표시·의미 있는 내용 누락은 실패다.
revision과 각 후속 viewport의 CPU pixel hash도 달라야 한다. held publication을 두 번 수집해 화면의
안정성을 확인한 뒤에만 다음 frame을 허용한다.

얇은 stroke에서는 저해상도 CPU의 혼합색 표본에 원래 source의 순수색이 하나도 없을 수 있다.
이 표본의 RGB 최솟값/최댓값만 쓰면 정상적인 GPU coverage를 잘못 거절한다. 이를 임의의 색상
오차 상향으로 대신하지 않는다. 같은 snapshot/full stamp를 configure/capture 교체 없이 2–4배 CPU
해상도로 독립 재생하고, viewport와 placement만 같은 비율로 확대한다. geometryProof의 실제 CPU
descriptor·snapshot·stamp·비율을 대조하며 4096 dimension·4 MP·16 MiB와 Native scale/좌표 한도 안에서
가장 큰 배율을 선택한다. 2배도 불가능하면 `referenceUnavailable`로 실패한다.

AA 경계 색상은 그 독립 source의 1 pixel 주변 palette에서 13개 고정 RGB 방향
(`{-1,0,1}³`의 0·반대 방향 중복 제외)의 지원 범위 안에 있어야 한다. 채널별 양자화 1을 방향의 L1
크기로 투영하며, 단순 RGB 상자가 허용할 수 있는 source에 없는 색상 상관관계도 거절한다. GPU
캡처에서 기대 색상이나 보정값을 만들지 않는다. 추가로 CPU 경계 표본의 RGB 오차가 3×3 정수 이동 중
zero-shift에서 유일하게 가장 작아야 한다. 더 나은 이동이나 동률로 모호한 정렬은 실패한다. 기존
uniform interior 한도와 1 pixel 경계 지원은 유지한다. reference가 없는 기존 Windows 수집기의 엄격한
계약은 그대로이며 Linux 수집기는 reference 누락 시 약한 비교로 fallback하지 않는다.

Linux의 기존 `RasterRegistry`는 같은 raster OS thread/current context에서 front/back target을
사용한다. 이번 도구는 새 범용 GPU retirement API를 만드는 것이 아니라 그 실제 consumer 표시를
관측한다. 실제 retained context의 GL vendor/renderer/version은 최초 owner 생성 시 기록하며,
별도 context의 driver 문자열로 대체하지 않는다. 각 표시 동안 live raster slot 1·readback 0·
orphan/retire 실패 0을 확인하고, mount를 유지한 실제 close 뒤 등록 해제·slot 0·Native handle/lease 0을
확인한다. 공개 consumer GPU fence를 새로 얻었다는 뜻은 아니므로 `consumerGpuCompletionVerified`
값은 false다.

성공 영수증의 `continuousPresentationSupported:true`는 4회를 넘는 이 Linux GL 경로의 관측 결과다.
Windows/macOS의 불변 target 예산이나 consumer 완료 계약을 바꾸지 않는다. 실제 driver 문자열과
상관없이 이 수집기는 `hardwareAccelerationVerified:false`를 유지하며 실물 GPU 성능·장치 상실·
Wayland·실물 OS IME/접근성·최종 한컴 비교는 별도 검증 범위다.

```bash
cd bindings/Flutter
python3 -B test/linux_qualification_contract.py
python3 -B test/desktop_qualification_contract.py
```

위 portable 검사는 잘못된 nonce/PID/stamp, 실제 raster 진행/driver identity 누락, whole-frame
readback·orphan 자원·등록 해제/Native 정리 누락, source에 없는 hue·경계 밖 halo·1/2 pixel 이동·
잘못된 단색·빈 cover·stale 표시와 모호한 정렬을 거절한다. Linux SDK나 실제 compositor를 실행한
결과와 구분한다. 실제 표시 성공은 별도 target 실행 영수증으로 확인한다.
