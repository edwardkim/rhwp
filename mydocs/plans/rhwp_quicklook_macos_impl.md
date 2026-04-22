# rhwp-macos — Quick Look macOS 확장 구현 계획서

## 메타데이터

| 항목 | 값 |
|------|-----|
| 프로젝트 성격 | **포크 내 디렉토리 추가** (`postmelee/rhwp` fork, macOS 통합 기준 `macos/devel`) |
| 앱 번들 표시명 | "HWP Quick Look" (잠정) |
| 배포 목표 | Homebrew cask (`.app` 배포) |
| 최종 사용자 기능 | ① Finder Quick Look (스페이스바) ② Finder 썸네일 ③ macOS HostApp 문서 Viewer |
| 개발 전제 | 무료 Apple ID로 로컬 본인 Mac 테스트까지 완결. 공증/배포는 별도 후속 트랙 |
| 업스트림 고정 SHA | `34b6e20af37209b0df943acd30a67afdf6a84a2b` (edwardkim/rhwp@ios/devel, 2026-04-20 07:04 UTC) |
| 업스트림 라이선스 | MIT (Copyright (c) 2025-2026 Edward Kim) |
| 최소 macOS 버전 | 12.0 (Monterey) — `QLPreviewProvider` 요구 |
| Xcode 최소 버전 | 15 이상 |
| Rust 타겟 | `aarch64-apple-darwin` + `x86_64-apple-darwin` (universal) |
| **포크 저장소 경로** | `/Users/melee/Documents/projects/forks/rhwp` (메인 worktree, `devel` 브랜치) |
| **macOS worktree 경로** | `/Users/melee/Documents/projects/rhwp-macos` (`local/task3` 작업 브랜치) |
| **포크 리모트** | `origin` = `postmelee/rhwp` / `upstream` = `edwardkim/rhwp` |
| **통합 기준 브랜치** | `macos/devel` (origin의 macOS 개발 통합 브랜치) |
| **현재 작업 이슈** | `postmelee/rhwp#3` — <https://github.com/postmelee/rhwp/issues/3> |
| **현재 작업 브랜치** | `local/task3` (from `macos/devel`) |
| **번들 ID 접두사** | `com.postmelee.rhwpmac` |
| **UTI 네임스페이스** | Exporter: `com.postmelee.rhwpmac.hwp[x]` / Importer 참조: `com.hancom.hwp[x]` |
| 예상 총 공수 | **8.5~14일** (단독 작업자 기준) |

## 포크 기반 + git worktree 전략 요약

포크 내 macOS 개발 통합 브랜치인 `macos/devel`을 기준으로 별도 worktree를 운용한다. 실제 기능 구현은 GitHub Issue 번호에 맞춘 `local/task{번호}` 브랜치에서 진행하고, 현재 Quick Look 구현 작업은 `postmelee/rhwp#3` 기반 `local/task3`에서 진행한다.

```
forks/rhwp/           ← 메인 worktree (devel) — 다른 기여 (PR, 버그 수정 등)
rhwp-macos/           ← macOS worktree (local/task3) — issue #3 작업

origin/macos/devel    ← macOS 개발 통합 기준
local/task3           ← issue #3 작업 브랜치 (macos/devel에서 분기)
```

**장점**:
1. **코드 중복 제거** — vendoring 없이 원본 하나만 관리
2. **iOS 빌드 즉시 검증** — `CGTreeRenderer.swift` UIKit 제거 후 iOS 빌드 검증 가능
3. **업스트림 PR 직결** — 작업 완료 후 그대로 PR 제출 가능
4. **작업 단위 분리** — `macos/devel`을 깨끗한 기준으로 유지하고 issue별 `local/task{번호}`에 커밋 축적
5. **다른 기여와 병행** — `devel` 메인 worktree와 macOS worktree를 stash 없이 동시 진행

## 선행 조사 결과

### 1. iOS 렌더 파이프라인은 파일시스템 독립

`rhwp_render_page_tree` FFI 경로는 파일시스템을 건드리지 않는다. 샌드박스 문제 없음. 번들에 `ttfs/` 포함 불필요.

### 2. 폰트 페인팅은 Swift/CoreText 책임

`FontFallback.swift`가 HWP 폰트명 → 시스템 폰트명(`AppleMyungjo`, `AppleSDGothicNeo-Regular` 등) 매핑. `CTFontCreateWithName`으로 시스템 폰트만 사용. macOS 시스템 폰트는 iOS와 대부분 동일.

