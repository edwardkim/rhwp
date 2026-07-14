# rhwp Quick Look

macOS Finder 에서 `.hwp` / `.hwpx` / `.hml` 파일을 스페이스바로 미리보고, 파일 아이콘에 썸네일을 표시하는 Quick Look 확장이다.

## 구성

| 타겟 | 역할 |
|------|------|
| `RhwpQuickLook` (앱) | 확장을 담는 호스트 앱. UTI 를 선언하고 Launch Services 에 확장을 등록시킨다. |
| `RhwpPreviewExtension` | `QLPreviewProvider`. 문서를 PDF 로 렌더해 Quick Look 창에 넘긴다. |
| `RhwpThumbnailExtension` | `QLThumbnailProvider`. 첫 페이지를 그려 Finder 아이콘 썸네일을 만든다. |

렌더는 Rust 코어(`bindings/Native` 의 C ABI)를 XCFramework 로 링크해 수행한다. Swift 쪽에는 렌더 로직이 없다.

```
Finder ──▶ Quick Look 확장 ──▶ rhwp_render_pdf (C ABI)
                                    └─▶ HWP 파서 ─▶ Document IR ─▶ SVG ─▶ PDF
                              ◀── PDF bytes ──┘
```

## 빌드

### 사전 준비

1. **XCFramework 생성** (필수, 먼저 실행):

   ```bash
   ./scripts/package-swift-xcframework.sh
   ```

   `dist/swift/build/RhwpNative.xcframework` 가 만들어진다. Xcode 프로젝트는 이 경로를 직접 참조하므로, 없으면 링크 단계에서 실패한다.

2. **XcodeGen** (프로젝트 파일 생성기):

   ```bash
   brew install xcodegen
   ```

### 빌드

```bash
cd rhwp-quicklook
xcodegen generate                       # project.yml → RhwpQuickLook.xcodeproj
xcodebuild -project RhwpQuickLook.xcodeproj \
           -scheme RhwpQuickLook \
           -configuration Release \
           -destination 'platform=macOS' build
```

`RhwpQuickLook.xcodeproj` 는 생성물이라 커밋하지 않는다. `project.yml` 이 유일한 원본이다.

## 설치

Quick Look 확장은 **앱 번들 안에만 존재할 수 있고**, 앱이 한 번 Launch Services 에 등록되어야 Finder 가 확장을 집어든다.

```bash
# 빌드 산출물을 /Applications 로 복사한 뒤 한 번 실행
cp -R <DerivedData>/Build/Products/Release/RhwpQuickLook.app /Applications/
open /Applications/RhwpQuickLook.app
```

등록 확인:

```bash
pluginkit -m -p com.apple.quicklook.preview   | grep -i rhwp
pluginkit -m -p com.apple.quicklook.thumbnail | grep -i rhwp
```

Finder 에서 `.hwp` 파일을 선택하고 스페이스바.

## 설계 근거

### 왜 PDF 인가

`QLPreviewReply(dataOfContentType: .pdf)` 로 PDF 바이트만 넘기면 확대·스크롤·페이지 이동 UI 를 Quick Look 이 전부 제공한다. 커스텀 뷰(`QLPreviewingController` + NSView)를 만들면 그 UI 를 직접 구현해야 한다.

### 페이지 상한 3

확장 프로세스는 하드 제약을 받는다: 약 **80MB 경고 / 120MB 강제 종료 / 30초 타임아웃**.

1페이지 렌더 시 최대 RSS 실측 (`embed_text=0`):

| 문서 성격 | 1쪽 | 3쪽 | 5쪽 | 10쪽 |
|---|---|---|---|---|
| 이미지 다수 (8쪽) | 92MB | 94MB | **179MB** | 193MB |
| 비트맵 다수 (20쪽) | 100MB | 102MB | 104MB | 143MB |
| 이미지 중간 (74쪽) | 41MB | 48MB | 50MB | 81MB |

**5페이지는 강제 종료선(120MB)을 넘는다.** 3페이지가 실측 최대 102MB 로 안전한 상한이다. 늘리려면 반드시 재측정한다. 상수는 `Shared/RhwpRenderer.swift` 의 `previewPageLimit`.

이 수치는 Task #2263(BinData 지연 로딩)과 #2264(`embed_text` 옵션)를 **둘 다 적용한 뒤**의 값이다. 두 개선 전에는 1페이지 렌더도 강제 종료선을 넘었다.

### `embed_text = false`

PDF 에 텍스트를 폰트로 임베드하면 폰트 서브셋 과정에서 RSS 가 95MB 가량 더 뛴다 (#2264). 미리보기는 시각 표현만 필요하므로 글리프를 path 로 그린다. **대가: 미리보기 PDF 에서 텍스트 선택·검색이 안 된다.** 코어의 기본값은 `true` 이고, 확장에서만 끈다.

### 폰트 번들

코어의 기본 폰트 탐색 경로는 작업디렉터리 상대경로(`ttfs/`)라 샌드박스된 확장 프로세스에서는 잡히지 않는다. `ttfs/opensource` 를 번들 Resources 에 담고 그 **절대경로**를 `rhwp_render_pdf` 에 넘긴다. 안 넘기면 한글이 깨진다.

### UTI

macOS 에는 `.hwp` / `.hwpx` 의 기본 UTI 가 없다. 호스트 앱 `Info.plist` 의 `UTImportedTypeDeclarations` 로 `kr.co.hancom.hwp` / `.hwpx` / `.hml` 을 선언한다.

HWPX 는 ZIP 컨테이너지만 `com.pkware.zip-archive` 에 **conform 시키지 않는다.** conform 시키면 아카이브용 Quick Look 확장이 먼저 가로챌 수 있다.

## 현재 상태 — 검증 범위

**검증된 것** (빌드 산출물 구조 검사):

- Xcode 빌드 성공, 두 확장이 앱 번들 `Contents/PlugIns/` 에 임베드됨
- 확장 바이너리에 미해결 `rhwp_` 심볼 0, `_rhwp_render_pdf` 정의 포함 (Rust 정적 라이브러리 링크 확인)
- 폰트가 `.appex/Contents/Resources/opensource/` 에 포함됨
- 두 확장의 `NSExtensionPointIdentifier` / `NSExtensionPrincipalClass` / `QLSupportedContentTypes`(3개 UTI) 정상
- 앱 Info.plist 의 UTI 선언 정상
- FFI 계층은 별도 테스트 5개 + 벤치로 검증 (실제 PDF 산출, 41~102MB)

**검증되지 않은 것**:

- **Finder 실제 통합.** 이 환경에 코드서명 인증서가 없다. Ad-hoc 서명(`CODE_SIGN_IDENTITY: "-"`)으로 빌드했고, Launch Services 등록 → Finder 미리보기 표시까지의 경로는 실행해보지 못했다. Developer ID 인증서가 있는 환경에서 위 "설치" 절차로 확인해야 한다.
- 30초 타임아웃 안에서의 실제 동작 (FFI 벤치 기준 0.16~1.32초라 여유는 크다).
