# Task 403 구현 계획서: VSCode 확장으로 rhwp 제공하기 설계

## 아키텍처 개요

```
┌─────────────────────────────────────────────────────┐
│ VSCode Extension Host                               │
│                                                     │
│  extension.ts                                       │
│    └─ HwpEditorProvider (CustomReadonlyEditorProvider)│
│         ├─ openCustomDocument(): 파일 바이너리 읽기   │
│         └─ resolveCustomEditor(): Webview 생성       │
│              ├─ WASM 바이너리 전송 (postMessage)      │
│              └─ HWP 파일 데이터 전송 (postMessage)    │
├─────────────────────────────────────────────────────┤
│ Webview (샌드박스)                                    │
│                                                     │
│  viewer.ts                                          │
│    ├─ WASM 초기화 (WebAssembly.instantiate)          │
│    ├─ HwpDocument 생성 (파일 바이너리 → 문서 객체)    │
│    ├─ 페이지 렌더링 (Canvas 2D)                      │
│    ├─ 가상 스크롤 (visible pages만 렌더링)            │
│    └─ 줌/네비게이션 UI                               │
└─────────────────────────────────────────────────────┘
```

### 데이터 흐름

```
.hwp 파일 더블클릭
  → Extension Host: workspace.fs.readFile() → Uint8Array
  → postMessage({ type: 'load', wasm: wasmBytes, file: hwpBytes })
  → Webview: WebAssembly.instantiate(wasmBytes)
  → Webview: new HwpDocument(hwpBytes)
  → Webview: renderPageToCanvas(pageNum, canvas, scale)
  → Canvas에 문서 표시
```

## 구현 단계

### 1단계: 프로젝트 스캐폴딩 및 확장 매니페스트

**목표**: `rhwp-vscode/` 독립 패키지 생성, VSCode 확장 기본 구조 구축

**작업 내용**:
- `rhwp-vscode/` 디렉토리 생성
- `package.json` — 확장 매니페스트 작성
  - `contributes.customEditors`: `*.hwp`, `*.hwpx` 등록
  - `viewType`: `rhwp.hwpViewer`
  - `priority`: `default` (기본 뷰어로 동작)
- `tsconfig.json` — TypeScript 설정
- `webpack.config.js` — Extension Host + Webview 이중 번들 설정
  - Extension Host: `target: 'node'`, `externals: { vscode }`
  - Webview: `target: 'web'`, WASM을 `asset/resource`로 처리
- `.vscodeignore` — 배포 시 불필요 파일 제외
- `src/extension.ts` — 확장 진입점 (빈 활성화/비활성화)

**산출물**: `npm install && npm run compile` 성공, VSCode에서 확장 로드 확인

### 2단계: Custom Editor Provider + 파일 로딩

**목표**: HWP 파일을 열면 Webview가 생성되고 파일 데이터가 전달되는 파이프라인 구축

**작업 내용**:
- `src/hwp-editor-provider.ts` — `CustomReadonlyEditorProvider` 구현
  - `openCustomDocument()`: `workspace.fs.readFile(uri)` → `HwpDocument` (확장 측 모델)
  - `resolveCustomEditor()`: Webview HTML 생성, CSP 설정, 파일 데이터 전송
- Webview HTML 템플릿
  - CSP: `script-src ${cspSource}; style-src ${cspSource}; wasm-unsafe-eval`
  - nonce 기반 스크립트 로딩
- `src/webview/viewer.ts` — Webview 진입점
  - `window.addEventListener('message')` → 파일 데이터 수신
  - 수신 확인 메시지 반환 (파일명, 크기 표시)
- Extension Host → Webview 메시지 프로토콜 정의
  ```typescript
  // Host → Webview
  { type: 'load', fileName: string, fileData: Uint8Array }
  // Webview → Host
  { type: 'ready' }
  { type: 'loaded', pageCount: number }
  ```

**산출물**: .hwp 파일 열기 → Webview에 파일명/크기 표시

### 3단계: WASM 통합 및 단일 페이지 렌더링

