# 구현 계획서 — Task #136

**이슈**: [#136](https://github.com/edwardkim/rhwp/issues/136)
**타이틀**: 디버그 오버레이 내보내기 글자 배치 오류 — webview 렌더링 방식으로 전환
**마일스톤**: M100
**브랜치**: `local/task136`

---

## 단계 1: webview — `exportDebugOverlay` 메시지 처리

**파일**: `rhwp-vscode/src/webview/viewer.ts`

`window.addEventListener("message", ...)` 핸들러에 `exportDebugOverlay` 케이스 추가:

```typescript
if (msg.type === "exportDebugOverlay") {
  if (!hwpDoc) {
    vscode.postMessage({ type: "debugOverlaySvgs", error: "문서가 로드되지 않았습니다" });
    return;
  }
  try {
    hwpDoc.set_debug_overlay(true);
    const svgs: string[] = [];
    for (let i = 0; i < pageInfos.length; i++) {
      svgs.push(hwpDoc.renderPageSvg(i));
    }
    hwpDoc.set_debug_overlay(false);
    vscode.postMessage({ type: "debugOverlaySvgs", svgs });
  } catch (err: any) {
    hwpDoc.set_debug_overlay(false);
    vscode.postMessage({ type: "debugOverlaySvgs", error: err.message ?? String(err) });
  }
}
```

---

## 단계 2: hwp-editor-provider — `sendDebugOverlay()` + SVG 응답 콜백

**파일**: `rhwp-vscode/src/hwp-editor-provider.ts`

### 2-1. `pendingDebugOverlay` Set 추가

```typescript
private readonly pendingDebugOverlay = new Set<string>();
```

### 2-2. SVG 응답 콜백 타입 추가

```typescript
private readonly debugOverlayCallbacks = new Map<string, (svgs: string[]) => void>();
```

### 2-3. `sendDebugOverlay()` 메서드 추가

```typescript
async sendDebugOverlay(uri: vscode.Uri, onSvgs: (svgs: string[]) => void): Promise<void> {
  const key = uri.toString();
  const panel = this.panels.get(key);
  this.debugOverlayCallbacks.set(key, onSvgs);
  if (panel) {
    panel.reveal();
    panel.webview.postMessage({ type: "exportDebugOverlay" });
  } else {
    await vscode.commands.executeCommand("vscode.openWith", uri, HwpEditorProvider.viewType);
    this.pendingDebugOverlay.add(key);
  }
}
```

### 2-4. `onDidReceiveMessage` 핸들러에 케이스 추가

```typescript
if (msg.type === "debugOverlaySvgs") {
  const cb = this.debugOverlayCallbacks.get(key);
  this.debugOverlayCallbacks.delete(key);
  if (msg.error) {
    vscode.window.showErrorMessage(`디버그 오버레이 실패: ${msg.error}`);
  } else if (cb) {
    cb(msg.svgs);
  }
}
```

### 2-5. `ready` 핸들러에 pendingDebugOverlay 처리 추가

```typescript
if (this.pendingDebugOverlay.delete(key)) {
  setTimeout(() => webview.postMessage({ type: "exportDebugOverlay" }), 500);
}
```

---

## 단계 3: extension.ts — `cmdDebugOverlay` webview 위임 방식으로 전환

**파일**: `rhwp-vscode/src/extension.ts`

기존 `cmdDebugOverlay(uri, extensionPath)` 함수를 `cmdDebugOverlay(uri, provider)` 로 변경.
WASM 직접 로드 코드 제거, webview에 위임:

```typescript
async function cmdDebugOverlay(uri: vscode.Uri, provider: HwpEditorProvider): Promise<void> {
  await provider.sendDebugOverlay(uri, async (svgs) => {
    const baseName = path.basename(uri.fsPath);
    const hash = crypto.createHash("md5").update(uri.fsPath).digest("hex").slice(0, 8);
    const tmpFile = path.join(os.tmpdir(), `rhwp-debug-${hash}.html`);

    const pageHtml = svgs
      .map((svg, i) =>
        `<div class="page"><div class="page-label">Page ${i + 1}</div>${svg}</div>`
      )
      .join("\n");

    const html = `<!DOCTYPE html>
<html lang="ko">
<head>
<meta charset="UTF-8">
<title>Debug Overlay — ${baseName}</title>
<style>
  body { background: #555; margin: 0; padding: 16px; font-family: sans-serif; }
  .page { background: white; margin: 0 auto 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.4); width: fit-content; position: relative; }
  .page-label { position: absolute; top: 4px; right: 8px; font-size: 11px; color: #888; background: rgba(255,255,255,0.8); padding: 1px 4px; border-radius: 3px; }
  svg { display: block; }
</style>
</head>
<body>
${pageHtml}
</body>
</html>`;

    fs.writeFileSync(tmpFile, html, "utf8");
    vscode.commands.executeCommand("vscode.open", vscode.Uri.file(tmpFile));
  });
}
```

`activate()` 내 커맨드 등록부도 수정:
```typescript
// 기존
await cmdDebugOverlay(target, context.extensionPath);
// 변경
await cmdDebugOverlay(target, provider);
```

---

## 완료 기준

- 디버그 오버레이 HTML의 표 안 글자 배치가 뷰어와 동일하게 렌더링됨
- 뷰어 미오픈 상태에서 커맨드 실행 시 자동으로 열고 오버레이 생성됨
- 에러 발생 시 VSCode 알림으로 표시됨
