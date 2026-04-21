# rhwp-macos Quick Look — Stage 2 완료 보고서

## 개요

- 작성 시각: 2026-04-22 05:51 KST
- 대상 계획서: `mydocs/plans/rhwp_quicklook_macos_impl.md`
- 단계: Stage 2 — Rust macOS 빌드 파이프라인(XCFramework)
- 작업 브랜치: `macos/devel`

## 수행 내용

1. `rhwp-macos/scripts/build-rust-macos.sh`를 추가했다.
2. `aarch64-apple-darwin`, `x86_64-apple-darwin` 타겟을 release staticlib로 빌드하도록 구성했다.
3. `xcrun lipo`로 universal `librhwp.a`를 생성하도록 구성했다.
4. `cbindgen`으로 생성한 헤더의 `rhwp_` FFI 심볼 8종과 `RhwpPageSize` 필드를 검증하도록 구성했다.
5. `xcodebuild -create-xcframework`로 `Rhwp.xcframework`를 생성하도록 구성했다.

## 검증 결과

| 항목 | 결과 |
|------|------|
| `Rhwp.xcframework` 생성 | 통과 |
| universal binary 아키텍처 | `x86_64 arm64` 확인 |
| `_rhwp_` exported symbol | 8종 확인 |
| FFI 헤더 심볼 검증 | 8종 동일 확인 |
| 산출물 크기 | `librhwp.a` 97M, `Rhwp.xcframework` 97M |

확인된 FFI 심볼:

```text
rhwp_close
rhwp_free_string
rhwp_image_data
rhwp_open
rhwp_page_count
rhwp_page_size
rhwp_render_page_svg
rhwp_render_page_tree
```

## 실행 명령

```bash
./rhwp-macos/scripts/build-rust-macos.sh
xcrun lipo -info rhwp-macos/Frameworks/universal/librhwp.a
nm -gU rhwp-macos/Frameworks/universal/librhwp.a | rg ' _rhwp_'
```

## 특이사항

- 최초 실행 시 `cbindgen`과 `x86_64-apple-darwin` Rust 타겟이 없어 설치 후 진행했다.
- 샌드박스 내부 첫 빌드는 crates.io DNS 제한으로 실패했으며, 네트워크 허용 후 정상 빌드했다.
- `xcodebuild -create-xcframework` 실행 중 CoreSimulatorService 관련 경고가 출력되었지만, macOS XCFramework 생성은 성공했다.
- `rhwp-macos/Frameworks/`는 `.gitignore` 대상이므로 산출물은 Git에 포함하지 않는다.

## 다음 단계

Stage 3에서는 공유 Swift 4종의 UIKit 의존 제거와 iOS 빌드 검증을 진행한다. 공유 소스 수정이 포함되므로 작업지시자 승인 후 시작한다.
