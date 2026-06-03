# Task 403 완료 보고서: VSCode 확장으로 rhwp 제공하기 설계

## 수행 결과

### 산출물

| 산출물 | 위치 | 설명 |
|--------|------|------|
| VSCode 확장 패키지 | `rhwp-vscode/` | 독립 npm 패키지, webpack 이중 번들 |
| 확장 매니페스트 | `rhwp-vscode/package.json` | `*.hwp`, `*.hwpx` 자동 연결 |
| Extension Host | `rhwp-vscode/src/extension.ts` | 진입점 |
| Editor Provider | `rhwp-vscode/src/hwp-editor-provider.ts` | CustomReadonlyEditorProvider |
| Webview 뷰어 | `rhwp-vscode/src/webview/viewer.ts` | WASM 초기화 + 가상 스크롤 + Canvas 렌더링 |
| 설계 문서 | `mydocs/tech/vscode_extension_design.md` | 아키텍처, 프로토콜, 빌드 파이프라인 |

### 단계별 진행 요약

| 단계 | 내용 | 결과 |
|------|------|------|
| 1 | 프로젝트 스캐폴딩 | `rhwp-vscode/` 생성, package.json + webpack + tsconfig 설정, 빌드 성공 |
| 2 | Custom Editor Provider | HwpEditorProvider 구현, Webview HTML + CSP, 메시지 프로토콜 |
| 3 | WASM 통합 | `initSync` + postMessage 방식, rhwp.js webpack 번들링, null-loader로 .wasm 처리 |
| 4 | 가상 스크롤 | 플레이스홀더 기반 on-demand 렌더링, Ctrl+Wheel 줌 (0.25x~3.0x) |
| 5 | 설계 문서 | 아키텍처, 메시지 프로토콜, WASM 전략, 빌드 구성, 확장 로드맵 문서화 |

### 핵심 기술 결정

1. **WASM 로딩**: `initSync` + ArrayBuffer postMessage (fetch/URL 불필요, CSP 안전)
2. **번들링**: webpack으로 `rhwp.js`를 viewer.js에 포함, `.wasm`은 null-loader + CopyPlugin
3. **가상 스크롤**: 플레이스홀더 div + 뷰포트 기반 on-demand Canvas 렌더링/해제
4. **소스 분리**: `rhwp-studio/`와 완전 독립, 공통 의존성은 `pkg/` (WASM)뿐

## 빌드 확인

```
$ cd rhwp-vscode && npx webpack --mode development
Extension Host: extension.js (7.5KB) — 성공
Webview: viewer.js (221KB, rhwp.js 포함) — 성공
Media: rhwp_bg.wasm (3.14MB, CopyPlugin) — 복사 완료
```
