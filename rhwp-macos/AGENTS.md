# AGENTS.md — rhwp-macos (macOS Quick Look 확장)

이 파일은 `rhwp-macos/` 서브디렉토리에 한정된 에이전트 규칙이다.
프로젝트 전반 규칙(하이퍼-워터폴, 문서 생성 규칙, 타스크 진행 절차)은 저장소 루트의 `AGENTS.md`에서 상속된다.

## 프로젝트 개요

**목표**: macOS 전용 HWP/HWPX Quick Look 확장

- **Quick Look Preview Extension**: Finder에서 `.hwp`/`.hwpx` 스페이스바 미리보기
- **Thumbnail Extension**: Finder 아이콘/표/갤러리 뷰 썸네일 생성
- **Host App**: 확장 등록용 최소 SwiftUI 앱 + 사용 안내 UI
- **배포**: Homebrew cask 1차 (비공증 unsigned), 후속으로 notarization 트랙

## 코어 재사용 전략 (포크 기반)

- 업스트림 [`edwardkim/rhwp`](https://github.com/edwardkim/rhwp)를 `postmelee/rhwp`로 Fork
- macOS 개발 통합 기준은 `macos/devel` 브랜치로 둔다.
- 실제 기능 구현은 GitHub Issue 번호에 맞춘 `local/task{번호}` 브랜치에서 진행한다.
- 현재 Quick Look 구현 작업은 `postmelee/rhwp#3` 기반 `local/task3`에서 진행한다.
- **git worktree**로 별도 디렉토리(`/Users/melee/Documents/projects/rhwp-macos`)를 사용한다.
- 공유 Swift 4종(`RhwpDocument`, `RenderTree`, `FontFallback`, `CGTreeRenderer`)은 `rhwp-ios/Sources/` **원본을 직접 수정** (복사 아님)
- CoreGraphics/CoreText 단일 스택 — AppKit/UIKit 금지

**상세 기술 결정**: `mydocs/plans/rhwp_quicklook_macos_impl.md` 참조

---

## 필수 참조 문서

- `mydocs/plans/rhwp_quicklook_macos_impl.md` — 전체 구현 계획서 (포크 기반 + worktree)
- 저장소 루트 `AGENTS.md` — 업스트림 기여 규칙 (하이퍼-워터폴)
- `mydocs/manual/browser_extension_dev_guide.md` — 브라우저 확장 개발 가이드
- `mydocs/tech/font_fallback_strategy.md` — 폰트 폴백 전략

---

## 빌드 및 실행

### 선행 조건 (최초 1회)

```bash
# Rust 툴체인 (universal 바이너리용)
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# FFI 헤더 생성기
cargo install cbindgen

# Xcode 프로젝트 선언적 관리
brew install xcodegen
```

추가로 Xcode 15 이상 + macOS 12 이상 SDK + 본인 Apple ID 등록(Personal Team) 필요.

### Rust 코어 빌드 (XCFramework)

```bash
cd /Users/melee/Documents/projects/rhwp-macos       # macOS worktree
./rhwp-macos/scripts/build-rust-macos.sh
```

산출물:
- `rhwp-macos/Frameworks/universal/librhwp.a` — arm64 + x86_64 lipo 결과
- `rhwp-macos/Frameworks/Rhwp.xcframework` — Swift에서 링크하는 최종 모듈

**주의**: `rhwp-macos/Frameworks/` 하위는 전체 `.gitignore`. 빌드 스크립트가 매번 재생성한다.

### Xcode 프로젝트 재생성

```bash
cd rhwp-macos
xcodegen generate   # project.yml → RhwpMacOS.xcodeproj
```

Xcode 프로젝트 파일은 XcodeGen의 `rhwp-macos/project.yml`에서 선언적으로 관리한다. `.xcodeproj`를 직접 편집하지 않는다.

### 앱/확장 빌드 (Debug)

```bash
cd /Users/melee/Documents/projects/rhwp-macos
./rhwp-macos/scripts/build-rust-macos.sh
cd rhwp-macos && xcodegen generate
xcodebuild -scheme HostApp -configuration Debug \
  CONFIGURATION_BUILD_DIR="$PWD/build/debug" build
```

빌드 산출물을 `/Applications/`에 복사 후 1회 실행하면 Quick Look/Thumbnail 확장이 시스템에 등록된다.

```bash
cp -R "build/debug/HWP Quick Look.app" /Applications/
open "/Applications/HWP Quick Look.app"
```

### 릴리스 빌드 + 패키징

```bash
./rhwp-macos/scripts/release.sh 0.1.0
# → rhwp-macos/build/rhwp-macos-0.1.0.zip
# → SHA256 출력 (Homebrew cask용)
```

### AppKit/UIKit 금지 lint

공유 코드(`rhwp-ios/Sources/` 내 4종)는 CoreGraphics/CoreText/Foundation/ImageIO만 사용한다.

```bash
./rhwp-macos/scripts/check-no-appkit.sh
```

### 기준 브랜치 동기화

별도의 vendoring 스크립트 없음. `git merge`로 직접 동기화한다.

```bash
cd /Users/melee/Documents/projects/rhwp-macos       # macOS worktree
git switch macos/devel
git fetch origin upstream
git pull --ff-only origin macos/devel

# 업스트림 ios/devel 반영이 필요하다고 판단될 때만 수행
git merge upstream/ios/devel

# 작업 브랜치로 기준 변경 수신
git switch local/task3
git merge macos/devel
```

동기화 후 반드시 수행:
1. `rhwp-macos/scripts/build-rust-macos.sh` 성공 확인
2. `rhwp-macos/scripts/check-no-appkit.sh` 통과 확인
3. 샘플 HWP 3종 QL 렌더 육안 검증

---

## 업스트림 기여 규칙

본 프로젝트의 macOS 통합 기준은 `postmelee/rhwp` 포크의 `macos/devel` 브랜치다. 개별 작업은 `local/task{issue번호}`에서 수행하며, 현재 Quick Look 구현은 `local/task3`에서 진행한다. 업스트림(`edwardkim/rhwp`)에 기여할 때는 다음 규칙을 따른다.

### 분류 기준

| 변경 성격 | 작업 위치 | 상향 기여 |
|-----------|-----------|-----------|
| QL/Thumbnail Extension (`rhwp-macos/Sources/QLExtension/`, `ThumbnailExtension/`) | task 브랜치 → `macos/devel` | 해당 없음 |
| macOS 호스트 앱 UI (`rhwp-macos/Sources/HostApp/`) | task 브랜치 → `macos/devel` | 해당 없음 |
| 빌드 스크립트/프로젝트 설정 (`rhwp-macos/scripts/`, `project.yml`) | task 브랜치 → `macos/devel` | 해당 없음 |
| **CGTreeRenderer의 UIKit→CG 치환** (iOS에도 유용한 개선) | task 브랜치 + **업스트림 PR** | **필수** |
| **FontFallback의 macOS 폰트 매핑 추가** | task 브랜치 + **업스트림 PR** | **필수** |
| **Rust 코어 버그 수정 / 렌더링 개선** | **업스트림 PR 우선** → `git merge`로 수신 | **필수** |
| **FFI 시그니처 추가/변경 제안** | **업스트림 이슈 등록 + PR 제안** | **필수** |

### 상향 기여 절차

1. task 브랜치(현재 `local/task3`)에서 먼저 구현/검증 — 동작하는 상태 확보
2. `edwardkim/rhwp`의 `ios/devel` 브랜치를 base로 PR 생성
3. PR 설명에 다음 포함:
   - macOS Quick Look 확장 개발 과정에서 발견된 배경
   - iOS에도 동일하게 유효한 이유
   - iOS 빌드 통과 검증 결과
4. PR 머지 후 `macos/devel`에서 `git merge upstream/ios/devel`로 동기화하고, task 브랜치로 다시 병합

### 업스트림 통합 전략 (v0.1.0 이후)

v0.1.0 완성 후 업스트림에 PR 2건 분리 제안:

**PR 1 — 공유 레이어 정리**: `CGTreeRenderer.swift`, `FontFallback.swift`의 UIKit 제거. iOS 빌드 통과 검증 첨부.

**PR 2 — macOS Quick Look 추가**: `rhwp-macos/` 디렉토리 전체 (PR 1 머지 후).

---

## HWPUNIT 및 좌표계

- 1 pt = 100 HWPUNIT

Quick Look `drawingBlock`에 전달되는 `CGContext`는 **좌상단 원점** (iOS `UIView.draw`와 동일). 본 프로젝트는 CGPDFContext를 쓰지 않으므로 flip 코드 없음.

---

## 샘플 파일 및 출력 폴더

### 샘플

- `rhwp-macos/Sources/HostApp/Resources/sample.hwpx` — `rhwp-ios/Resources/sample.hwpx` 복사본 (최초 QL 동작 검증용)
- 추가 테스트용 HWP는 `tests/samples/`에 두되 Git 비추적 (용량/저작권 이슈)

### 출력 폴더 (Git 비추적)

`output/` 하위를 용도별 서브폴더로 분리. `.gitignore` 등록.

| 폴더 | 용도 |
|------|------|
| `output/png/` | CLI 검증 툴 PNG 덤프 (Stage 3 육안 검증) |
| `output/debug/` | QL 디버그 로그 캡처 |

---

## QL/Thumbnail 디버깅 명령

### 확장 등록 확인

```bash
pluginkit -mAvvv | grep rhwpmac
```

### Quick Look 재시작

```bash
qlmanage -r
qlmanage -r cache
killall Finder
```

### 샌드박스 위반 모니터링

```bash
log stream --predicate 'subsystem == "com.apple.quicklook"'
log stream --predicate 'eventMessage CONTAINS "sandbox"'
```

### 단독 QL 호출 (확장 디버그)

```bash
qlmanage -p path/to/sample.hwp
qlmanage -t -s 512 -o /tmp path/to/sample.hwp   # 썸네일
```

### 메모리 계측

Instruments → Allocations / Leaks → Attach to `QuickLookUIService` 또는 `com.apple.quicklook.ThumbnailsAgent`. 목표: 100페이지 HWP 기준 peak RSS < 200MB.

---

## 디버깅 워크플로우 (일반)

렌더 이상 디버깅 시 다음 순서:

1. **샘플 재현** — `qlmanage -p`로 문제 HWP 재현
2. **CLI PNG 덤프** — Stage 3의 CLI 검증 툴로 같은 파일을 PNG 출력, 렌더러 단독 동작 확인
3. **상류 IR 대조** — 문제 원인이 FFI 이전(파서)인지 이후(렌더러)인지 구분하기 위해, 업스트림 `rhwp` 리포에서 `rhwp dump-pages` / `rhwp ir-diff` 실행하여 IR을 비교
4. **로컬 Swift 레이어** — CGTreeRenderer에 단계별 `CGContext` 저장 후 비교

업스트림 CLI 도구(`rhwp dump`, `rhwp ir-diff`, `rhwp export-svg`)는 메인 worktree(`forks/rhwp`)에서 직접 실행한다.

---

## Worktree 구성

```
/Users/melee/Documents/projects/forks/rhwp    ← 메인 worktree (devel) — 다른 기여
/Users/melee/Documents/projects/rhwp-macos    ← macOS worktree (local/task3) — issue #3 작업
```

두 worktree는 같은 git 저장소를 공유한다. remote, branch, fetch는 한쪽에서 수행하면 양쪽 반영.

### 브랜치 관리

| 브랜치 | worktree | 용도 |
|--------|----------|------|
| `devel` | `forks/rhwp` | 다른 기여 (PR, 버그 수정 등) |
| `macos/devel` | `rhwp-macos` | macOS 개발 통합 기준. `origin/macos/devel`과 동기화 |
| `local/task{num}` | `rhwp-macos`에서 `macos/devel` 기준 분기 | 타스크별 작업. 현재 `local/task3` |
| `macos/quicklook` | 없음(원격/이력 참조) | 초기 스캐폴드 브랜치. 현재 변경은 `macos/devel`에 병합됨 |

### 업스트림 기여 타스크 특례

본 브랜치 작업이 상향 기여 대상 변경을 포함하는 경우 (CGTreeRenderer/FontFallback/Rust 코어):

1. 타스크 진행 절차의 구현 계획서에 **"상향 기여 범위"** 섹션 포함
   - edwardkim/rhwp로 보낼 diff 범위
   - 본 브랜치에만 남길 macOS 특화 부분 구분
2. 단계별 완료 보고서에 업스트림 PR 번호/상태 기록
3. 최종 보고서에 `git merge upstream/ios/devel` 실행 시점 명시

---

## 작업 규칙

- 파생 아티팩트(XCFramework, .app, zip) 커밋 금지. `.gitignore` 유지.
- 공유 코드(`rhwp-ios/Sources/` 내 4종) AppKit/UIKit 도입 금지.
- `rhwp-macos/Sources/HostApp/` 또는 확장 Sources에서만 macOS 전용 프레임워크(SwiftUI 등) 사용.
