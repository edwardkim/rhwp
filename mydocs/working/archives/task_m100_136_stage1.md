# 단계별 완료 보고서 — Task #136 Stage 1~3 (전체 완료)

**이슈**: [#136](https://github.com/edwardkim/rhwp/issues/136)
**브랜치**: `local/task136`
**커밋**: `343acdc`

---

## 완료 내용

### 단계 1: viewer.ts — `exportDebugOverlay` 메시지 처리

- `window.addEventListener("message", ...)` 핸들러에 `exportDebugOverlay` 케이스 추가
- `hwpDoc.set_debug_overlay(true)` → `renderPageSvg()` × N → `set_debug_overlay(false)`
- 결과 SVG 배열을 `{ type: "debugOverlaySvgs", svgs }` 로 extension host에 응답
- 에러 발생 시 `{ type: "debugOverlaySvgs", error }` 응답

### 단계 2: hwp-editor-provider.ts — `sendDebugOverlay()` + 콜백 처리

- `pendingDebugOverlay: Set<string>` 추가
- `debugOverlayCallbacks: Map<string, (svgs: string[]) => void>` 추가
- `sendDebugOverlay(uri, onSvgs)` 메서드 추가
  - 뷰어 오픈 시 → 즉시 `exportDebugOverlay` 메시지 전송
  - 미오픈 시 → `vscode.openWith` + `pendingDebugOverlay` 등록
- `onDidReceiveMessage`에 `debugOverlaySvgs` 케이스 추가 → 콜백 호출
- `ready` 핸들러에 `pendingDebugOverlay` 처리 추가 (500ms 지연)

### 단계 3: extension.ts — `cmdDebugOverlay` webview 위임 방식으로 전환

- 기존 extension host 직접 WASM 로드 코드 제거
- `provider.sendDebugOverlay(uri, callback)` 호출로 대체
- HTML 생성/저장/열기 로직은 콜백 내부로 이동 (유지)

## 빌드 결과

```
webpack 5.105.4 compiled successfully in 3500 ms (extension)
webpack 5.105.4 compiled successfully in 3434 ms (webview)
```

에러 없음. 기존 size WARNING은 폰트/WASM 크기로 기존과 동일.

## 기대 효과

- 디버그 오버레이 HTML의 표 안 글자 배치가 뷰어와 동일
- `measureTextWidth` 스텁(`text.length * 8`) 의존 제거 (디버그 오버레이 한정)
- 뷰어 미오픈 상태에서도 자동으로 열고 오버레이 생성
