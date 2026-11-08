# Apple SDK·Metal producer 검증

이 단계는 실제 macOS에서 Native Metal backend와 Objective-C++ plugin을 빌드·링크하고,
생성한 Native dylib의 실제 Metal device·queue·target을 검증하기 위한 도구입니다. 기본 피처와
기존 네 개 불변·quarantine 프레임 한도는 변경하지 않습니다. 연속 Flutter GPU 표시나 consumer
GPU 완료를 구현한 단계가 아닙니다.

## 실제 target 실행

준비된 Xcode·macOS SDK, Rust 1.93.1, 해당 Mac용 Flutter SDK의 `FlutterMacOS.framework`가
필요합니다. source·lockfile·검사를 커밋한 완전한 checkout에서 실행합니다. 지원 architecture는
현재 Python host와 일치하는 `arm64` 또는 `x86_64`입니다. Native Cargo 빌드도 각각
`--target aarch64-apple-darwin` 또는 `--target x86_64-apple-darwin`으로 고정합니다.
선택한 Rust target의 표준 라이브러리를 미리 설치해야 하며, Rosetta Python에서도 암묵적인
Rust host와 혼합하지 않습니다. framework의 Headers와 binary는 지정한
Flutter SDK 안에 있어야 하며 SDK의 Flutter·engine revision과 framework binary hash를 기록합니다.

```bash
python3 bindings/Flutter/tool/check_macos_metal_sdk.py \
  --flutter-sdk /SDK/flutter \
  --flutter-framework /SDK/flutter/bin/cache/artifacts/engine/darwin-arm64/FlutterMacOS.xcframework/macos-arm64_x86_64/FlutterMacOS.framework \
  --enable-metal-experiment \
  --output /private/tmp/rhwp-metal-sdk-qualification
```

framework 경로는 실제 SDK의 slice에 맞게 지정합니다. `--enable-metal-experiment`는 별도 scratch
Cargo 빌드의 `native-skia-metal`에만 적용합니다. 저장소의 feature default, 앱 배포 설정이나 plugin의
consumer 정책을 변경하지 않습니다. 이 명시적 옵션이 없거나 실제 Darwin·Apple clang이 아니면
실행을 거절합니다. Linux의 SDK 대역이나 Wine을 실제 target 성공으로 표시하지 않습니다.

도구가 수행하는 순서는 다음과 같습니다.

1. Apple SDK·clang·Flutter/engine revision과 소스 SHA·hash를 고정합니다.
2. 실제 FlutterMacOS framework와 Metal/CoreVideo/CoreGraphics/IOSurface/QuartzCore를 사용하여
   `RhwpMetalPlugin.mm` 전체를 ARC·C++17·`-Wall -Wextra -Werror`로 컴파일·링크합니다.
3. 같은 Apple SDK에서 작은 typed Metal probe dylib와 Foundation CV 종료 회귀,
   AppKit/Core Animation 소유 view의 animation 정책 회귀를 빌드합니다. 두 회귀 실행이
   통과해야 다음 단계로 진행합니다. animation 회귀는 실제 CA action lookup과 ambient
   animated transaction에서 소유 layer의 암묵적 action 차단, 즉시 geometry/가림/분리,
   caller context 복원을 검사하며 GPU/화면 수집으로 보고하지 않습니다.
4. 고유 `native-target`·`native-build`로 실제 Native Metal dylib를 빌드하고 동적 라이브러리
   의존성을 기록합니다. 검증한 Native artifact를 명시적으로 재사용하면 Rust 재빌드는
   생략하지만 SDK compile/link와 두 회귀·독립 producer 실행은 생략하지 않습니다.
5. 실제 Native worker의 불변 snapshot으로 opaque·premultiplied alpha clear와 편집 후 stale 거절을
   실행합니다. unpublished target이 변경되지 않는지와 최종 source/snapshot/frame/lease 계수 0을
   확인합니다.

