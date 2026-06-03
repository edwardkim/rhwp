# 최종 결과 보고서 — Task #132

**이슈**: [#132](https://github.com/edwardkim/rhwp/issues/132)
**타이틀**: VS Code 익스텐션 컨텍스트 메뉴 — 인쇄 / SVG 내보내기 / 디버그 오버레이 / 문단 덤프
**마일스톤**: M100
**브랜치**: `local/task132`
**완료일**: 2026-04-13

---

## 구현 결과

### 추가된 컨텍스트 메뉴 커맨드

탐색기 파일 우클릭 + 에디터 탭 우클릭 시 .hwp/.hwpx 파일에 4개 메뉴 노출:

| 커맨드 | 타이틀 | 기능 |
|--------|--------|------|
| `rhwp.print` | HWP: 인쇄 | webview `window.print()` 호출 |
| `rhwp.exportSvg` | HWP: SVG로 내보내기 | extension host WASM → 페이지별 SVG 저장 |
| `rhwp.debugOverlay` | HWP: 디버그 오버레이 보기 | 문단/표 경계 시각화 HTML 생성 |
| `rhwp.dumpParagraph` | HWP: 문단 덤프 | ParaShape + LINE_SEG → Output 채널 출력 |

### 변경 파일 목록

| 파일 | 내용 |
|------|------|
| `rhwp-vscode/package.json` | commands + menus (explorer/context, editor/title/context) |
| `rhwp-vscode/src/extension.ts` | 4개 커맨드 핸들러 등록 + 3개 커맨드 구현 함수 |
| `rhwp-vscode/src/hwp-editor-provider.ts` | panels Map + pendingPrint Set + sendPrint() + register() 반환 타입 변경 |
| `rhwp-vscode/src/webview/viewer.ts` | print 메시지 처리 (`window.print()`) |
| `rhwp-vscode/src/wasm-host.ts` (신규) | extension host WASM 초기화 헬퍼 |
| `rhwp-vscode/webpack.config.js` | extensionConfig에 @rhwp-wasm alias + wasm null-loader |

### 기술 핵심

**extension host에서 WASM 직접 로드**:
- `fs.readFileSync('dist/media/rhwp_bg.wasm')` → `initSync({ module: wasmBuf })`
- `measureTextWidth` stub (`text.length * 8`): Canvas API 없는 Node.js 환경 대응
- SVG 렌더링은 viewBox 기반이므로 stub 정확도로 충분

**인쇄 흐름**:
- 뷰어 열림 → `panels` Map에서 패널 조회 → `postMessage({ type: "print" })` → `window.print()`
- 뷰어 미열림 → `vscode.openWith` 실행 → `pendingPrint` Set에 등록 → ready 수신 후 500ms 지연 print

**디버그 오버레이**:
- `set_debug_overlay(true)` + `renderPageSvg()` × N페이지
- 전 페이지 SVG를 하나의 HTML로 합쳐 `os.tmpdir()` 임시 파일 저장
- `vscode.open`으로 VS Code 내장 탭에서 즉시 확인

**문단 덤프**:
- `getSectionCount()` / `getParagraphCount()` → QuickPick 2단계 선택
- `getParaPropertiesAt()` + `getLineInfo()` → Output 채널 `"HWP Dump"` 출력
- `dumpChannel.show(true)`: 포커스 이동 없이 채널 표시

### 빌드 검증

```
webpack 5.105.4 compiled successfully
extension.js 71 KiB
tsc --noEmit: 0 errors, 0 warnings
```

---

## 효과

| 기능 | 전 | 후 |
|------|----|----|
| 인쇄 | 터미널에서 별도 도구 필요 | 우클릭 한 번으로 인쇄 다이얼로그 |
| SVG 내보내기 | CLI `rhwp export-svg` 명령 필요 | VS Code에서 바로 SVG 생성 + 저장 |
| 디버그 오버레이 | CLI + 파일 탐색기로 SVG 확인 | 우클릭 → VS Code 탭에서 즉시 확인 |
| 문단 덤프 | CLI `rhwp dump -s N -p M` | QuickPick 선택 → Output 채널 즉시 확인 |
