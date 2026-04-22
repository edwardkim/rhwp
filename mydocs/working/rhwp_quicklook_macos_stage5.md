# rhwp-macos Quick Look — Stage 5 완료 보고서

## 개요

- 작성 시각: 2026-04-22 14:50 KST
- 대상 계획서: `mydocs/plans/rhwp_quicklook_macos_impl.md`
- 단계: Stage 5 — 앱 번들/등록/썸네일 처리
- 작업 브랜치: `local/task3`

## 수행 내용

1. `ThumbnailExtension` macOS app-extension 타겟을 추가하고 HostApp에 embed하도록 구성했다.
2. Preview와 Thumbnail이 같은 첫 페이지 렌더링 경로를 사용하도록 `Sources/Shared/HwpPageImageRenderer.swift`를 분리했다.
3. `HwpThumbnailProvider`를 추가해 HWP/HWPX 첫 페이지를 Finder 아이콘 썸네일로 렌더링하도록 했다.
4. 50MB 초과 파일은 ThumbnailExtension에서도 파싱하지 않고 fallback 썸네일을 반환하도록 했다.
5. HostApp에 Preview/Thumbnail 등록 상태 표시와 새로고침 버튼을 추가했다.
6. HostApp 자체는 `pluginkit` 상태 확인을 위해 sandbox entitlement를 제거하고, Preview/Thumbnail 확장은 sandbox와 read-only file entitlement를 유지했다.
7. 기존 로고 자산으로 macOS AppIcon placeholder를 구성했다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| XcodeGen 프로젝트 생성 | 통과 |
| HostApp + QLExtension + ThumbnailExtension Debug 빌드 | 통과 |
| Preview 확장 등록 | 통과 (`com.postmelee.rhwpmac.QLExtension`) |
| Thumbnail 확장 등록 | 통과 (`com.postmelee.rhwpmac.ThumbnailExtension`) |
| Finder 아이콘 보기 썸네일 | 통과 (`samples/basic/KTX.hwp` 첫 페이지 썸네일 표시) |
| Thumbnail 생성 시간 | 통과 (`0.675s`, 2초 미만) |
| HostApp 등록 상태 UI | 통과 (Preview/Thumbnail 모두 `Registered`) |
| AppIcon 번들 포함 | 통과 (`Contents/Resources/AppIcon.icns`) |

## 실행 명령

```bash
xcodegen generate

xcodebuild -project RhwpMacOS.xcodeproj \
  -scheme HostApp \
  -configuration Debug \
  CONFIGURATION_BUILD_DIR=/Users/melee/Documents/projects/rhwp-macos/rhwp-macos/build/debug \
  build

qlmanage -r
qlmanage -r cache

/tmp/stage5_thumbnail_check \
  /Users/melee/Documents/projects/rhwp-macos/samples/basic/KTX.hwp \
  /Users/melee/Documents/projects/rhwp-macos/output/stage5-thumbnail/KTX-api-fixed.png
```

## Finder 검증

- Finder에서 `samples/basic` 폴더를 아이콘 보기로 열었다.
- `KTX.hwp`, `KTX-003.hwp` 등 HWP 파일이 기본 파일 아이콘이 아니라 문서 첫 페이지 기반 썸네일로 표시되는 것을 확인했다.
- HostApp 실행 후 등록 상태 UI에서 `Quick Look Preview`와 `Quick Look Thumbnail`이 모두 `Registered`로 표시되는 것을 확인했다.

## 특이사항

- `pluginkit -m -p com.apple.quicklook.thumbnail`은 macOS 26.4.1 환경에서 빈 목록을 반환했지만, `pluginkit -m` 전체 목록과 Finder/QuickLookThumbnailing API 경로에서는 `com.postmelee.rhwpmac.ThumbnailExtension`이 정상 사용되었다.
- `qlmanage -t -s 256 -o ... samples/basic/KTX.hwp`는 extension completion 로그가 빠르게 찍힌 뒤에도 CLI 프로세스가 종료되지 않았다. Stage 5의 썸네일 검증은 실제 Finder 아이콘 보기와 `QLThumbnailGenerator` API 산출물 기준으로 완료했다.
- `QLThumbnailReply` drawing block의 `CGContext`는 scale이 반영된 clip bounds를 제공하므로, `request.maximumSize`가 아니라 `context.boundingBoxOfClipPath` 기준으로 배경과 페이지 이미지를 그리도록 조정했다.

## 다음 단계

Stage 6에서는 패키징/배포 검증, 최종 보고서 작성, 문서 정리와 브랜치 정리를 진행한다.