### 3. FFI는 상태 없는 호출 패턴

`rhwp_open(data, len)` → 핸들. 파일 경로 인자 없음. 폰트 탐색 경로 설정자 FFI 없음.

### 4. FFI 표면 — 8종 확정 (2026-04-22 재검증 완료)

| FFI 함수 | 용도 |
|---------|------|
| `rhwp_open(ptr, len) → RhwpHandle*` | 파싱 + 핸들 생성 |
| `rhwp_close(handle)` | 핸들 해제 |
| `rhwp_page_count(handle) → uint32_t` | 총 페이지 수 |
| `rhwp_page_size(handle, page) → RhwpPageSize` | 페이지 크기 (pt) |
| `rhwp_render_page_svg(handle, page) → char*` | 페이지 SVG (디버그용) |
| `rhwp_render_page_tree(handle, page) → char*` | 렌더 트리 JSON (**QL 핵심 경로**) |
| `rhwp_image_data(handle, bin_data_id, &len) → uint8_t*` | 이미지 바이너리 (1-indexed) |
| `rhwp_free_string(char*)` | C 문자열 해제 |

### 5. 포크 인프라 현황 (ios/devel 브랜치)

| 항목 | 상태 |
|------|------|
| `Cargo.toml` crate-type | `["cdylib", "rlib", "staticlib"]` ← **staticlib 이미 포함** |
| `cbindgen.toml` | ✅ 루트에 존재 |
| `rhwp-ios/project.yml` | ✅ XcodeGen 기반 iOS 프로젝트 |
| `rhwp-ios/Sources/` 공유 4종 | `RhwpDocument.swift`, `RenderTree.swift`, `FontFallback.swift`, `CGTreeRenderer.swift` (19,196B) |
| `rhwp-ios/Resources/sample.hwpx` | ✅ 번들 샘플 존재 |

### 6. 업스트림 iOS 앱 기능 수준 (2026-04-22 추가 재검토)

`edwardkim/rhwp@ios/devel`의 iOS 앱은 현재 **편집 앱이 아니라 Viewer 앱**이다.

근거:
- Swift 앱은 `UIDocumentPickerViewController(forOpeningContentTypes:)`로 HWP/HWPX 파일을 여는 경로만 제공한다.
- `DocumentViewModel`은 `loadDocument`, `loadSampleFromBundle`, `loadPage`, `unloadPage` 중심이며 편집 상태, dirty flag, 저장/export UI가 없다.
- `src/ios_ffi.rs`가 iOS로 노출하는 C ABI는 `rhwp_open`, `rhwp_page_count`, `rhwp_page_size`, `rhwp_render_page_svg`, `rhwp_render_page_tree`, `rhwp_image_data`, `rhwp_close` 등 읽기/렌더링 함수뿐이다.
- Rust core에는 `insert_text_native`, `delete_text_native`, `export_hwp_native`, `export_hwpx_native`, `convert_to_editable_native` 같은 편집/직렬화 기능이 있으나, 현재 iOS FFI와 Swift UI에는 연결되어 있지 않다.

따라서 macOS HostApp의 기능 parity 기준은 다음으로 정의한다.

| 기능 | iOS upstream | macOS 목표 |
|------|-------------|------------|
| 번들 샘플 자동 로드 | 지원 | 지원 |
| 로컬 HWP/HWPX 열기 | 지원 | 지원 (`NSOpenPanel`) |
| 다중 페이지 렌더링 | 지원 | 지원 |
| 페이지 lazy load/cache | 지원 | 지원 |
| 팬/스크롤 | 지원 | 지원 |
| 확대/축소 | 지원 | 지원 |
| 현재/전체 페이지 표시 | 지원 | 지원 |
| 텍스트/개체 편집 | 미지원 | 미지원 (별도 이슈) |
| 저장/export | 미지원 | 미지원 (별도 이슈) |

## 아키텍처 개요

