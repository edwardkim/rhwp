---
kind: decision
status: historical
canonical: rhwp-vscode/README.md
last_verified: 2026-07-17
---

# rhwp VSCode 확장 설계 문서

## 1. 개요

VSCode에서 HWP/HWPX 파일을 직접 열어볼 수 있는 읽기 전용 뷰어 확장.
기존 rhwp WASM 렌더링 파이프라인을 재활용하여 rhwp-studio와 동일한 품질의 문서 렌더링을 제공한다.

## 2. 아키텍처

```
┌─────────────────────────────────────────────────┐
│ Extension Host (Node.js)                        │
│                                                 │
│  extension.ts → HwpEditorProvider               │
│    ├─ openCustomDocument(): 파일 URI 저장        │
│    ├─ resolveCustomEditor(): Webview 생성        │
│    │    ├─ Webview HTML + CSP 설정               │
│    │    └─ onDidReceiveMessage('ready')          │
│    │         ├─ workspace.fs.readFile(hwp)       │
│    │         ├─ workspace.fs.readFile(wasm)      │
│    │         └─ postMessage({ fileData, wasmData })│
│    └─ 메시지 프로토콜 관리                         │
├─────────────────────────────────────────────────┤
│ Webview (Browser Sandbox)                       │
│                                                 │
│  viewer.js (webpack 번들 = viewer.ts + rhwp.js) │
│    ├─ initSync(wasmBuffer)                      │
│    ├─ new HwpDocument(hwpBytes)                 │
│    ├─ 가상 스크롤 (플레이스홀더 + on-demand 렌더) │
│    ├─ renderPageToCanvas(pageNum, canvas, scale) │
│    └─ Ctrl+Wheel 줌                             │
└─────────────────────────────────────────────────┘
```

## 3. 메시지 프로토콜

### Host → Webview

| type | 필드 | 설명 |
|------|------|------|
| `load` | `fileName: string` | 파일명 |
| | `fileData: number[]` | HWP 파일 바이너리 (Uint8Array → Array) |
| | `wasmData: number[]` | WASM 바이너리 (Uint8Array → Array) |

### Webview → Host

| type | 필드 | 설명 |
|------|------|------|
| `ready` | — | Webview 초기화 완료, 데이터 요청 |
| `loaded` | `pageCount: number` | 문서 로드 완료, 총 페이지 수 |

## 4. WASM 로딩 전략

### 선택: `initSync` + postMessage 바이너리 전달

```
Extension Host                          Webview
     │                                     │
     │  ← postMessage({ type: 'ready' })   │
     │                                     │
     │  workspace.fs.readFile(wasm) ────►  │
     │  workspace.fs.readFile(hwp)  ────►  │
     │                                     │
     │  postMessage({ wasmData, fileData })│
     │  ──────────────────────────────────► │
     │                                     │
     │                    initSync(wasmBuffer)
     │                    new HwpDocument(hwpBytes)
     │                    renderPageToCanvas()
```

**이유**:
- `asWebviewUri`는 `.wasm` 파일에 `application/wasm` MIME 타입을 설정하지 않아 `WebAssembly.instantiateStreaming()` 실패
- `initSync`는 `fetch`/`import.meta.url`을 사용하지 않아 webpack 번들에 안전
- VSCode 1.82+에서 `Uint8Array` postMessage 전송이 효율적 (structured cloning)

### CSP 설정

```
default-src 'none';
script-src 'nonce-{nonce}' {cspSource};
style-src 'nonce-{nonce}' {cspSource};
img-src {cspSource} data:;
font-src {cspSource};
connect-src {cspSource};
wasm-unsafe-eval
```

`wasm-unsafe-eval`이 핵심 — `WebAssembly.instantiate()`를 허용한다.

## 5. 디렉토리 구조

