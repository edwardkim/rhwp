# rhwp-macos Quick Look — Stage 3 완료 보고서

## 개요

- 작성 시각: 2026-04-22 06:28 KST
- 대상 계획서: `mydocs/plans/rhwp_quicklook_macos_impl.md`
- 단계: Stage 3 — Swift 코어 포팅
- 작업 브랜치: `local/task3`

## 수행 내용

1. `rhwp-ios/Sources/FontFallback.swift`에서 UIKit import를 제거했다.
2. `rhwp-ios/Sources/CGTreeRenderer.swift`에서 UIKit 의존을 CoreGraphics/CoreText/Foundation/ImageIO 조합으로 치환했다.
3. `UIImage(data:)` 기반 이미지 디코딩을 `CGImageSourceCreateWithData` + `CGImageSourceCreateImageAtIndex`로 치환했다.
4. `UIColor` 기반 색상 처리를 `CGColor`와 Core Text 전용 attribute key로 치환했다.
5. 공유 Swift 4종에 AppKit/UIKit 의존이 다시 들어오지 않도록 `rhwp-macos/scripts/check-no-appkit.sh`를 추가했다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| AppKit/UIKit 금지 lint | 통과 |
| 공유 Swift 4종 iOS SDK typecheck | 통과 |
| 공유 Swift 4종 macOS SDK typecheck | 통과 |
| iOS Rust staticlib 빌드 | 통과 (`aarch64-apple-ios-sim`, `aarch64-apple-ios`) |
| iOS Xcode 앱 빌드 (arm64 simulator) | 통과 |
| iOS Xcode 앱 빌드 (iphoneos, signing 제외) | 통과 |
| iOS 기기용 실행 파일 동적 의존성 검사 | 통과 (`librhwp.dylib` 의존 없음) |
| CLI PNG 덤프 | 통과 (HWP 3종) |
| 한글 텍스트 누락 확인 | 통과 (렌더 트리 한글 run + CoreText 글리프 + 비어 있지 않은 PNG 픽셀 검사) |
| 실제 iPhone 설치 | 통과 |
| 실제 iPhone 실행 | 통과 (작업지시자 확인) |

## 실행 명령

```bash
./rhwp-macos/scripts/check-no-appkit.sh

swiftc -typecheck \
  -sdk /Applications/Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS26.4.sdk \
  -target arm64-apple-ios16.0 \
  -module-cache-path /tmp/rhwp-swift-module-cache \
  -Xcc -fmodules-cache-path=/tmp/rhwp-clang-module-cache \
  -import-objc-header rhwp-ios/Sources/rhwp-Bridging-Header.h \
  rhwp-ios/Sources/RhwpDocument.swift \
  rhwp-ios/Sources/RenderTree.swift \
  rhwp-ios/Sources/FontFallback.swift \
  rhwp-ios/Sources/CGTreeRenderer.swift

swiftc -typecheck \
  -sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.4.sdk \
  -target arm64-apple-macos12.0 \
  -module-cache-path /tmp/rhwp-swift-macos-module-cache \
  -Xcc -fmodules-cache-path=/tmp/rhwp-clang-macos-module-cache \
  -import-objc-header rhwp-ios/Sources/rhwp-Bridging-Header.h \
  rhwp-ios/Sources/RhwpDocument.swift \
  rhwp-ios/Sources/RenderTree.swift \
  rhwp-ios/Sources/FontFallback.swift \
  rhwp-ios/Sources/CGTreeRenderer.swift

cargo build --release --lib --target aarch64-apple-ios-sim
cargo build --release --lib --target aarch64-apple-ios

xcodebuild -project rhwp-ios/AlHangeul.xcodeproj \
  -target AlHangeul \
  -configuration Debug \
  -sdk iphoneos \
  CODE_SIGNING_ALLOWED=NO \
  build

otool -L rhwp-ios/build/Debug-iphoneos/알한글.app/알한글

xcodebuild -project rhwp-ios/AlHangeul.xcodeproj \
  -target AlHangeul \
  -configuration Debug \
  -sdk iphonesimulator \
  CODE_SIGNING_ALLOWED=NO \
  ARCHS=arm64 \
  ONLY_ACTIVE_ARCH=YES \
  build

./rhwp-macos/scripts/validate-stage3-render.sh

xcrun devicectl list devices

xcodebuild -project rhwp-ios/AlHangeul.xcodeproj \
  -scheme AlHangeul \
  -configuration Debug \
  -destination id=5CBD2650-302F-5912-8453-DC6611304DEA \
  -derivedDataPath output/stage3-device-derived \
  build

xcrun devicectl device install app \
  --device 5CBD2650-302F-5912-8453-DC6611304DEA \
  output/stage3-device-derived/Build/Products/Debug-iphoneos/알한글.app

xcrun devicectl device process launch \
  --device 5CBD2650-302F-5912-8453-DC6611304DEA \
  com.postmelee.alhangeul
```