```
postmelee/rhwp (포크, macOS 기준 macos/devel / 작업 local/task3)
│
├── Cargo.toml                    ← staticlib 이미 포함. 변경 없음
├── cbindgen.toml                 ← 이미 존재. 변경 없음
├── src/                          ← Rust 코어 파서/렌더러 + ios_ffi. 변경 없음
│
├── rhwp-ios/                     ← iOS 앱 (기존, 구조 변경 없음)
│   ├── project.yml
│   └── Sources/
│       ├── RhwpDocument.swift    ← FFI 래퍼 (공유, 변경 없음)
│       ├── RenderTree.swift      ← JSON 모델 (공유, 변경 없음)
│       ├── FontFallback.swift    ← 폰트 매핑 (공유, import UIKit 제거)
│       ├── CGTreeRenderer.swift  ← 렌더러 (공유, UIKit → CG/CT 치환)
│       ├── rhwp.h                ← FFI 헤더 (공유)
│       ├── rhwp-Bridging-Header.h
│       ├── AlHangeulApp.swift    ← iOS 전용
│       ├── ContentView.swift     ← iOS 전용
│       ├── DocumentView.swift    ← iOS 전용
│       ├── DocumentViewModel.swift ← iOS 전용
│       ├── DocumentPickerView.swift ← iOS 전용
│       ├── PagedScrollView.swift ← iOS 전용
│       ├── PageCanvasView.swift  ← iOS 전용
│       └── Info.plist
│
├── rhwp-macos/                   ← ★ 신규 추가
│   ├── project.yml               ← XcodeGen (macOS 전용)
│   ├── Sources/
│   │   ├── HostApp/
│   │   │   ├── HostApp.swift
│   │   │   └── Resources/sample.hwpx
│   │   ├── QLExtension/
│   │   │   ├── HwpPreviewProvider.swift
│   │   │   └── Info.plist
│   │   └── ThumbnailExtension/
│   │       ├── HwpThumbnailProvider.swift
│   │       └── Info.plist
│   ├── Frameworks/               ← 빌드 산출물 (.gitignore)
│   └── scripts/
│       ├── build-rust-macos.sh
│       └── check-no-appkit.sh
│
└── ...
```

### 데이터 플로우

```
.hwp/.hwpx 파일 (Finder 선택)
    ↓ QLPreviewProvider.providePreview(for: request)
    ↓ [크기 게이트] data.count < MAX_FILE_SIZE 검사
    ↓ Data(contentsOf: request.fileURL)
    ↓ RhwpDocument(data:) → rhwp_open (FFI)
    ↓ pageSize(at: 0) → rhwp_page_size (FFI)
    ↓ renderPageTree(at: 0) → rhwp_render_page_tree (FFI, JSON)
    ↓ JSONDecoder → RenderNode 트리
    ↓ CGTreeRenderer.render(tree:, in: ctx, ...) ← 순수 CoreGraphics/CoreText
    ↓ (필요 시) rhwp_image_data → CGImageSource 디코드
    ↓ QLPreviewReply 반환 (contextSize + drawingBlock)
```

### 핵심 설계 원칙

- **AppKit/UIKit 금지 (공유 코드)**: 공유 4종은 `CoreGraphics`, `CoreText`, `Foundation`, `ImageIO`만 사용
- **FFI 신규 추가 금지**: ios/devel의 FFI 표면(8종)을 변경 없이 사용
- **Stateless QL 호출**: QL extension은 호출마다 새로 파싱/렌더. `DocumentViewModel` 층 불필요
- **iOS 비파괴**: 공유 코드 수정 후 iOS Xcode 빌드가 여전히 통과해야 함

---

## 단계별 구현 계획 (Stage 0~7)

---

### Stage 0 — 업스트림 재검증

**목적**: 구현 착수 시점에 업스트림이 고정 SHA와 동일한지 30분 내 확인.

**작업 내용**:

1. SHA 드리프트 확인
   ```bash
   cd /Users/melee/Documents/projects/forks/rhwp
   git fetch upstream
   git log --oneline upstream/ios/devel -1
   # 34b6e20이면 OK. 다르면 커밋 로그 확인 후 영향 평가
   ```

2. FFI 표면 동일성 확인
   ```bash
   git show upstream/ios/devel:rhwp-ios/Sources/RhwpDocument.swift | \
     grep -oE 'rhwp_[a-z_]+' | sort -u
   ```

3. CGTreeRenderer 크기 확인
   ```bash
   git show upstream/ios/devel:rhwp-ios/Sources/CGTreeRenderer.swift | wc -c
   # 기준: 19196 바이트. ±20% 이내면 OK
   ```

4. M2 open 이슈 상태 확인
   ```bash
   gh api "repos/edwardkim/rhwp/issues?milestone=4&state=open" \
     --jq '.[] | select(.pull_request | not) | "#\(.number) \(.title)"'
   ```