각 command는 시작부터 private 로그·PID를 기록하며 종료 코드와 artifact hash를
`macos-sdk-receipt.json`에 남깁니다. 소스는 검증 전후 동일해야 합니다. 오래 걸리는 빌드가 끝날 때까지
호출자가 완료를 확인하고, 종료된 명령의 로그를 확인합니다. 기존 출력 경로는 덮어쓰지 않습니다.
source가 변경되거나 실제 명령이 실패하면 해당 실행은 실패로 기록합니다.
`cvTerminalStatusVerified`와 `metalViewAnimationPolicyVerified`는 각 실제 SDK 회귀가
실행되어 통과한 뒤에만 true로 기록하며, Metal producer와 consumer 표시는 구분합니다.

source identity는 prefix 목록이 아닌 전체 tracked Git 파일의 mode·blob OID manifest입니다.
매 step 전후 전체 working tree/index의 clean 상태를 검사하므로 `vendor` path 의존성,
workspace manifest, 내장 글꼴·template·compile-time 문서 변경도 거절합니다. 이 manifest의
Git blob hash는 artifact의 SHA-256과 구분해 `sourceHashKind`에 명시합니다. 별도 untracked
source도 허용하지 않으며 출력·cache는 checkout 외부의 고유 scratch 경로를 사용합니다.
소스 checkout이나 기본 feature를 수정하지 않습니다.

각 step은 별도 process group에서 실행하고 private log에 처음부터 기록합니다.
compile/link 기본 상한은 3600초(`--step-timeout`, 30~7200초), disposable Metal producer는
120초(`--producer-timeout`, 1~300초)입니다. timeout/중단이면 소유한 process group에
TERM 후 KILL을 보내고 실제 종료를 기다립니다. `timedOut`·종료 코드와 실패 영수증을
남기며, 부모가 종료한 뒤에도 compiler/device helper 자식이 남지 않게 합니다.

## pixel·수명 계약

실제 MTLDevice·MTLCommandQueue와 새 BGRA8 target을 사용합니다. Native의 producer 완료를
확인한 뒤에만 검증용 CPU readback을 실행합니다. discrete 장치의 managed target은 별도 blit
동기화를 완료한 뒤 읽습니다. Native의 정상 whole-frame readback 수는 계속 0이며, probe가 수행한
검증용 읽기는 별도로 집계합니다.

32×24 viewport의 바깥에 snapshot 페이지를 배치하여 대상 전체를 독립적인 clear-color 대조군으로
사용합니다. opaque BGRA `(48,32,16,255)`와 premultiplied BGRA `(8,16,32,128)`에서 채널 오차는
1 이하여야 합니다. 편집 후 예전 stamp는 `staleFrame`으로 거절하고 초기 seed bytes를 보존해야 합니다.
producer 완료 근거가 없는 실패는 해당 테스트 프로세스에서 target·owner를 유지한 채 실패하며
완료를 추정하여 재사용하지 않습니다.

이 성공은 **Apple SDK compile/link와 bounded Metal producer**의 검증입니다. Flutter의 실제
CVPixelBuffer import/sampling·장시간 표시·consumer GPU fence·pool 재사용·device loss·실물 OS
IME/접근성이나 최종 한컴 비교를 완료한 것으로 확대하지 않습니다. 다음 값은 성공 영수증에서도
계속 false입니다.

- `consumerGpuCompletionVerified`
- `continuousPresentationSupported`
- `flutterConsumerSamplingVerified`

## target 없이 가능한 검사와 현재 경계

```bash
python3 -B bindings/Flutter/macos/tests/sdk_qualification_contract.py
```

이 검사는 독립 premultiplied 색상, 잘못된 pixel·stamp·backend·완료 근거 거절, stale target 보존,
격리된 빌드 출력과 비 Apple 플랫폼 거절을 검사합니다. 임시 Git 저장소의 vendor 변경·
untracked source, architecture별 Cargo target/artifact 경로 및 실제 child timeout·descendant
종료·private log 기록도 검증합니다. Apple SDK를 컴파일하거나 Metal 장치에서
실행한 결과가 아닙니다. 실제 target 환경에서 위 도구를 끝까지 실행해 영수증을 확보하기 전까지
SDK·실물 Metal의 검증은 **미완료**입니다.