**목표**: Webview 내에서 WASM을 로드하고 첫 페이지를 Canvas에 렌더링

**작업 내용**:
- WASM 번들링 전략 구현
  - `pkg/rhwp_bg.wasm`을 확장 빌드 시 `dist/media/`에 복사
  - Extension Host에서 WASM 바이너리를 읽어 Webview에 postMessage로 전송
  - Webview에서 `WebAssembly.instantiate()`로 초기화
- `src/webview/wasm-loader.ts` — WASM 초기화 모듈
  - wasm-bindgen 생성 JS(`rhwp.js`)의 init 함수를 WASM 바이트 배열로 호출
  - `HwpDocument` 인스턴스 생성
- `src/webview/page-canvas.ts` — 단일 페이지 Canvas 렌더링
  - `HwpDocument.renderPageToCanvas(pageNum, canvas, scale)` 호출
  - DPI 스케일링 처리 (`window.devicePixelRatio`)
- 페이지 정보 표시 (총 페이지 수, 현재 페이지)

**산출물**: .hwp 파일 열기 → 첫 페이지가 Canvas에 렌더링

### 4단계: 가상 스크롤 및 다중 페이지 뷰

**목표**: 전체 페이지를 스크롤하며 열람할 수 있는 문서 뷰어 완성

**작업 내용**:
- `src/webview/virtual-scroll.ts` — 가상 스크롤 구현
  - 전체 문서 높이 계산 (페이지별 `getPageInfo()` → 높이 합산)
  - 뷰포트에 보이는 페이지만 Canvas 생성/렌더링
  - 스크롤 시 벗어난 페이지 Canvas 해제, 새 페이지 렌더링
- `src/webview/zoom-control.ts` — 줌 컨트롤
  - 줌 인/아웃 버튼 (또는 Ctrl+마우스 휠)
  - 줌 레벨에 따른 Canvas 스케일 조정
- 페이지 네비게이션
  - 상태 표시줄에 현재 페이지 / 전체 페이지 표시
  - 페이지 번호 클릭 시 해당 페이지로 이동
- 스타일시트 (`src/webview/viewer.css`)
  - VSCode 테마 색상 변수 활용 (`--vscode-editor-background` 등)
  - 페이지 그림자, 간격, 스크롤 영역 레이아웃

**산출물**: 전체 문서 스크롤 열람 + 줌 + 페이지 네비게이션 동작

### 5단계: 설계 문서 작성

**목표**: 구현 결과를 바탕으로 아키텍처 설계 문서 정리

**작업 내용**:
- `mydocs/tech/vscode_extension_design.md` — 기술 설계 문서
  - 아키텍처 다이어그램 (Extension Host ↔ Webview ↔ WASM)
  - 메시지 프로토콜 명세
  - WASM 로딩 전략 및 CSP 설정
  - 디렉토리 구조 및 빌드 파이프라인
  - rhwp-studio와의 차이점 및 공유 범위
  - 향후 확장 로드맵 (편집, 검색, 아웃라인)
- 최종 결과 보고서 작성

**산출물**: 설계 문서 완성, 최종 보고서

## 기술 결정 사항

| 항목 | 결정 | 이유 |
|------|------|------|
| Editor Provider | `CustomReadonlyEditorProvider` | v1은 읽기 전용. 향후 `CustomEditorProvider`로 전환 |
| WASM 전달 방식 | postMessage (Uint8Array) | asWebviewUri는 WASM MIME 타입 미지원. VSCode 1.57+ 지원 |
| 번들러 | webpack | VSCode 확장 공식 권장. Extension Host + Webview 이중 번들 |
| 렌더링 | Canvas 2D | rhwp WASM의 `renderPageToCanvas()` 직접 재활용 |
| CSP | `wasm-unsafe-eval` | Webview에서 WebAssembly.instantiate() 허용에 필수 |

## 승인 요청

위 구현 계획서(5단계)를 검토 후 승인 부탁드립니다. 승인 후 1단계부터 진행하겠습니다.