**검증 기준**:
- [ ] SHA 동일 또는 FFI 미변경
- [ ] FFI 8개 모두 존재
- [ ] CGTreeRenderer 크기 ±20% 이내
- [ ] M2 open 이슈 변동 없거나 영향 무시 가능

**예상 공수**: 0.25일

---

### Stage 1 — Worktree 생성 + macOS 디렉토리 스캐폴드

**목적**: `git worktree`로 macOS 전용 worktree를 유지하고, `macos/devel` 기준에서 issue별 작업 브랜치를 분기하여 `rhwp-macos/` 디렉토리 구조를 확립한다.

**작업 내용**:

1. macOS worktree 생성 또는 확인
   ```bash
   cd /Users/melee/Documents/projects/forks/rhwp
   git fetch origin upstream
   git worktree list
   # /Users/melee/Documents/projects/rhwp-macos 가 표시되어야 한다.
   ```

2. 통합 기준 브랜치와 issue 작업 브랜치 정리
   ```bash
   cd /Users/melee/Documents/projects/rhwp-macos
   git switch macos/devel
   git pull --ff-only origin macos/devel
   git switch -c local/task3
   ```

   현재 상태:
   - `macos/devel`: `origin/macos/devel`과 동일하게 유지하는 macOS 개발 통합 기준
   - `local/task3`: <https://github.com/postmelee/rhwp/issues/3> 구현 작업 브랜치
   - `macos/quicklook`: 초기 스캐폴드용 브랜치였으며 현재는 `macos/devel`에 병합 완료

3. macOS 작업 디렉토리로 이동 후 구조 생성
   ```bash
   cd /Users/melee/Documents/projects/rhwp-macos
   mkdir -p rhwp-macos/{Sources/{HostApp/Resources,QLExtension,ThumbnailExtension},Frameworks,scripts}
   ```

4. 샘플 파일 복사
   ```bash
   cp rhwp-ios/Resources/sample.hwpx rhwp-macos/Sources/HostApp/Resources/
   ```

5. `.gitignore`에 macOS 관련 항목 추가

6. 초기 커밋: `macOS Quick Look: scaffold rhwp-macos/ directory`

**검증 기준**:
- [ ] `git worktree list`에 `/Users/melee/Documents/projects/rhwp-macos` 표시
- [ ] `macos/devel`이 `origin/macos/devel`과 동일
- [ ] 현재 작업 브랜치가 issue #3용 `local/task3`
- [ ] `rhwp-macos/` 디렉토리 구조 존재
- [ ] `rhwp-ios/` 기존 파일 변경 없음
- [ ] 메인 worktree (`forks/rhwp`)는 여전히 `devel` 브랜치

**예상 공수**: 0.25일

---

### Stage 2 — Rust macOS 빌드 파이프라인 (XCFramework)

**목적**: Swift에서 링크할 수 있는 universal `Rhwp.xcframework` 자동 생성.

**작업 내용**:

1. `rhwp-macos/scripts/build-rust-macos.sh` 작성
   ```bash
   #!/bin/bash
   set -euo pipefail
   SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
   ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
   OUT="$SCRIPT_DIR/../Frameworks"

   export MACOSX_DEPLOYMENT_TARGET=12.0
   cd "$ROOT"

   echo "[1/4] Rust staticlib (arm64 + x86_64)..."
   cargo build --release --lib --target aarch64-apple-darwin
   cargo build --release --lib --target x86_64-apple-darwin

   echo "[2/4] universal binary..."
   mkdir -p "$OUT/universal"
   xcrun lipo -create \
     "$ROOT/target/aarch64-apple-darwin/release/librhwp.a" \
     "$ROOT/target/x86_64-apple-darwin/release/librhwp.a" \
     -output "$OUT/universal/librhwp.a"
   xcrun lipo -info "$OUT/universal/librhwp.a"

   echo "[3/4] cbindgen FFI 심볼 검증..."
   GENERATED_H="$OUT/generated_rhwp.h"
   HEADER_SYMBOLS="$OUT/rhwp_header_symbols.txt"
   GENERATED_SYMBOLS="$OUT/generated_rhwp_symbols.txt"
   cbindgen --quiet --config "$ROOT/cbindgen.toml" --crate rhwp \
     --output "$GENERATED_H" "$ROOT"
   grep -oE '\brhwp_[a-z_]+' "$ROOT/rhwp-ios/Sources/rhwp.h" | sort -u > "$HEADER_SYMBOLS"
   grep -oE '\brhwp_[a-z_]+' "$GENERATED_H" | sort -u > "$GENERATED_SYMBOLS"
   diff -u "$HEADER_SYMBOLS" "$GENERATED_SYMBOLS"

   echo "[4/4] XCFramework..."
   rm -rf "$OUT/Rhwp.xcframework"
   MODMAP_DIR="$OUT/modulemap"
   mkdir -p "$MODMAP_DIR"
   cat > "$MODMAP_DIR/module.modulemap" <<EOF
   module Rhwp {
     header "rhwp.h"
     export *
   }
   EOF
   cp "$ROOT/rhwp-ios/Sources/rhwp.h" "$MODMAP_DIR/rhwp.h"

   xcodebuild -create-xcframework \
     -library "$OUT/universal/librhwp.a" -headers "$MODMAP_DIR" \
     -output "$OUT/Rhwp.xcframework"

   echo "완료: $OUT/Rhwp.xcframework"
   du -sh "$OUT/universal/librhwp.a" "$OUT/Rhwp.xcframework"
   ```

