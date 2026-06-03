# 3단계 완료 보고서 — Task #132

**단계**: 3단계 — rhwp.exportSvg 구현
**완료일**: 2026-04-13

## 변경 파일

- `rhwp-vscode/src/wasm-host.ts` (신규)
- `rhwp-vscode/src/extension.ts` (cmdExportSvg 추가)
- `rhwp-vscode/webpack.config.js` (extensionConfig alias + null-loader 추가)

## 변경 내용

### wasm-host.ts (신규)

extension host(Node.js)에서 WASM을 초기화하는 헬퍼 모듈:

- `initWasmHost(extensionPath)`: 1회 초기화 (중복 호출 안전)
  - `measureTextWidth` stub 등록 (`text.length * 8`) — Canvas API 없는 Node.js 대응
  - `fs.readFileSync(dist/media/rhwp_bg.wasm)` → `initSync({ module: wasmBuf })`
- `HwpDocument` 재내보내기 (`@rhwp-wasm/rhwp.js` require)

### cmdExportSvg

1. `showOpenDialog`로 출력 폴더 선택 (기본: HWP 파일과 동일 폴더)
2. `withProgress` (Notification) — 페이지별 진행 표시
3. `initWasmHost()` → `new HwpDocument(fileBytes)` → `getDocumentInfo()`로 페이지 수 확인
4. 페이지별 `renderPageSvg(i)` → `{baseName}_p{i+1}.svg` 저장
5. 완료 알림 + "폴더 열기" 버튼 (`revealFileInOS`)

### webpack.config.js

- extensionConfig에 `@rhwp-wasm` alias 추가 (`../pkg` 경로)
- extensionConfig에 `.wasm` null-loader 규칙 추가 (WASM은 fs로 직접 읽으므로)

## 빌드 결과

```
extension.js 71 KiB — 빌드 성공
webpack 5.105.4 compiled with 2 warnings (폰트/WASM 크기 경고 — 기존과 동일)
```
