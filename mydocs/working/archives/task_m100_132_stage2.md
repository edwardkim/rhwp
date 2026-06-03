# 2단계 완료 보고서 — Task #132

**단계**: 2단계 — 패널 추적 Map + rhwp.print 커맨드 + viewer.ts print 메시지 처리
**완료일**: 2026-04-13

## 변경 파일

- `rhwp-vscode/src/hwp-editor-provider.ts`
- `rhwp-vscode/src/webview/viewer.ts`
- `rhwp-vscode/src/extension.ts` (커맨드 등록 골격)

## 변경 내용

### hwp-editor-provider.ts

- `register()` 반환 타입 변경: `vscode.Disposable` → `{ provider, disposable }` (외부에서 provider 인스턴스 접근 가능)
- `panels: Map<string, vscode.WebviewPanel>` 추가 — 파일 URI → 패널 추적
- `pendingPrint: Set<string>` 추가 — 뷰어 미오픈 상태에서 인쇄 요청 처리
- `resolveCustomEditor()`에서 패널 등록/해제 (onDidDispose 시 Map에서 제거)
- `sendPrint(uri)` 메서드:
  - 패널이 열려 있으면 `webview.postMessage({ type: "print" })` 즉시 전송
  - 패널이 없으면 `vscode.openWith`로 뷰어를 먼저 열고 ready 수신 후 500ms 지연 후 print 전송

### viewer.ts

- `window.print()` 메시지 처리 추가:
  ```typescript
  if (msg.type === "print") { window.print(); }
  ```

### extension.ts

- `HwpEditorProvider.register()` 호출 방식 변경 (provider 인스턴스 분리)
- 4개 커맨드 핸들러 골격 등록 (exportSvg/debugOverlay/dumpParagraph는 stub)
- `resolveUri()` 헬퍼: 컨텍스트 메뉴 uri 또는 활성 편집기 uri 반환