2. 최초 실행: `chmod +x rhwp-macos/scripts/build-rust-macos.sh && ./rhwp-macos/scripts/build-rust-macos.sh`

**검증 기준**:
- [ ] `Rhwp.xcframework` 생성
- [ ] universal binary에 `arm64` + `x86_64` 포함
- [ ] `nm` 결과에 `_rhwp_` 심볼 8개 이상
- [ ] cbindgen 생성 헤더의 `rhwp_` FFI 심볼 8종이 `rhwp-ios/Sources/rhwp.h`와 동일

**예상 공수**: 1일

---

### Stage 3 — Swift 코어 포팅 (원본 직접 수정 + iOS 빌드 검증)

**목적**: 공유 4종의 UIKit 의존을 CG/CT로 치환. iOS와 macOS에서 동일 코드로 동작.

**핵심**: `rhwp-ios/Sources/` 원본을 직접 수정. 수정 후 iOS Xcode 빌드 통과 확인.

**작업 내용**:

1. **`RhwpDocument.swift`**: 변경 없음

2. **`RenderTree.swift`**: 변경 없음

3. **`FontFallback.swift`**: `import UIKit` → 제거 (CoreText만 사용 중)

4. **`CGTreeRenderer.swift`** (핵심 — 19KB 전수 치환)

   | 원본 (UIKit) | 치환 (CG/CT) |
   |-------------|-------------|
   | `import UIKit` | `import CoreGraphics` + `import CoreText` + `import ImageIO` |
   | `UIColor.white.cgColor` | `CGColor(gray: 1, alpha: 1)` |
   | `UIColor(red:green:blue:alpha:).cgColor` | `CGColor(srgbRed:green:blue:alpha:)` |
   | `UIColor.clear.cgColor` | `CGColor(gray: 0, alpha: 0)` |
   | `UIImage(data:)?.cgImage` | `CGImageSourceCreateWithData` + `CGImageSourceCreateImageAtIndex` |
   | `UIScreen.main.scale` | 제거 |
   | `UIBezierPath` | `CGMutablePath` + `CGPath` API |

5. **iOS 빌드 검증** (포크 기반 최대 장점)
   ```bash
   cd rhwp-ios && xcodegen generate
   xcodebuild -scheme AlHangeul -destination 'platform=iOS Simulator,name=iPhone 16' build
   ```

6. **AppKit/UIKit 금지 lint** — `rhwp-macos/scripts/check-no-appkit.sh`
   ```bash
   #!/bin/bash
   set -euo pipefail
   ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
   SHARED_FILES=(
     "$ROOT/rhwp-ios/Sources/RhwpDocument.swift"
     "$ROOT/rhwp-ios/Sources/RenderTree.swift"
     "$ROOT/rhwp-ios/Sources/FontFallback.swift"
     "$ROOT/rhwp-ios/Sources/CGTreeRenderer.swift"
   )
   HITS=""
   for f in "${SHARED_FILES[@]}"; do
     FOUND=$(grep -nE 'AppKit|NSColor|NSImage|NSFont|NSView|UIKit|UIColor|UIImage|UIFont|UIBezier' "$f" 2>/dev/null || true)
     if [ -n "$FOUND" ]; then
       HITS+="$f:\n$FOUND\n"
     fi
   done
   if [ -n "$HITS" ]; then
     echo "FAIL: 공유 코드에 플랫폼 의존 발견"
     echo -e "$HITS"
     exit 1
   fi
   echo "OK: 공유 4종에 UIKit/AppKit 의존 없음"
   ```

