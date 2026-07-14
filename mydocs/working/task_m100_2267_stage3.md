# 3단계 완료 보고서 — M100 #2267: Xcode 스캐폴딩

## 한 일

`rhwp-quicklook/` 에 XcodeGen 스펙(`project.yml`)과 Swift 소스를 만들었다. 타겟 3개: 호스트 앱 + Preview 확장 + Thumbnail 확장.

`RhwpQuickLook.xcodeproj` 는 생성물이라 `.gitignore` 에 넣었다. `project.yml` 이 유일한 원본이다.

## 페이지 상한을 3으로 정한 근거

확장 프로세스 한도는 약 80MB 경고 / **120MB 강제 종료** / 30초 타임아웃이다. 페이지 수별 최대 RSS 실측 (`embed_text=0`):

| 문서 성격 | 1쪽 | 3쪽 | 5쪽 | 10쪽 |
|---|---|---|---|---|
| 이미지 다수 (8쪽) | 92MB | 94MB | **179MB** | 193MB |
| 비트맵 다수 (20쪽) | 100MB | 102MB | 104MB | 143MB |
| 이미지 중간 (74쪽) | 41MB | 48MB | 50MB | 81MB |

**5페이지는 강제 종료선을 넘는다.** 3페이지가 실측 최대 102MB 로 안전한 상한이다. 상수는 `Shared/RhwpRenderer.swift` 의 `previewPageLimit` 에 근거와 함께 박아뒀다. 늘리려면 재측정해야 한다.

## 설계 결정

- **PDF 데이터 기반 미리보기.** `QLPreviewReply(dataOfContentType: .pdf)` 로 바이트만 넘긴다. 확대·스크롤·페이지 이동 UI 를 Quick Look 이 제공한다.
- **`embed_text = false`.** 폰트 임베드를 끄면 RSS 가 95MB 가량 줄어든다 (#2264). 대가로 미리보기 PDF 에서 텍스트 선택·검색이 안 된다. 코어 기본값은 `true` 로 두고 확장에서만 끈다.
- **폰트 번들 절대경로 전달.** 코어의 기본 폰트 탐색은 작업디렉터리 상대경로(`ttfs/`)라 샌드박스 확장에서는 안 잡힌다. `ttfs/opensource` 를 확장 Resources 에 담고 절대경로를 넘긴다.
- **HWPX 를 `com.pkware.zip-archive` 에 conform 시키지 않음.** conform 시키면 아카이브용 Quick Look 확장이 먼저 가로챌 수 있다.

## 검증 — 번들 구조

빌드 성공(`** BUILD SUCCEEDED **`) 후 산출물 검사:

| 항목 | 결과 |
|------|------|
| 확장 임베드 | `Contents/PlugIns/` 에 `RhwpPreviewExtension.appex`, `RhwpThumbnailExtension.appex` |
| Rust 라이브러리 링크 | 미해결 `rhwp_` 심볼 **0**, `_rhwp_render_pdf` 정의 포함 |
| 폰트 번들링 | `.appex/Contents/Resources/opensource/` 에 NotoSansKR 계열 포함 |
| Preview 확장 | point=`com.apple.quicklook.preview`, class=`RhwpPreviewExtension.PreviewProvider`, UTI 3개 |
| Thumbnail 확장 | point=`com.apple.quicklook.thumbnail`, class=`RhwpThumbnailExtension.ThumbnailProvider`, UTI 3개 |
| 앱 UTI 선언 | `kr.co.hancom.hwp`←hwp, `.hwpx`←hwpx, `.hml`←hml |
| 서명 | `Identifier=com.rhwp.quicklook, Signature=adhoc` |

## 검증되지 않은 것

**Finder 실제 통합.** 이 환경에 코드서명 인증서가 0개다. Ad-hoc 서명으로 빌드는 되지만 Launch Services 등록 → Finder 미리보기 표시 경로는 실행해보지 못했다. 작업지시자가 미검증을 감수하기로 결정하고 진행했다.

Developer ID 인증서가 있는 환경에서 README 의 "설치" 절차(앱을 `/Applications` 에 복사 → 1회 실행 → `pluginkit -m -p com.apple.quicklook.preview` 로 등록 확인)로 확인해야 한다.

## 삽질 기록 — `plutil -extract` 는 `-o -` 없으면 원본을 덮어쓴다

검증 중 빌드된 `Info.plist` 의 `QLSupportedContentTypes` 가 사라진 것처럼 보였다. 원인은 프로젝트가 아니라 **내 검증 명령**이었다.

```bash
plutil -extract NSExtension.NSExtensionAttributes.QLSupportedContentTypes json "$PLIST"   # ← 원본을 덮어씀
plutil -extract NSExtension.NSExtensionAttributes.QLSupportedContentTypes json -o - "$PLIST"  # ← 올바름
```

`-o` 를 생략하면 plutil 이 추출 결과를 **입력 파일에 다시 쓴다.** 소스 plist 는 멀쩡했고 재빌드로 복구됐다. plist 를 읽기만 할 때는 반드시 `-o -` 를 붙일 것.
