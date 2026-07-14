# 최종 결과 보고서 — M100 #2267: macOS Finder Quick Look 미리보기·썸네일 확장

## 결론

Finder 에서 `.hwp` / `.hwpx` / `.hml` 을 스페이스바로 미리보고 아이콘 썸네일을 표시하는 Quick Look 확장을 구현했다. **빌드·링크·번들 구조는 검증했고, Finder 실제 통합은 코드서명 인증서 부재로 검증하지 못했다.**

## 산출물

| 경로 | 내용 |
|------|------|
| `bindings/Native/src/lib.rs` | C ABI: `rhwp_page_count`, `rhwp_render_pdf`, `rhwp_buffer_free` |
| `bindings/swift/` | C 헤더 + Swift 래퍼 |
| `scripts/package-swift-xcframework.sh` | 배포 타깃 고정 |
| `rhwp-quicklook/` | XcodeGen 스펙 + Swift 소스 (앱 1 + 확장 2) + README |

## 아키텍처

```
Finder ──▶ Quick Look 확장 (Swift) ──▶ rhwp_render_pdf (C ABI)
                                            └─▶ HWP 파서 ─▶ Document IR ─▶ SVG ─▶ PDF
                                    ◀── PDF bytes ──┘
```

Swift 쪽에 렌더 로직은 없다. `QLPreviewReply(dataOfContentType: .pdf)` 로 PDF 바이트만 넘기면 확대·스크롤·페이지 이동 UI 는 Quick Look 이 제공한다.

## 이 타스크의 전제 — 메모리가 관문이었다

Quick Look 확장은 약 **80MB 경고 / 120MB 강제 종료 / 30초 타임아웃**을 받는다. 0단계 스파이크 시점에는 1페이지 렌더조차 이 선을 넘어서, PDF 아키텍처 자체가 성립하지 않았다.

선행 타스크 둘로 관문을 통과했다:

| 타스크 | 개선 | 효과 |
|--------|------|------|
| #2263 | BinData 지연 로딩 | 파싱·레이아웃 최대 RSS 244MB → **49MB** |
| #2264 | PDF `embed_text` 옵션 | `svg2pdf::to_chunk` 최대 RSS 164MB → **69MB** |

두 개선 후 1페이지 렌더 최대 RSS: **41 / 42 / 80 / 88 / 99 MB** — 전부 강제 종료선 아래. 이 수치가 없었으면 이 타스크는 불가능했다.

## 실측으로 정한 것

**페이지 상한 = 3.** 페이지 수별 최대 RSS:

| 문서 성격 | 1쪽 | 3쪽 | 5쪽 | 10쪽 |
|---|---|---|---|---|
| 이미지 다수 (8쪽) | 92MB | 94MB | **179MB** | 193MB |
| 비트맵 다수 (20쪽) | 100MB | 102MB | 104MB | 143MB |
| 이미지 중간 (74쪽) | 41MB | 48MB | 50MB | 81MB |

5페이지는 강제 종료선을 넘는다. 3페이지가 실측 최대 102MB 로 안전한 상한.

## 검증

| 계층 | 방법 | 결과 |
|------|------|------|
| FFI | 단위 테스트 5개 | 통과 (유효 PDF 매직+trailer, 범위 초과, 파일 없음) |
| FFI 성능 | 벤치 | 1페이지 41~100MB / 0.16~1.32초 |
| XCFramework | 슬라이스·심볼·링크 | 3슬라이스, macOS universal, 링크 경고 0, 실제 PDF 산출 |
| 앱 번들 | 구조 검사 | 확장 임베드 / 미해결 `rhwp_` 심볼 0 / 폰트 포함 / 확장 포인트·UTI 정상 |

## 검증되지 않은 것 — 정직한 고지

**Finder 실제 통합.** 이 환경에 코드서명 인증서가 0개다. Ad-hoc 서명으로 빌드는 되지만, Launch Services 등록 → Finder 미리보기 표시까지는 실행해보지 못했다. 작업지시자 승인 하에 미검증을 감수하고 진행했다.

Developer ID 인증서가 있는 환경에서 `rhwp-quicklook/README.md` 의 설치 절차로 확인해야 한다.

## 알려진 트레이드오프

- **미리보기 PDF 에서 텍스트 선택·검색이 안 된다.** `embed_text=false` 로 글리프를 path 로 그리기 때문. 메모리 한도를 지키기 위한 의도적 선택이며, 코어 기본값은 `true` 로 유지했다.
- 미리보기는 앞쪽 3페이지만 보여준다.

## 후속 권고

1. **CI 에 `cargo check --manifest-path bindings/Native/Cargo.toml` 추가.** `bindings/Native` 는 워크스페이스 밖이라 CI 가 컴파일하지 않고, 그래서 이 타스크 착수 시점에 **이미 컴파일이 깨져 있었다** (#1161 의 시그니처 변경을 따라가지 못함). 같은 일이 또 일어난다. 별도 이슈 등록 필요.
2. 인증서 확보 후 Finder 통합 검증 → 결과에 따라 후속 이슈.
3. `embed_text=true` 상태로도 한도 안에 들어가는 문서 유형을 판별할 수 있으면 텍스트 선택을 살릴 여지가 있다 (조건부 활성화).