7. CLI 검증 (macOS에서 `CGBitmapContext`로 PNG 덤프)

**검증 기준**:
- [ ] `check-no-appkit.sh` 통과
- [ ] **iOS Xcode 빌드 통과**
- [ ] CLI로 샘플 HWP 3종 → PNG 덤프 성공
- [ ] 한글 텍스트 누락 없음

**예상 공수**: 2~3일

---

### Stage 4 — Quick Look Preview Extension 최소 동작

**목적**: Finder에서 `.hwp` 파일 스페이스바 → 첫 페이지 미리보기.

**작업 내용**:

1. `rhwp-macos/project.yml` — 공유 파일은 **`../rhwp-ios/Sources/` 상대경로 참조**
   ```yaml
   name: RhwpMacOS
   options:
     bundleIdPrefix: com.postmelee
     deploymentTarget:
       macOS: "12.0"
   settings:
     base:
       SWIFT_VERSION: "5.9"
   targets:
     HostApp:
       type: application
       platform: macOS
       sources:
         - path: Sources/HostApp
         - path: ../rhwp-ios/Sources/RhwpDocument.swift
         - path: ../rhwp-ios/Sources/RenderTree.swift
         - path: ../rhwp-ios/Sources/FontFallback.swift
         - path: ../rhwp-ios/Sources/CGTreeRenderer.swift
       dependencies:
         - framework: Frameworks/Rhwp.xcframework
           embed: true
       settings:
         base:
           PRODUCT_BUNDLE_IDENTIFIER: com.postmelee.rhwpmac
           SWIFT_OBJC_BRIDGING_HEADER: ../rhwp-ios/Sources/rhwp-Bridging-Header.h
           CODE_SIGN_STYLE: Automatic
     QLExtension:
       type: app-extension
       platform: macOS
       sources:
         - path: Sources/QLExtension
         - path: ../rhwp-ios/Sources/RhwpDocument.swift
         - path: ../rhwp-ios/Sources/RenderTree.swift
         - path: ../rhwp-ios/Sources/FontFallback.swift
         - path: ../rhwp-ios/Sources/CGTreeRenderer.swift
       dependencies:
         - target: HostApp
           embed: true
         - framework: Frameworks/Rhwp.xcframework
           embed: false
       settings:
         base:
           PRODUCT_BUNDLE_IDENTIFIER: com.postmelee.rhwpmac.QLExtension
           INFOPLIST_FILE: Sources/QLExtension/Info.plist
           SWIFT_OBJC_BRIDGING_HEADER: ../rhwp-ios/Sources/rhwp-Bridging-Header.h
   ```

2. UTI 선언 — HostApp Info.plist에 Exporter(자체) + Importer(hancom)

3. QL Extension Info.plist — 4종 UTI 지원

4. `HwpPreviewProvider.swift` — 50MB 크기 가드 포함

5. `HostApp.swift` — 최소 SwiftUI 앱 (설치 안내 화면)

6. 빌드 + 설치 + 검증
   ```bash
   ./rhwp-macos/scripts/build-rust-macos.sh
   cd rhwp-macos && xcodegen generate
   xcodebuild -scheme HostApp -configuration Debug \
       CONFIGURATION_BUILD_DIR="$PWD/build/debug" build
   cp -R "build/debug/HWP Quick Look.app" /Applications/
   open "/Applications/HWP Quick Look.app"
   pluginkit -mAvvv | grep rhwpmac
   ```

7. 메모리 계측 — peak RSS < 200MB (100페이지 HWP)

**검증 기준**:
- [ ] Finder 스페이스바 → 첫 페이지 렌더 표시
- [ ] 렌더 10초 내 완료 (목표 3초)
- [ ] `pluginkit` 등록 확인
- [ ] 50MB 초과 파일 → fallback, 크래시 없음
- [ ] peak memory < 200MB

**예상 공수**: 2~3일

---

### Stage 5 — Thumbnail Extension + 호스트 앱 완성도

**목적**: Finder 썸네일 지원 + HostApp UX 개선.

**작업 내용**:

1. `project.yml`에 ThumbnailExtension 추가 (공유 파일 동일 상대경로 참조)

2. `HwpThumbnailProvider.swift` — 크기 가드 + aspect ratio 맞춤

3. HostApp UI 보완 — 확장 등록 상태, 샘플 보기, 버전/SHA 표시

4. 앱 아이콘 플레이스홀더

