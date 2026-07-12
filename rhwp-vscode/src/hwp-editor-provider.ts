import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

export class HwpEditorProvider implements vscode.CustomReadonlyEditorProvider {
  private static readonly viewType = "rhwp.hwpViewer";

  /** 파일 URI 문자열 → 열린 WebviewPanel 추적 */
  private readonly panels = new Map<string, vscode.WebviewPanel>();

  static register(context: vscode.ExtensionContext): { provider: HwpEditorProvider; disposable: vscode.Disposable } {
    const provider = new HwpEditorProvider(context);
    const disposable = vscode.window.registerCustomEditorProvider(
      HwpEditorProvider.viewType,
      provider,
      {
        webviewOptions: { retainContextWhenHidden: true },
        supportsMultipleEditorsPerDocument: false,
      }
    );
    return { provider, disposable };
  }

  constructor(private readonly context: vscode.ExtensionContext) {}

  /** 해당 파일의 webview에 디버그 오버레이 렌더링 요청 */
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

  /** 해당 파일의 webview에 SVG 내보내기 요청 */
  async sendExportSvg(uri: vscode.Uri, onSvgs: (svgs: string[]) => void): Promise<void> {
    const key = uri.toString();
    const panel = this.panels.get(key);
    this.exportSvgCallbacks.set(key, onSvgs);
    if (panel) {
      panel.reveal();
      panel.webview.postMessage({ type: "exportSvg" });
    } else {
      await vscode.commands.executeCommand("vscode.openWith", uri, HwpEditorProvider.viewType);
      this.pendingExportSvg.add(key);
    }
  }

  /** 열린 직후 SVG 내보내기를 해야 할 URI 집합 */
  private readonly pendingExportSvg = new Set<string>();

  /** 열린 직후 디버그 오버레이를 내보내야 할 URI 집합 */
  private readonly pendingDebugOverlay = new Set<string>();

  /** SVG 내보내기 응답 콜백 */
  private readonly exportSvgCallbacks = new Map<string, (svgs: string[]) => void>();

  /** 디버그 오버레이 SVG 응답 콜백 */
  private readonly debugOverlayCallbacks = new Map<string, (svgs: string[]) => void>();

  async openCustomDocument(
    uri: vscode.Uri,
    _openContext: vscode.CustomDocumentOpenContext,
    _token: vscode.CancellationToken
  ): Promise<vscode.CustomDocument> {
    return { uri, dispose: () => {} };
  }

