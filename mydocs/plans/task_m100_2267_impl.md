# 구현 계획서 — M100 #2267: macOS Finder Quick Look 확장

수행 계획서: `task_m100_2267.md`

## 1단계 — C ABI 확장

`bindings/Native` 에 Quick Look 확장이 필요로 하는 최소 API 를 추가한다.

```c
typedef struct { uint8_t *data; size_t len; char *error; } RhwpBuffer;

void rhwp_buffer_free(RhwpBuffer buffer);
int32_t rhwp_page_count(const char *input_path);
RhwpBuffer rhwp_render_pdf(const char *input_path, uint32_t first_page,
                           int32_t max_pages, const char *font_dir, int32_t embed_text);
```

- `font_dir` 를 인자로 받는 이유: 코어의 기본 폰트 탐색 경로는 작업디렉터리 상대경로(`ttfs/`)라 샌드박스된 확장 프로세스에서는 절대 잡히지 않는다. 번들 Resources 의 **절대경로**를 넘겨야 한다.
- `embed_text` 를 인자로 받는 이유: #2264 의 메모리 절감을 확장에서만 켠다 (코어 기본값은 `true` 유지).
- 오류는 `error` 문자열로 반환한다. FFI 경계를 넘는 panic 은 unwind 로 잡는다.

**검증**: `cargo test --manifest-path bindings/Native/Cargo.toml` — page_count / 유효 PDF(매직 + trailer) / 범위 초과 / 파일 없음.

> `bindings/Native` 는 워크스페이스 밖이라 CI 가 컴파일하지 않는다. 그래서 이미 컴파일이 깨져 있었다 (#1161 에서 `get_control_image_*_native` 에 `cell_path` 파라미터가 추가됐는데 이 크레이트만 안 따라감). 1단계에서 함께 고친다.

## 2단계 — XCFramework 빌드 검증

`scripts/package-swift-xcframework.sh` 로 XCFramework 를 만들고, Swift 가 실제로 링크·호출할 수 있는지 확인한다.

**검증**: 슬라이스 3종(`ios-arm64`, `ios-arm64-simulator`, `macos-arm64_x86_64`), macOS 슬라이스가 universal, `_rhwp_render_pdf` 심볼 존재, C 프로그램으로 링크해 실제 PDF 산출, 배포 타깃 경고 0.

## 3단계 — Xcode 스캐폴딩

`rhwp-quicklook/` 에 XcodeGen 스펙과 Swift 소스를 만든다.

- 타겟 3개: 호스트 앱 + Preview 확장 + Thumbnail 확장
- 호스트 앱 Info.plist 에 `UTImportedTypeDeclarations` (hwp / hwpx / hml)
- 확장 Info.plist 에 `NSExtensionPointIdentifier` + `QLSupportedContentTypes`
- `ttfs/opensource` 를 확장 번들 Resources 로 포함
- **페이지 상한을 실측으로 확정** — 메모리 한도(120MB)에서 역산

**검증**: 빌드 성공 + 번들 구조 검사 (확장 임베드 / 미해결 `rhwp_` 심볼 0 / 폰트 포함 / 확장 포인트·UTI).

Finder 실제 통합은 코드서명 인증서 부재로 검증 불가 — 작업지시자가 미검증을 감수하기로 결정.

## 4단계 — 문서화·정리

README (빌드·설치·설계 근거·검증 범위), 단계별 보고서, 최종 보고서.