## 렌더 검증 결과

`validate-stage3-render.sh`는 공유 Swift 4종과 macOS universal `librhwp.a`를 묶은 CLI를 빌드한 뒤, 첫 페이지를 `CGBitmapContext`에 렌더링하고 PNG로 저장한다. 동시에 렌더 트리에서 한글 `TextRun`을 수집하고, CoreText 폰트 폴백으로 한글 글리프 존재 여부를 확인한다.

| 샘플 | PNG | 텍스트 run | 한글 run | 한글 스칼라 | 비흰색 픽셀 |
|------|-----|------------|----------|-------------|-------------|
| `samples/basic/KTX.hwp` | `output/stage3-render/KTX-page1.png` | 435 | 76 | 209 | 450455 |
| `samples/basic/request.hwp` | `output/stage3-render/request-page1.png` | 104 | 36 | 309 | 54724 |
| `samples/exam_kor.hwp` | `output/stage3-render/exam_kor-page1.png` | 69 | 51 | 940 | 96464 |

## 특이사항

- `xcodegen`은 현재 로컬 PATH에 없어 기존 `rhwp-ios/AlHangeul.xcodeproj`로 검증했다.
- 최초 검증 시에는 로컬 Xcode 설정과 서명 설정 문제로 전체 앱 빌드가 완료되지 않았으나, 작업지시자의 iOS 서명 설정 변경 후 arm64 시뮬레이터와 `iphoneos` 빌드를 재검증했다.
- 실제 iPhone 실행 시 앱이 즉시 종료되는 문제가 확인되었다. 원인은 `-lrhwp` 링크가 `librhwp.a` 대신 같은 Rust 산출물 폴더의 `librhwp.dylib`를 선택해, 앱 실행 파일이 로컬 절대경로의 dylib에 동적 의존한 것이다.
- `rhwp-ios/project.yml`과 `rhwp-ios/AlHangeul.xcodeproj/project.pbxproj`의 링크 설정을 SDK별 `librhwp.a` 직접 지정 방식으로 변경했다. 이후 `otool -L`에서 `librhwp.dylib` 의존이 사라진 것을 확인했다.
- `xcrun devicectl`에서 실제 iPhone `Taegyu`가 연결된 것을 확인했고, 서명된 Debug 앱 설치까지 성공했다.
- 최초 실제 iPhone 실행 시에는 기기 보안 정책으로 차단되었다. 에러는 `invalid code signature, inadequate entitlements or its profile has not been explicitly trusted by the user`였으며, 코드 크래시가 아니라 iPhone 설정에서 개발자 프로파일 신뢰가 필요한 상태였다.
- 작업지시자가 개발자 앱 신뢰 설정 후 실제 iPhone 실행이 정상 동작함을 확인했다.
- Stage 3 계획서의 공식 검증 기준은 모두 완료되었다.

## 다음 단계

Stage 4에서는 `rhwp-macos/project.yml`, HostApp, Quick Look Preview Extension 최소 동작을 구현한다. 이 단계에서 공유 Swift 4종을 macOS extension 타겟에 연결하고 Finder Quick Look 첫 페이지 미리보기를 검증한다.