  async resolveCustomEditor(
    document: vscode.CustomDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    const webview = webviewPanel.webview;

    // 패널 추적 등록
    const key = document.uri.toString();
    this.panels.set(key, webviewPanel);
    webviewPanel.onDidDispose(() => this.panels.delete(key));

    webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this.context.extensionUri, "dist"),
      ],
    };

    webview.html = this.getHtml(webview);

    // Webview ready 후 HWP 파일 데이터만 전송 (WASM은 Webview에서 fetch)
    webview.onDidReceiveMessage(async (msg) => {
      if (msg.type === "ready") {
        const fileData = await vscode.workspace.fs.readFile(document.uri);
        const fileName = document.uri.path.split("/").pop() ?? "";

        webview.postMessage({
          type: "load",
          fileName,
          fileData: new Uint8Array(fileData),
        });

        // 열리자마자 SVG 내보내기를 해야 하는 경우
        if (this.pendingExportSvg.delete(key)) {
          setTimeout(() => webview.postMessage({ type: "exportSvg" }), 500);
        }
        // 열리자마자 디버그 오버레이를 내보내야 하는 경우
        if (this.pendingDebugOverlay.delete(key)) {
          setTimeout(() => webview.postMessage({ type: "exportDebugOverlay" }), 500);
        }
      }

      if (msg.type === "exportSvgDone") {
        const cb = this.exportSvgCallbacks.get(key);
        this.exportSvgCallbacks.delete(key);
        if (msg.error) {
          vscode.window.showErrorMessage(`SVG 내보내기 실패: ${msg.error}`);
        } else if (cb) {
          cb(msg.svgs);
        } else {
          // 뷰어 내부 우클릭으로 요청된 경우 — 콜백 없이 직접 폴더 선택 → 저장
          const defaultDir = vscode.Uri.file(
            require("path").dirname(document.uri.fsPath)
          );
          const folders = await vscode.window.showOpenDialog({
            defaultUri: defaultDir,
            canSelectFolders: true,
            canSelectFiles: false,
            canSelectMany: false,
            openLabel: "이 폴더에 SVG 저장",
          });
          if (!folders || folders.length === 0) return;
          const outDir = folders[0].fsPath;
          const baseName = require("path").basename(
            document.uri.fsPath,
            require("path").extname(document.uri.fsPath)
          );
          const fs = require("fs");
          for (let i = 0; i < msg.svgs.length; i++) {
            fs.writeFileSync(
              require("path").join(outDir, `${baseName}_p${i + 1}.svg`),
              msg.svgs[i],
              "utf8"
            );
          }
          const sel = await vscode.window.showInformationMessage(
            `SVG ${msg.svgs.length}개 저장 완료 → ${outDir}`,
            "폴더 열기"
          );
          if (sel === "폴더 열기") {
            vscode.commands.executeCommand("revealFileInOS", vscode.Uri.file(outDir));
          }
        }
      }

      if (msg.type === "debugOverlaySvgs") {
        const cb = this.debugOverlayCallbacks.get(key);
        this.debugOverlayCallbacks.delete(key);
        if (msg.error) {
          vscode.window.showErrorMessage(`디버그 오버레이 실패: ${msg.error}`);
        } else if (cb) {
          cb(msg.svgs);
        }
      }
    });
  }

  private getHtml(webview: vscode.Webview): string {
    const viewerUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, "dist", "webview", "viewer.js")
    );
    const wasmUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, "dist", "media", "rhwp_bg.wasm")
    );
    const fontsBase = webview.asWebviewUri(
      vscode.Uri.joinPath(this.context.extensionUri, "dist", "media", "fonts")
    );

    const nonce = getNonce();
    const cspSource = webview.cspSource;

    // 폰트 매핑: [CSS font-family, woff2 파일명, format]
    const fontEntries: [string, string, string][] = [
      // 함초롬체 CDN (woff)
      ['함초롬바탕', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff', 'woff'],
      ['함초롬돋움', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff', 'woff'],
      ['함초롱바탕', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff', 'woff'],
      ['함초롱돋움', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff', 'woff'],
      ['한컴바탕', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff', 'woff'],
      ['한컴돋움', 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_four@1.0/HCRDotum.woff', 'woff'],
      // 오픈소스 로컬 (woff2)
      ['Noto Serif KR', `${fontsBase}/NotoSerifKR-Regular.woff2`, 'woff2'],
      ['Noto Sans KR', `${fontsBase}/NotoSansKR-Regular.woff2`, 'woff2'],
      ['Pretendard', `${fontsBase}/Pretendard-Regular.woff2`, 'woff2'],
      ['D2Coding', `${fontsBase}/D2Coding-Regular.woff2`, 'woff2'],
      ['나눔고딕', `${fontsBase}/NanumGothic-Regular.woff2`, 'woff2'],
      ['나눔명조', `${fontsBase}/NanumMyeongjo-Regular.woff2`, 'woff2'],
      ['고운바탕', `${fontsBase}/GowunBatang-Regular.woff2`, 'woff2'],
      ['고운돋움', `${fontsBase}/GowunDodum-Regular.woff2`, 'woff2'],
      // HY 폰트 → Noto 대체
      ['HY헤드라인M', `${fontsBase}/NotoSansKR-Bold.woff2`, 'woff2'],
      ['HY견명조', `${fontsBase}/NotoSerifKR-Bold.woff2`, 'woff2'],
      ['HY신명조', `${fontsBase}/NotoSerifKR-Regular.woff2`, 'woff2'],
      ['HY그래픽', `${fontsBase}/NotoSansKR-Regular.woff2`, 'woff2'],
      ['휴먼명조', `${fontsBase}/NotoSerifKR-Regular.woff2`, 'woff2'],
      // 시스템 폰트 → 오픈소스 대체
      ['맑은 고딕', `${fontsBase}/Pretendard-Regular.woff2`, 'woff2'],
      ['바탕', `${fontsBase}/NotoSerifKR-Regular.woff2`, 'woff2'],
      ['돋움', `${fontsBase}/NotoSansKR-Regular.woff2`, 'woff2'],
      ['굴림', `${fontsBase}/NotoSansKR-Regular.woff2`, 'woff2'],
      ['굴림체', `${fontsBase}/D2Coding-Regular.woff2`, 'woff2'],
      ['바탕체', `${fontsBase}/D2Coding-Regular.woff2`, 'woff2'],
      ['궁서', `${fontsBase}/GowunBatang-Regular.woff2`, 'woff2'],
    ];
    const fontFaceCSS = fontEntries.map(([name, url, fmt]) =>
      `@font-face { font-family: "${name}"; src: url("${url}") format("${fmt}"); font-display: swap; }`
    ).join('\n    ');

    return /* html */ `<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy"
    content="default-src 'none';
             script-src 'nonce-${nonce}' ${cspSource} 'unsafe-eval' 'wasm-unsafe-eval';
             style-src 'nonce-${nonce}' ${cspSource};
             img-src ${cspSource} data:;
             font-src ${cspSource} https://cdn.jsdelivr.net;
             connect-src ${cspSource}">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>HWP Viewer</title>
  <style nonce="${nonce}">
    ${fontFaceCSS}
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      background: var(--vscode-editor-background);
      color: var(--vscode-editor-foreground);
      overflow: hidden;
      height: 100vh;
      display: flex;
      flex-direction: column;
    }
    #scroll-container {
      position: relative;
      overflow: auto;
      flex: 1;
      gap: 12px;
      padding: 12px 0;
    }
    #scroll-content {
      display: flex;
      flex-direction: column;
      align-items: center;
      gap: 12px;
      min-width: fit-content;
    }
    .page-wrapper {
      flex-shrink: 0;
      box-shadow: 0 2px 8px rgba(0,0,0,0.3);
      background: white;
    }
    /* 2쪽 보기: 두 쪽을 좌우로 배치 */
    .page-row {
      display: flex;
      flex-direction: row;
      gap: 12px;
      align-items: flex-start;
    }
    /* 네비게이션 사이드바 (nav-) */
    #app-shell {
      flex: 1;
      display: flex;
      flex-direction: row;
      min-height: 0;
      overflow: hidden;
      position: relative;
    }
    #nav-sidebar {
      width: 240px;
      flex-shrink: 0;
      display: flex;
      flex-direction: column;
      background: var(--vscode-sideBar-background, #252526);
      border-right: 1px solid var(--vscode-sideBar-border, rgba(255,255,255,0.1));
      overflow: hidden;
      transition: width 0.15s ease;
    }
    #nav-sidebar.collapsed {
      width: 0;
      border-right: none;
    }
    /* 접혔을 때 편집영역 좌측에 나타나는 열기 버튼 */
    #nav-reopen {
      position: absolute;
      left: 0;
      top: 50%;
      transform: translateY(-50%);
      z-index: 10;
      display: none;
      width: 18px;
      height: 44px;
      border: none;
      border-radius: 0 6px 6px 0;
      background: var(--vscode-sideBar-background, #252526);
      color: var(--vscode-sideBar-foreground, #ccc);
      cursor: pointer;
      box-shadow: 1px 0 4px rgba(0,0,0,0.3);
      font-size: 12px;
      line-height: 44px;
      padding: 0;
    }
    #nav-reopen:hover { background: rgba(255,255,255,0.12); }
    #app-shell.sidebar-collapsed #nav-reopen { display: block; }
    #nav-tabs {
      display: flex;
      flex-shrink: 0;
      align-items: center;
      border-bottom: 1px solid var(--vscode-sideBar-border, rgba(255,255,255,0.1));
    }
    #nav-collapse {
      flex-shrink: 0;
      width: 26px;
      align-self: stretch;
      border: none;
      background: transparent;
      color: var(--vscode-sideBar-foreground, #999);
      cursor: pointer;
      font-size: 12px;
    }
    #nav-collapse:hover { background: rgba(255,255,255,0.06); }
    .nav-tab {
      flex: 1;
      padding: 6px 4px;
      border: none;
      background: transparent;
      color: var(--vscode-sideBar-foreground, #ccc);
      cursor: pointer;
      font-size: 12px;
      border-bottom: 2px solid transparent;
      white-space: nowrap;
    }
    .nav-tab.active {
      border-bottom-color: var(--vscode-focusBorder, #007acc);
      color: var(--vscode-foreground, #fff);
    }
    .nav-tab:hover { background: rgba(255,255,255,0.06); }
    #nav-body {
      flex: 1;
      overflow-y: auto;
      overflow-x: hidden;
      min-height: 0;
    }
    .nav-panel { padding: 8px; }
    .nav-panel[hidden] { display: none; }
    .nav-thumb {
      display: flex;
      flex-direction: column;
      align-items: center;
      margin-bottom: 10px;
      cursor: pointer;
      padding: 4px;
      border-radius: 4px;
    }
    .nav-thumb:hover { background: rgba(255,255,255,0.06); }
    .nav-thumb.current { background: var(--vscode-list-activeSelectionBackground, rgba(0,122,204,0.35)); }
    .nav-thumb canvas {
      background: white;
      box-shadow: 0 1px 4px rgba(0,0,0,0.4);
      max-width: 100%;
      display: block;
    }
    .nav-thumb-label {
      margin-top: 3px;
      font-size: 11px;
      color: var(--vscode-descriptionForeground, #999);
    }
    .nav-item {
      padding: 4px 6px;
      cursor: pointer;
      font-size: 12px;
      border-radius: 3px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      color: var(--vscode-sideBar-foreground, #ccc);
    }
    .nav-item:hover { background: rgba(255,255,255,0.06); }
    .nav-empty {
      padding: 12px 8px;
      font-size: 12px;
      color: var(--vscode-descriptionForeground, #999);
      text-align: center;
    }
    /* 상태 표시줄 */
    #status-bar {
      display: flex;
      align-items: center;
      height: 26px;
      padding: 0 10px;
      background: var(--vscode-statusBar-background, #007acc);
      border-top: 1px solid var(--vscode-statusBar-border, transparent);
      flex-shrink: 0;
      font-size: 12px;
      color: var(--vscode-statusBar-foreground, #fff);
      user-select: none;
    }
    .stb-item {
      line-height: 26px;
      white-space: nowrap;
      flex-shrink: 0;
      padding: 0 4px;
    }
    .stb-divider {
      width: 1px;
      height: 14px;
      background: var(--vscode-statusBar-foreground, #fff);
      opacity: 0.3;
      margin: 0 6px;
      flex-shrink: 0;
    }
    .stb-message {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      padding: 0 8px;
      line-height: 26px;
      opacity: 0.8;
    }
    .stb-right {
      display: flex;
      align-items: center;
      gap: 2px;
      flex-shrink: 0;
      margin-left: auto;
    }
    .stb-btn {
      height: 22px;
      border: none;
      border-radius: 3px;
      background: transparent;
      color: inherit;
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      padding: 0 6px;
      font-size: 14px;
      line-height: 1;
    }
    .stb-btn:hover {
      background: rgba(255,255,255,0.15);
    }
    .stb-zoom-val {
      font-size: 12px;
      min-width: 40px;
      text-align: center;
      line-height: 26px;
    }
  </style>
</head>
<body>
  <div id="app-shell">
    <div id="nav-sidebar">
      <div id="nav-tabs">
        <button class="nav-tab active" data-tab="thumb" title="\uc378\ub124\uc77c">\uc378\ub124\uc77c</button>
        <button class="nav-tab" data-tab="outline" title="\ubaa9\ucc28">\ubaa9\ucc28</button>
        <button class="nav-tab" data-tab="bookmark" title="\ubd81\ub9c8\ud06c">\ubd81\ub9c8\ud06c</button>
        <button id="nav-collapse" title="\uc0ac\uc774\ub4dc\ubc14 \ub2eb\uae30">\u25c0</button>
      </div>
      <div id="nav-body">
        <div class="nav-panel" data-panel="thumb"></div>
        <div class="nav-panel" data-panel="outline" hidden></div>
        <div class="nav-panel" data-panel="bookmark" hidden></div>
      </div>
    </div>
    <div id="scroll-container" data-wasm-uri="${wasmUri}"><div id="scroll-content"></div></div>
    <button id="nav-reopen" title="\uc0ac\uc774\ub4dc\ubc14 \uc5f4\uae30">\u25b6</button>
  </div>
  <div id="status-bar">
    <button id="stb-sidebar-toggle" class="stb-btn" title="\uc0ac\uc774\ub4dc\ubc14 \uc811\uae30/\ud3bc\uce58\uae30">\u2630</button>
    <span id="stb-page" class="stb-item">- / - \uca4d</span>
    <span class="stb-divider"></span>
    <span id="stb-message" class="stb-message">\ubb38\uc11c\ub97c \ubd88\ub7ec\uc624\ub294 \uc911...</span>
    <span class="stb-right">
      <button id="stb-view-mode" class="stb-btn" title="1\ucabd/2\ucabd \ubcf4\uae30">1\ucabd</button>
      <span class="stb-divider"></span>
      <button id="stb-zoom-out" class="stb-btn" title="\ucd95\uc18c">\u2212</button>
      <span id="stb-zoom-val" class="stb-zoom-val">100%</span>
      <button id="stb-zoom-in" class="stb-btn" title="\ud655\ub300">+</button>
    </span>
  </div>
  <script nonce="${nonce}" src="${viewerUri}"></script>
</body>
</html>`;
  }
}

function getNonce(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let nonce = "";
  for (let i = 0; i < 32; i++) {
    nonce += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return nonce;
}
