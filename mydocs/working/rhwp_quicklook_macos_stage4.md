# rhwp-macos Quick Look — Stage 4 완료 보고서

## 개요

- 작성 시각: 2026-04-22 07:45 KST
- 대상 계획서: `mydocs/plans/rhwp_quicklook_macos_impl.md`
- 단계: Stage 4 — macOS Quick Look Extension 최소 구현
- 작업 브랜치: `local/task3`

## 수행 내용

1. `rhwp-macos/project.yml`을 추가해 XcodeGen 기반 macOS 프로젝트를 정의했다.
2. `HostApp` macOS 앱 타겟을 추가하고 Quick Look 확장을 embed하도록 구성했다.
3. `QLExtension` macOS app-extension 타겟을 추가하고 Stage 3에서 검증한 공유 Swift 렌더러 4종과 `Rhwp.xcframework`를 연결했다.
4. `HwpPreviewProvider`를 구현해 Finder Quick Look에서 첫 페이지를 PNG preview로 반환하도록 했다.
5. 50MB 초과 파일은 파싱 전에 plain text fallback preview를 반환하도록 했다.
6. macOS 앱 확장이 `pluginkit` discovery에서 제외되지 않도록 HostApp/Extension에 sandbox 및 read-only file entitlement를 추가했다.
7. Hancom Viewer가 설치된 환경에서 Finder가 사용하는 UTI(`com.haansoft.hancomofficeviewer.mac.hwp`, `.hwpx`)를 지원 목록에 추가했다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| XcodeGen 프로젝트 생성 | 통과 |
| macOS HostApp + QLExtension Debug 빌드 | 통과 |
| 확장 sandbox entitlement | 통과 |
| `pluginkit` Quick Look preview 등록 | 통과 (`com.postmelee.rhwpmac.QLExtension`) |
| Rust 라이브러리 정적 링크 확인 | 통과 (`librhwp.dylib` 의존 없음) |
| Finder 훑어보기 첫 페이지 렌더 | 통과 (`samples/basic/KTX.hwp`) |
| 50MB 초과 fallback | 통과 (`output/stage4-quicklook/large-over-50mb.hwp`) |
| 메모리 관측치 | 통과 (확장 RSS 50,368KB, 200MB 미만) |

## 실행 명령

```bash
xcodegen generate

xcodebuild -project RhwpMacOS.xcodeproj \
  -scheme HostApp \
  -configuration Debug \
  CONFIGURATION_BUILD_DIR=/Users/melee/Documents/projects/rhwp-macos/rhwp-macos/build/debug \
  build

pluginkit -m -p com.apple.quicklook.preview

otool -L \
  "rhwp-macos/build/debug/HWP Quick Look.app/Contents/PlugIns/HWP Quick Look Preview.appex/Contents/MacOS/HWP Quick Look Preview.debug.dylib"

mdls -name kMDItemContentType -name kMDItemContentTypeTree \
  "/Users/melee/Documents/projects/rhwp-macos/samples/basic/KTX.hwp"

mkfile 51m output/stage4-quicklook/large-over-50mb.hwp

ps -o pid,rss,command -p 94555
```

## Finder 검증

- `samples/basic/KTX.hwp`를 Finder에서 선택한 뒤 `파일 > 훑어보기`를 실행했다.
- 최초에는 Hancom Viewer UTI만 붙어 있어 기본 파일 아이콘 preview가 표시되었다.
- HostApp/Extension plist에 `com.haansoft.hancomofficeviewer.mac.hwp`와 `com.haansoft.hancomofficeviewer.mac.hwpx`를 추가한 뒤 Finder Quick Look 창에서 KTX 첫 페이지 이미지가 표시되는 것을 확인했다.
- `output/stage4-quicklook/large-over-50mb.hwp`는 53.5MB 파일로 생성했고, Finder preview에서 `The file is larger than 50 MB.` fallback 문구가 표시되는 것을 확인했다.

## 특이사항

- macOS app extension은 sandbox entitlement가 없으면 `pluginkit` discovery에 나타나지 않았다. `com.apple.security.app-sandbox`와 `com.apple.security.files.user-selected.read-only`를 추가한 뒤 등록 목록에 정상 표시되었다.
- Xcode 26 템플릿 기준 data-based preview provider는 `func providePreview(for:) async throws -> QLPreviewReply` 형태를 사용한다. callback 형태에서 템플릿 방식으로 변경했다.
- `qlmanage -p -z samples/basic/KTX.hwp`는 Finder에서 정상 동작하는 동일 확장에 대해 `EXConcreteExtension` 내부 `NSInvalidArgumentException`으로 종료되었다. 따라서 Stage 4의 사용자-facing 검증은 Finder 실제 훑어보기 경로와 Computer Use 화면 확인을 기준으로 삼았다.
- 메모리는 Finder preview 후 `ps`로 확장 프로세스 RSS 50,368KB를 확인했다. Instruments 기반 peak 측정은 이번 단계에서 수행하지 않았다.

## 다음 단계

Stage 5에서는 앱 아이콘, 설치/개발 문서, 최종 보고서, 브랜치 정리를 진행한다. 필요하면 `qlmanage -p -z` 내부 예외는 별도 트러블슈팅 항목으로 분리한다.
