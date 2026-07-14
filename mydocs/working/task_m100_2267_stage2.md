# 2단계 완료 보고서 — M100 #2267: XCFramework 빌드 검증

## 검증 결과

`scripts/package-swift-xcframework.sh` 산출물 `dist/swift/build/RhwpNative.xcframework`:

| 항목 | 결과 |
|------|------|
| 슬라이스 | `ios-arm64`, `ios-arm64-simulator`, `macos-arm64_x86_64` (3종) |
| macOS 슬라이스 | universal (x86_64 + arm64) |
| `_rhwp_render_pdf` 심볼 | 존재 |
| C 프로그램 링크 (`-mmacosx-version-min=12.0`) | 경고 0 |
| 실제 렌더 | 유효 PDF 산출, 최대 RSS 48~102MB |

## 고친 결함 — 배포 타깃이 고정되어 있지 않았다

패키징 스크립트가 배포 타깃을 지정하지 않아, 오브젝트가 **빌드 머신의 macOS 버전(26.2)** 으로 빌드됐다. 반면 `Package.swift` 는 macOS 12 를 선언한다. 결과적으로 오래된 배포 타깃의 소비자가 링크하면:

```
ld: warning: object file was built for newer 'macOS' version (26.2) than being linked (12.0)
```

스크립트에서 배포 타깃을 명시적으로 고정했다:

```bash
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-12.0}"
export IPHONEOS_DEPLOYMENT_TARGET="${IPHONEOS_DEPLOYMENT_TARGET:-13.0}"
```

**첫 수정 후에도 경고 4개가 남았다.** 원인은 cargo 가 `blake3` 의 cc 컴파일 오브젝트를 **캐시**하고 있어서였다 — 환경변수를 바꿔도 재컴파일하지 않는다. 타깃별로 `cargo clean -p blake3 --release --target <triple>` 후 재패키징하니 경고 0.
