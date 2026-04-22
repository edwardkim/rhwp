# rhwp-macos Quick Look — Stage 6 완료 보고서

## 개요

- 작성 시각: 2026-04-22 15:16 KST
- 대상 계획서: `mydocs/plans/rhwp_quicklook_macos_impl.md`
- 단계: Stage 6 — macOS HostApp Viewer parity
- 작업 브랜치: `local/task3`

## 배경

Stage 6 릴리스 패키징 전에 upstream `edwardkim/rhwp@ios/devel`의 iOS 앱 기능 수준을 재검토했다. iOS 앱은 현재 편집/저장 앱이 아니라 HWP/HWPX Viewer이며, Swift UI와 iOS FFI에는 편집 API가 연결되어 있지 않다. 이에 따라 macOS HostApp 목표를 iOS Viewer parity로 재정의했다.

## 수행 내용

1. 구현 계획서를 갱신해 기존 Stage 6을 HostApp Viewer 구현으로 재정의하고, 릴리스 패키징은 Stage 7로 이동했다.
2. HostApp 타깃이 공유 렌더링 소스와 `Rhwp.xcframework`를 직접 링크하도록 `project.yml`을 수정했다.
3. HostApp 코드를 앱 진입점, View, Store, Service, Support 파일로 분리했다.
4. 앱 실행 시 번들 `sample.hwpx`를 자동 로드하도록 했다.
5. `NSOpenPanel` 기반 HWP/HWPX 열기 기능을 추가했다.
6. Finder/터미널에서 HWP/HWPX 파일을 앱으로 열 수 있도록 `NSApplicationDelegate` 파일 열기 경로를 추가했다.
7. 다중 페이지 세로 스크롤, lazy page render/cache, 확대/축소/실제 크기, 현재 페이지/전체 페이지 표시를 구현했다.
8. Stage 5의 Preview/Thumbnail 등록 상태 UI를 사이드바에 유지했다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| XcodeGen 프로젝트 생성 | 통과 |
| HostApp + QLExtension + ThumbnailExtension Debug 빌드 | 통과 |
| 앱 실행 시 `sample.hwpx` 자동 로드 | 통과 (`66`쪽 표시) |
| `samples/basic/KTX.hwp` 열기 | 통과 (`open -a ... KTX.hwp`, `1`쪽 표시) |
| 다중 페이지 스크롤 | 통과 |
| zoom slider / zoom in / actual size | 통과 |
| 현재 페이지/전체 페이지 표시 | 통과 |
| Preview/Thumbnail 등록 상태 유지 | 통과 |
| ThumbnailExtension 회귀 확인 | 통과 (`0.048s`) |

## 실행 명령

```bash
xcodegen generate

xcodebuild -project RhwpMacOS.xcodeproj \
  -scheme HostApp \
  -configuration Debug \
  CONFIGURATION_BUILD_DIR=/Users/melee/Documents/projects/rhwp-macos/rhwp-macos/build/debug \
  build

open -a /Users/melee/Documents/projects/rhwp-macos/rhwp-macos/build/debug/HWP\ Quick\ Look.app \
  /Users/melee/Documents/projects/rhwp-macos/samples/basic/KTX.hwp

pluginkit -m | grep 'com.postmelee.rhwpmac'

/tmp/stage5_thumbnail_check \
  /Users/melee/Documents/projects/rhwp-macos/samples/basic/KTX.hwp \
  /Users/melee/Documents/projects/rhwp-macos/output/stage5-thumbnail/KTX-stage6-regression.png
```

## 특이사항

- macOS 최소 지원 버전이 12.0이므로 `NavigationSplitView`를 사용하지 않고 수동 split layout으로 구현했다.
- Open panel은 사용자 직접 선택 경로로 남기고, 자동 검증은 `NSApplicationDelegate` 파일 열기 경로를 통해 `KTX.hwp`를 앱에 전달했다.
- Rust core에는 편집/직렬화 기능이 있으나 iOS FFI/Swift UI에는 아직 연결되어 있지 않다. macOS 편집 기능은 Stage 6 범위가 아니라 별도 이슈로 분리하는 것이 맞다.

## 다음 단계

Stage 7에서 릴리스 패키징, README, Homebrew cask 초안, 최종 보고서 작성을 진행한다.
