# 최종 결과 보고서 — Task #136

**이슈**: [#136](https://github.com/edwardkim/rhwp/issues/136)
**타이틀**: 디버그 오버레이/SVG 내보내기 webview 위임 렌더링 전환
**마일스톤**: M100
**브랜치**: `local/task136`
**완료일**: 2026-04-14

---

## 문제

`HWP: 디버그 오버레이 보기`, `HWP: SVG로 내보내기` 커맨드가 extension host(Node.js)에서 WASM을 직접 로드하여 렌더링하던 방식으로 인해 표 안의 글자 배치가 뷰어와 다르게 틀어지는 문제 발생.

**원인**: Node.js 환경에 Canvas API가 없어 `measureTextWidth = text.length * 8` 스텁 사용 → 한글 등 가변폭 글자 배치 부정확.

## 해결

extension host에서 직접 렌더링하는 방식을 **webview 위임 방식**으로 전환.

```
extension host → { type: "exportSvg" / "exportDebugOverlay" } → webview
webview → Canvas measureText() 실제 호출 → renderPageSvg() × N
webview → { type: "exportSvgDone" / "debugOverlaySvgs", svgs } → extension host
extension host → 파일 저장 / HTML 생성
```

## 추가 변경

| 항목 | 내용 |
|------|------|
| 인쇄 커맨드 제거 | VSCode webview + WSL 환경에서 `window.print()` / `vscode.env.openExternal()` 모두 미동작 확인 |
| 뷰어 내부 우클릭 | 기본 메뉴(잘라내기/복사/붙여넣기) 억제 |
| webpack 경고 제거 | `performance: { hints: false }` 추가 — WASM/폰트 크기는 의도된 것 |
| 버전 | `0.7.2` → `0.7.3` |

## 수정 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/webview/viewer.ts` | `exportSvg`, `exportDebugOverlay` 메시지 처리 추가, 기본 우클릭 억제 |
| `src/hwp-editor-provider.ts` | `sendExportSvg()`, `sendDebugOverlay()` 메서드 추가, SVG 응답 콜백 처리 |
| `src/extension.ts` | `cmdExportSvg`, `cmdDebugOverlay` webview 위임 방식으로 전환, `cmdPrint` 제거 |
| `webpack.config.js` | `performance: { hints: false }` 추가 |
| `package.json` | 버전 0.7.3, 인쇄 커맨드 제거 |
| `CHANGELOG.md` | v0.7.3 항목 추가 |
| `README.md` | 기능 목록 현행화 |

## 빌드 결과

```
webpack 5.105.4 compiled successfully (경고 0건)
webpack 5.105.4 compiled successfully (경고 0건)
rhwp-vscode-0.7.3.vsix (29 files, 10.12MB)
```

## 테스트 결과

- 탐색기 우클릭 → SVG 내보내기: 폴더 선택 → 파일 저장 ✅
- 탐색기 우클릭 → 디버그 오버레이: 표 안 글자 배치 뷰어와 동일 ✅
- 뷰어 내부 우클릭: 기본 메뉴 억제 ✅