```
rhwp-vscode/
├── package.json              # 확장 매니페스트
├── tsconfig.json             # Extension Host TypeScript
├── tsconfig.webview.json     # Webview TypeScript
├── webpack.config.js         # 이중 번들 (host + webview)
├── .vscodeignore             # 배포 제외 목록
├── .gitignore
├── src/
│   ├── extension.ts          # activate/deactivate 진입점
│   ├── hwp-editor-provider.ts # CustomReadonlyEditorProvider 구현
│   └── webview/
│       └── viewer.ts         # Webview 뷰어 (WASM 초기화 + 렌더링)
└── dist/                     # 빌드 출력 (git 제외)
    ├── extension.js          # Extension Host 번들
    ├── webview/viewer.js     # Webview 번들 (rhwp.js 포함)
    └── media/rhwp_bg.wasm    # WASM 바이너리 (CopyPlugin)
```

## 6. 빌드 파이프라인

```
pkg/rhwp.js ──────┐
                   ├─ webpack (webview) ─→ dist/webview/viewer.js
src/webview/*.ts ──┘
                       CopyPlugin ───────→ dist/media/rhwp_bg.wasm
pkg/rhwp_bg.wasm ──────┘

src/extension.ts ─────── webpack (host) ──→ dist/extension.js
src/hwp-editor-provider.ts ──┘
```

### webpack 구성 포인트

| 항목 | 설정 |
|------|------|
| Extension Host | `target: 'node'`, `externals: { vscode }` |
| Webview | `target: 'web'`, `alias: { '@rhwp-wasm': '../pkg' }` |
| WASM 파일 | `null-loader` (webpack의 자동 asset 처리 비활성화) |
| WASM 복사 | `copy-webpack-plugin` → `dist/media/` |

## 7. 가상 스크롤

대용량 문서 (수백 페이지)에서도 원활한 스크롤을 위해 가상 스크롤을 적용한다.

### 동작 방식

1. **초기화**: 전체 페이지에 대해 빈 `div` 플레이스홀더 생성 (실제 크기 설정)
2. **스크롤**: 뷰포트 + 여유분(300px) 내의 페이지만 `renderPageToCanvas()` 호출
3. **해제**: 뷰포트를 벗어난 페이지의 Canvas 제거 → 메모리 절약
4. **줌**: Ctrl+마우스 휠로 0.25x ~ 3.0x 줌. 스크롤 비율 보존

### rhwp-studio와의 차이

| 항목 | rhwp-studio | rhwp-vscode |
|------|-------------|-------------|
| 가상 스크롤 | canvas-view.ts + virtual-scroll.ts | viewer.ts 단일 파일 |
| 페이지 풀링 | 있음 (pool/release) | 플레이스홀더 + on-demand |
| 편집 지원 | InputHandler 등 | 없음 (읽기 전용) |
| 폰트 로딩 | font-loader.ts | WASM 내장 (시스템 폰트) |

## 8. rhwp-studio와의 분리 원칙

- `rhwp-vscode/`는 **독립 패키지**. `rhwp-studio/` 코드를 import하지 않는다.
- 유일한 공통 의존성: `pkg/` (WASM 빌드 출력)
- Webview 코드(가상 스크롤 등)는 rhwp-studio를 참조하되 독립 구현
- 각 패키지의 빌드 도구가 다름: `rhwp-studio`=Vite, `rhwp-vscode`=webpack

## 9. 향후 확장 로드맵

### v0.2 — 편집 지원

- `CustomReadonlyEditorProvider` → `CustomEditorProvider` 전환
- 텍스트 입력, 커서, 선택 영역
- Undo/Redo

### v0.3 — 부가 기능

- 텍스트 검색 (Ctrl+F)
- 아웃라인 뷰 (제목/목차 트리)
- SVG/PDF 내보내기 명령
- 썸네일 사이드바

### v0.4 — 배포

- VSCode Marketplace 게시
- `.vsix` 패키징 자동화 (CI/CD)
- 웹 VSCode (vscode.dev) 지원