**검증 기준**:
- [ ] Finder 아이콘 뷰에서 HWP 썸네일 표시
- [ ] 썸네일 생성 < 2초
- [ ] HostApp에서 QL/Thumbnail "등록됨" 표시

**예상 공수**: 1~2일

---

### Stage 6 — macOS HostApp Viewer parity

**목적**: upstream iOS 앱의 Viewer 기능 수준을 macOS HostApp에도 구현한다. Quick Look 설치 상태 앱에 머물지 않고, 앱을 직접 실행했을 때 HWP/HWPX 문서를 열어 다중 페이지로 볼 수 있게 한다.

**범위**:
- 편집/저장/export는 제외한다. upstream iOS 앱에도 아직 연결되지 않은 기능이므로 별도 이슈로 분리한다.
- 기존 Quick Look Preview/Thumbnail Extension 기능은 유지한다.
- 기존 공유 Swift 렌더러와 Rust FFI 8종을 그대로 사용한다.

**작업 내용**:

1. `project.yml`에서 HostApp도 공유 렌더링 소스와 `Rhwp.xcframework`에 링크한다.

2. HostApp 구조 분리
   - `HostApp.swift`: 앱 entry + command/menu wiring
   - `Views/ContentView.swift`: root layout
   - `Views/DocumentViewerView.swift`: 문서 화면
   - `Views/DocumentPageView.swift`: `NSViewRepresentable` 기반 페이지 canvas
   - `Stores/DocumentViewerStore.swift`: 문서 로드, 페이지 cache, 현재 페이지, zoom 상태
   - `Services/DocumentOpenPanel.swift`: `NSOpenPanel` 파일 선택
   - `Services/ExtensionStatusModel.swift`: Stage 5 등록 상태 확인
   - `Support/BuildInfo.swift`: 버전 표시

3. Viewer 기능
   - 앱 실행 시 번들 `sample.hwpx` 자동 로드
   - `NSOpenPanel`로 `.hwp`, `.hwpx` 열기
   - 다중 페이지 세로 스크롤
   - `onAppear`/`onDisappear` 기반 페이지 lazy load/cache
   - zoom in/out/reset 및 slider
   - 현재 페이지/전체 페이지 표시
   - 에러/로딩 상태 표시

4. Quick Look 상태 UI는 사이드/하단 정보로 유지한다.

**검증 기준**:
- [ ] HostApp Debug 빌드 통과
- [ ] 앱 실행 시 `sample.hwpx` 자동 로드
- [ ] `samples/basic/KTX.hwp` 열기 성공
- [ ] 다중 페이지 스크롤 가능
- [ ] zoom in/out/reset 동작
- [ ] 현재 페이지/전체 페이지 표시
- [ ] Preview/Thumbnail 등록 상태 표시 유지
- [ ] Finder Quick Look/Thumbnail Stage 4~5 회귀 없음

**예상 공수**: 1~2일

---

### Stage 7 — 릴리스 패키징 + Homebrew cask 초안

**목적**: 재현 가능한 배포 산출물 생성.

**작업 내용**:

1. `rhwp-macos/scripts/release.sh` — 빌드 + zip 패키징

2. 포크 GitHub Releases로 배포 (`macos-v0.1.0` 태그)

3. 본인 Homebrew tap cask 초안
   ```ruby
   cask "rhwp-macos" do
     version "0.1.0"
     sha256 "<release.sh 출력>"
     url "https://github.com/postmelee/rhwp/releases/download/macos-v#{version}/rhwp-macos-#{version}.zip"
     name "HWP Quick Look"
     desc "Quick Look and Thumbnail extension for HWP/HWPX files"
     homepage "https://github.com/postmelee/rhwp"
     depends_on macos: ">= :monterey"
     app "HWP Quick Look.app"
   end
   ```

4. `rhwp-macos/README.md` — 설치, 빌드, 크레딧

**검증 기준**:
- [ ] `release.sh 0.1.0` 성공
- [ ] `brew install --cask rhwp-macos` 성공 (본인 tap)

**예상 공수**: 1~2일

---

## 전체 예상 공수

| Stage | 작업 | 공수 |
|-------|------|------|
| 0 | 업스트림 재검증 | 0.25일 |
| 1 | 브랜치 생성 + 스캐폴드 | 0.25일 |
| 2 | Rust XCFramework 빌드 | 1일 |
| 3 | Swift 코어 포팅 (원본 수정 + iOS 검증) | 2~3일 |
| 4 | QL Extension + UTI + 크기 가드 | 2~3일 |
| 5 | Thumbnail + HostApp UI | 1~2일 |
| 6 | macOS HostApp Viewer parity | 1~2일 |
| 7 | 릴리스 + cask 초안 | 1~2일 |
| **합계** | | **8.5 ~ 14.5일** |

