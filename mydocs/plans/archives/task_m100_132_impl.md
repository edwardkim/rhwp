# 구현 계획서 — Task #132

**이슈**: [#132](https://github.com/edwardkim/rhwp/issues/132)
**타이틀**: VS Code 익스텐션 컨텍스트 메뉴 — 인쇄 / SVG 내보내기 / 디버그 오버레이 / 문단 덤프
**마일스톤**: M100
**브랜치**: `local/task132`
**작성일**: 2026-04-13

---

## 코드 파악 결과

### 현재 구조

```
rhwp-vscode/
├── src/
│   ├── extension.ts           # activate(): HwpEditorProvider.register()만
│   ├── hwp-editor-provider.ts # CustomReadonlyEditorProvider, webview 관리
│   └── webview/
│       └── viewer.ts          # WASM 초기화 + canvas 렌더링 (webview 컨텍스트)
├── webpack.config.js          # extensionConfig(node) + webviewConfig(web) 분리
└── package.json               # contributes: customEditors만 있음
```

### 빌드 구조

- **extensionConfig** (`target: "node"`): `src/extension.ts` → `dist/extension.js`
  - `src/webview/` 제외 (`exclude: [/src\/webview/]`)
- **webviewConfig** (`target: "web"`): `src/webview/viewer.ts` → `dist/webview/viewer.js`
  - `@rhwp-wasm` alias → `../pkg`
  - WASM 파일: `dist/media/rhwp_bg.wasm` (CopyPlugin)

### 핵심 WASM API (문단 덤프에 사용)

```typescript
doc.getSectionCount(): number
doc.getParagraphCount(sec: number): number
doc.getParaPropertiesAt(sec: number, para: number): string  // JSON: ParaShape 정보
doc.getLineInfo(sec: number, para: number, char_offset: number): string  // JSON: LINE_SEG 정보
doc.getTableProperties(sec, para, ci): string  // 표 속성 (표 문단인 경우)
```

### measureTextWidth (extension host 처리)

webview의 `installMeasureTextWidth()`는 Canvas API를 사용.
extension host(Node.js)에서는 Canvas API 없음 → **SVG 내보내기 시 stub 등록 필요**:
```typescript
(globalThis as any).measureTextWidth = (_font: string, text: string): number => text.length * 8;
```
(SVG는 viewBox 기반이므로 정확한 측정 불필요, 렌더링 품질에 영향 없음)

---

## 단계별 구현 계획

### 1단계: package.json — commands + menus 등록

**파일**: `rhwp-vscode/package.json`

4개 커맨드를 `contributes.commands`와 `contributes.menus`에 추가한다.

```json
"commands": [
  { "command": "rhwp.print",         "title": "HWP: 인쇄" },
  { "command": "rhwp.exportSvg",     "title": "HWP: SVG로 내보내기" },
  { "command": "rhwp.debugOverlay",  "title": "HWP: 디버그 오버레이 보기" },
  { "command": "rhwp.dumpParagraph", "title": "HWP: 문단 덤프" }
]
```

`menus`: `explorer/context` + `editor/title/context` 양쪽에 동일하게 등록.
`when` 조건: `resourceExtname == .hwp || resourceExtname == .hwpx`
`group`: `"rhwp@1"` ~ `"rhwp@4"`

---

### 2단계: HwpEditorProvider 개선 — 패널 추적 + 인쇄

**파일**: `rhwp-vscode/src/hwp-editor-provider.ts`

**변경 내용**:

1. 파일 URI → 패널 Map 추가:
   ```typescript
   private readonly panels = new Map<string, vscode.WebviewPanel>();
   ```

2. `resolveCustomEditor()`에서 패널 등록/해제:
   ```typescript
   const key = document.uri.toString();
   this.panels.set(key, webviewPanel);
   webviewPanel.onDidDispose(() => this.panels.delete(key));
   ```

3. `openOrFocus()` 메서드 추가: 이미 열린 패널이면 포커스, 없으면 커맨드로 열기

4. `sendPrint(uri: vscode.Uri)` 메서드 추가:
   - 패널이 있으면 `webview.postMessage({ type: 'print' })` 전송
   - 패널이 없으면 `vscode.commands.executeCommand('vscode.openWith', uri, 'rhwp.hwpViewer')` 후 ready 수신 시 print 전송

**파일**: `rhwp-vscode/src/webview/viewer.ts`

`window.addEventListener('message', ...)` 핸들러에 `print` 타입 추가:
```typescript
if (msg.type === 'print') {
  window.print();
}
```

---

### 3단계: SVG 내보내기 커맨드 구현

**파일**: `rhwp-vscode/src/extension.ts` (또는 신규 `src/commands/export-svg.ts`)

**구현 흐름**:

1. `rhwp.exportSvg` 커맨드 등록 (uri 인자 수신)
2. 출력 폴더 선택 (기본: 파일과 동일 폴더):
   ```typescript
   const folder = await vscode.window.showOpenDialog({
     defaultUri: vscode.Uri.file(path.dirname(uri.fsPath)),
     canSelectFolders: true, canSelectFiles: false, openLabel: '이 폴더에 저장'
   });
   ```
3. extension host에서 WASM 초기화:
   ```typescript
   import { initSync, HwpDocument } from '../../pkg/rhwp.js';
   // measureTextWidth stub 등록
   (globalThis as any).measureTextWidth ??= (_: string, t: string) => t.length * 8;
   const wasmPath = path.join(context.extensionPath, 'dist', 'media', 'rhwp_bg.wasm');
   const wasmBuf = fs.readFileSync(wasmPath);
   initSync({ module: wasmBuf });
   ```
4. HWP 파일 로드 → 페이지 수 확인
5. progress notification으로 페이지별 SVG 저장:
   ```typescript
   vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, ... }, async (progress) => {
     for (let i = 0; i < pageCount; i++) {
       const svg = doc.renderPageSvg(i);
       const outPath = path.join(outDir, `${baseName}_p${i + 1}.svg`);
       fs.writeFileSync(outPath, svg, 'utf8');
       progress.report({ increment: 100 / pageCount, message: `${i+1}/${pageCount} 페이지` });
     }
   });
   ```
6. 완료 알림 + "폴더 열기" 버튼:
   ```typescript
   vscode.window.showInformationMessage(`SVG ${pageCount}개 저장 완료`, '폴더 열기')
     .then(sel => { if (sel) vscode.commands.executeCommand('revealFileInOS', outDirUri); });
   ```

**webpack.config.js 변경**: extensionConfig에 `../../pkg` 경로의 `.js` 번들 허용.
WASM은 이미 `dist/media/rhwp_bg.wasm`에 복사되므로 별도 추가 불필요.

---

### 4단계: 디버그 오버레이 + 문단 덤프 커맨드 구현

**디버그 오버레이 (`rhwp.debugOverlay`)**:

```typescript
// WASM 로드 (3단계와 동일 초기화)
doc.set_debug_overlay(true);
const svgs: string[] = [];
for (let i = 0; i < pageCount; i++) {
  svgs.push(doc.renderPageSvg(i));
}
doc.set_debug_overlay(false);

// 전 페이지를 하나의 HTML로 합쳐 임시 파일 저장
const tmpDir = os.tmpdir();
const tmpFile = path.join(tmpDir, `rhwp-debug-${hash}.html`);
const html = buildDebugHtml(svgs);   // 각 SVG를 <div>로 감싼 HTML
fs.writeFileSync(tmpFile, html, 'utf8');
vscode.commands.executeCommand('vscode.open', vscode.Uri.file(tmpFile));
```

임시 HTML은 각 SVG 페이지를 세로로 나열. VS Code 내장 HTML 미리보기로 즉시 확인.

**문단 덤프 (`rhwp.dumpParagraph`)**:

```typescript
// Output 채널 (1회 생성 후 재사용)
const dumpChannel = vscode.window.createOutputChannel('HWP Dump');

// WASM 로드
const secCount = doc.getSectionCount();

// 섹션 선택 (QuickPick)
const secItems = Array.from({ length: secCount }, (_, i) => ({
  label: `섹션 ${i}`, description: `문단 수: ${doc.getParagraphCount(i)}`
}));
const secPick = await vscode.window.showQuickPick(secItems, { placeHolder: '섹션 선택' });
const sec = secItems.indexOf(secPick);

// 문단 선택 (QuickPick)
const paraCount = doc.getParagraphCount(sec);
const paraItems = Array.from({ length: paraCount }, (_, i) => ({ label: `문단 ${i}` }));
const paraPick = await vscode.window.showQuickPick(paraItems, { placeHolder: '문단 선택' });
const para = paraItems.indexOf(paraPick);

// 정보 조회
const paraProps = JSON.parse(doc.getParaPropertiesAt(sec, para));
const lineInfo  = JSON.parse(doc.getLineInfo(sec, para, 0));

// Output 채널 출력 (CLI dump와 동일 포맷)
dumpChannel.clear();
dumpChannel.appendLine(`--- 문단 ${sec}.${para} ---`);
dumpChannel.appendLine(formatParaProps(paraProps));
dumpChannel.appendLine(formatLineInfo(lineInfo));
dumpChannel.show();
```

---

### 5단계: webpack 빌드 설정 + 검증

**webpack.config.js 변경**:

extensionConfig에 `pkg/` WASM JS 번들 포함을 위한 설정 추가:
```javascript
resolve: {
  extensions: ['.ts', '.js'],
  alias: {
    '@rhwp-wasm': path.resolve(__dirname, '..', 'pkg'),
  },
},
```

WASM 파일은 이미 CopyPlugin으로 `dist/media/`에 복사되므로 추가 불필요.

**검증 체크리스트**:
- [ ] `npm run build` 빌드 성공 (타입 에러 없음)
- [ ] 탐색기에서 .hwp 파일 우클릭 → 4개 메뉴 노출
- [ ] 에디터 탭 우클릭 → 4개 메뉴 노출
- [ ] `rhwp.print`: webview `window.print()` 호출 확인
- [ ] `rhwp.exportSvg`: SVG 파일 생성 + progress + 완료 알림 확인
- [ ] `rhwp.debugOverlay`: 임시 HTML 파일 열기 확인
- [ ] `rhwp.dumpParagraph`: QuickPick 섹션/문단 선택 → Output 채널 출력 확인
- [ ] 최종 결과 보고서 작성

---

## 파일 변경 목록

| 파일 | 변경 유형 | 내용 |
|------|-----------|------|
| `rhwp-vscode/package.json` | 수정 | commands + menus 추가 |
| `rhwp-vscode/src/extension.ts` | 수정 | 4개 커맨드 핸들러 등록 |
| `rhwp-vscode/src/hwp-editor-provider.ts` | 수정 | panels Map + sendPrint() |
| `rhwp-vscode/src/webview/viewer.ts` | 수정 | print 메시지 처리 |
| `rhwp-vscode/webpack.config.js` | 수정 | extensionConfig에 alias 추가 |

커맨드 구현이 길어질 경우 `src/commands/` 폴더로 분리.

---

## 승인 요청

위 구현 계획서를 검토 후 승인해주시면 1단계부터 구현을 시작하겠습니다.
