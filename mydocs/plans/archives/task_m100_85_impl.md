# Task #85: iOS 네이티브 HWP 뷰어 — 구현 계획서

## 현재 아키텍처

### Xcode 프로젝트 (`rhwp-safari/HWP Viewer/`)
- macOS 앱 + 확장: Safari 확장 안내 화면 → **유지**
- iOS 앱 + 확장: Safari 확장 안내 화면 → **뷰어 앱으로 전환**
- 공유 코드: `Shared (App)/ViewController.swift` — `#if os(iOS)` / `#if os(macOS)` 분기

### 뷰어 리소스 (`rhwp-safari/dist/`)
- `viewer.html` + `assets/viewer-*.js` + `assets/viewer-*.css`
- `assets/rhwp_bg-*.wasm` (Vite 빌드 산출물)
- `wasm/rhwp_bg.wasm` (원본)
- `fonts/` (오픈소스 woff2)

## 구현 단계 (3단계)

### 1단계: iOS ViewController를 WKWebView 뷰어로 전환

**대상 파일:** `Shared (App)/ViewController.swift`

**작업 내용:**
1. `#if os(iOS)` 분기에서 기존 안내 페이지(`Main.html`) 대신 뷰어(`viewer.html`) 로드
2. `WKWebView` 설정:
   - JavaScript 활성화
   - `allowFileAccessFromFileURLs` 활성화 (로컬 WASM/JS 로드)
   - `allowingReadAccessTo: Bundle.main.resourceURL` (번들 전체 접근)
3. 뷰어 리소스를 iOS 앱 번들에 포함 (현재 확장에만 포함)
4. 파일 선택 UI: `UIDocumentPickerViewController`로 HWP 파일 선택
5. 선택된 파일을 WKWebView의 뷰어에 전달 (`evaluateJavaScript` 또는 URL 파라미터)

**macOS 영향:** 없음 (`#if os(macOS)` 분기는 기존 그대로)

**검증:** iOS 실제 기기에서 앱 열기 → 파일 선택 → HWP 렌더링

---

### 2단계: Share Sheet + 파일 연결

**대상 파일:** `iOS (App)/Info.plist`, 신규 Share Extension

**작업 내용:**
1. `Info.plist`에 UTType 등록:
   - `public.hwp` → `.hwp` 파일
   - `public.hwpx` → `.hwpx` 파일
2. `CFBundleDocumentTypes`로 .hwp/.hwpx 파일 연결
3. Share Extension 추가: Safari에서 "HWP Viewer로 열기"
4. `SceneDelegate.swift`에서 외부 파일 열기 처리 (`scene(_:openURLContexts:)`)

**검증:** Safari에서 HWP 링크 → 공유 → HWP Viewer / 파일 앱에서 .hwp 탭 → HWP Viewer

---

### 3단계: UX + 검증

**작업 내용:**
1. 뷰어 전체화면: status bar, safe area 처리
2. 파일명 표시 (navigation bar)
3. 로딩 상태 표시
4. 에러 처리 (파일 로드 실패 등)
5. iOS 실제 기기 전체 테스트
6. macOS Safari 확장 회귀 테스트

---

## 핵심 기술 결정

### 뷰어 리소스 로딩 방식

WKWebView에서 로컬 파일을 로드하는 방식:
```swift
// 번들 내 viewer.html 로드 + 전체 번들 디렉토리 접근 허용
webView.loadFileURL(viewerURL, allowingReadAccessTo: bundleResourceURL)
```

이 방식은 `file://` 프로토콜로 로드하므로 `safari-web-extension://` 제약이 없다.
iOS Safari에서 웹버전이 동작하는 것을 확인했으므로 WKWebView에서도 동작 보장.

### HWP 파일 전달 방식

1. **URL 파라미터**: `viewer.html?url=file:///path/to/doc.hwp` — 로컬 파일 경로 전달
2. **JavaScript 브릿지**: `webView.evaluateJavaScript("loadFile(data)")` — 바이너리 데이터 직접 전달
3. **로컬 HTTP 서버**: 앱 내 간이 서버 → 복잡도 높음, 불필요

**권장: 방식 2 (JavaScript 브릿지)** — 파일 데이터를 base64로 인코딩하여 전달

### Xcode 프로젝트 수정 범위

- `ViewController.swift`: iOS 분기 수정 (macOS 유지)
- `iOS (App)/Info.plist`: UTType, Document Types 추가
- `iOS (App)/SceneDelegate.swift`: 외부 파일 열기 처리
- 뷰어 리소스: iOS 앱 번들에 복사 (현재 확장 번들에만 있음)

## 리스크

| 리스크 | 대응 |
|--------|------|
| WKWebView에서 WASM 로드 실패 | `allowFileAccessFromFileURLs` 설정, CSP 미적용 확인 |
| 대용량 HWP base64 전달 시 메모리 | 파일 크기 제한 (20MB) + ArrayBuffer 전달 방식 검토 |
| Xcode 프로젝트 수정 시 macOS 영향 | `#if os(iOS)` 분기로 격리 |