## 리스크 & 대응

| 리스크 | 대응 |
|--------|------|
| Rust 크로스 빌드 네이티브 의존성 충돌 | ios/devel iOS 빌드 성공 사례. feature flag로 비활성 |
| CGTreeRenderer UIKit 전수 치환 누락 | `check-no-appkit.sh` + grep 전수 |
| UIKit 제거 후 iOS 빌드 깨짐 | **포크 기반 최대 장점** — Stage 3에서 즉시 검증 |
| `@MainActor` 제약으로 QL drawingBlock 실패 | `nonisolated` 래퍼 또는 `@MainActor` 제거 |
| 업스트림 ios/devel 갱신 시 merge 충돌 | 공유 4종만 수정하므로 낮음 |
| 대용량 파일 OOM | 50MB 크기 가드 + fallback reply |

## 업스트림 통합 전략 (Stage 6 이후)

v0.1.0 릴리스 후, 업스트림에 PR 2건 분리 제안:

**PR 1 — 공유 레이어 정리**: `CGTreeRenderer.swift`, `FontFallback.swift`의 UIKit 제거. iOS 빌드 통과 검증 첨부.

**PR 2 — macOS Quick Look 추가**: `rhwp-macos/` 디렉토리 전체 (PR 1 머지 후).

## 선행 조건 체크리스트

```
선행 환경
  [ ] Apple ID → Xcode Accounts 추가
  [ ] rustup target add aarch64-apple-darwin x86_64-apple-darwin
  [ ] cargo install cbindgen
  [ ] brew install xcodegen

Stage 0  (0.25일)
  [ ] git fetch upstream → SHA 확인
  [ ] FFI 8종 확인
  [ ] CGTreeRenderer 크기 ±20%
  [ ] M2 open 이슈 변동 확인

Stage 1  (0.25일)
  [ ] git worktree list 확인
  [ ] macos/devel이 origin/macos/devel과 동일
  [ ] issue #3 작업 브랜치 local/task3 생성
  [ ] rhwp-macos/ 스캐폴드
  [ ] .gitignore macOS 항목 추가
  [ ] 초기 커밋

Stage 2  (1일)
  [ ] build-rust-macos.sh 작성
  [ ] Rhwp.xcframework 생성 성공
  [ ] 심볼 검증 (nm, 8개)

Stage 3  (2-3일)
  [ ] FontFallback.swift: import UIKit 제거
  [ ] CGTreeRenderer.swift: UIKit → CG/CT 전수 치환
  [ ] check-no-appkit.sh 통과
  [ ] iOS Xcode 빌드 통과
  [ ] CLI PNG 덤프 검증

Stage 4  (2-3일)
  [ ] project.yml (공유 파일 상대경로 참조)
  [ ] UTI Exporter + Importer 선언
  [ ] HwpPreviewProvider (크기 가드)
  [ ] Finder 스페이스바 검증

Stage 5  (1-2일)
  [ ] ThumbnailExtension
  [ ] HostApp UX 개선
  [ ] 앱 아이콘 플레이스홀더

Stage 6  (1-2일)
  [ ] release.sh
  [ ] 포크 GitHub Releases 배포
  [ ] Homebrew tap cask 초안
  [ ] rhwp-macos/README.md
```

## 참고 자료

- 포크 내 CLAUDE.md — 하이퍼-워터폴 규칙, 문서 구조
- `mydocs/manual/browser_extension_dev_guide.md` — 확장 개발 가이드
- `mydocs/tech/font_fallback_strategy.md` — 폰트 폴백 전략
- 업스트림 `rhwp-ios/Sources/` — 공유 Swift 파일 원본
- 업스트림 `src/ios_ffi.rs` — FFI 표면 정의
- 업스트림 milestone 4 — https://github.com/edwardkim/rhwp/milestone/4
- Apple: [QLPreviewProvider](https://developer.apple.com/documentation/quicklookui/qlpreviewprovider), [QLThumbnailProvider](https://developer.apple.com/documentation/quicklookthumbnailing/qlthumbnailprovider)
- [sbarex/QLMarkdown](https://github.com/sbarex/QLMarkdown) — 참조 구현
