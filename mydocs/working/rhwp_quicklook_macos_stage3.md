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
| 전체 iOS Xcode 앱 빌드 | 환경 문제로 미완료 |

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
```

## 특이사항

- `xcodegen`은 현재 로컬 PATH에 없어 기존 `rhwp-ios/AlHangeul.xcodeproj`로 검증했다.
- scheme 기반 `xcodebuild`는 destination 계산 단계에서 실패했다.
- target 직접 빌드는 Swift 단계 이전의 asset catalog 단계에서 실패했다. 원인은 `iphonesimulator` SDK build(`23E252`)와 설치된 simulator runtime build(`22D8075`, `23D8133`, `23E244`) 불일치다.
- 따라서 전체 앱 빌드 통과 대신 공유 Swift 4종을 iOS/macOS SDK 각각으로 `swiftc -typecheck`하여 Stage 3의 코드 변경 범위를 검증했다.
- Stage 3 계획의 CLI PNG 덤프 검증 도구는 아직 존재하지 않아 수행하지 못했다. 이 검증은 Stage 4의 macOS 프로젝트/호스트 앱 진입점 작성 후 이어서 수행한다.

## 다음 단계

Stage 4에서는 `rhwp-macos/project.yml`, HostApp, Quick Look Preview Extension 최소 동작을 구현한다. 이 단계에서 공유 Swift 4종을 macOS extension 타겟에 연결하고, Stage 3에서 남은 실제 렌더 출력 검증을 함께 진행한다.
