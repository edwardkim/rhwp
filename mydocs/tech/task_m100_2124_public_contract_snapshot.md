# Task M100 #2124 — 프론트 public contract snapshot

- 이슈: #2124
- 상위 umbrella: #2022
- 선행 계획 이슈: #2023
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 작성일: 2026-07-10
- 단계: Phase 0 baseline freeze / Stage 3

## 1. 목적

이 문서는 프론트 웹 리팩터링 전에 외부 소비자와 내부 배포 표면이 의존하는 public contract를
스냅샷으로 고정한다. #2124에서는 계약을 바꾸지 않고, 이후 Phase A/B 리팩터링에서 유지해야 할
경계와 금지선을 명확히 한다.

이 문서는 구현 변경 요청서가 아니다. 아래 항목은 현재 동작을 기록한 baseline이며, 계약 변경은 별도
이슈와 리뷰를 거쳐야 한다.

## 2. 계약 표면 요약

| 표면 | 주요 소스 | 현재 계약 | #2124 guardrail |
|------|-----------|-----------|-----------------|
| `@rhwp/editor` | `npm/editor/package.json`, `index.js`, `index.d.ts`, `README.md` | iframe 기반 `createEditor()`와 postMessage API | runtime dependency 추가 금지, React/Vue/Svelte 등 UI runtime 도입 금지 |
| `rhwp-studio` iframe endpoint | `rhwp-studio/src/main.ts` | `rhwp-request` / `rhwp-response`, legacy `hwpctl-load` | method, envelope, byte array shape 변경 금지 |
| `@rhwp/core` | `npm/README.md`, `src/wasm_api.rs`, fresh `pkg/rhwp.d.ts` | WASM init, `measureTextWidth`, `HwpDocument`, JSON string edit API | Rust-owned API를 프론트 리팩터링에서 임의 변경 금지 |
| VS Code extension | `rhwp-vscode/src/hwp-editor-provider.ts`, `src/webview/viewer.ts`, `webpack.config.js` | custom readonly editor, webview messages, WASM/font media paths | webview message와 asset path 변경 금지 |
| Chrome/Firefox extension | `rhwp-chrome`, `rhwp-firefox` | MV3 extension viewer, content script badge/hover, SW message router | shared message types와 build copy contract 변경 금지 |
| Safari extension | `rhwp-safari/src`, `rhwp-safari/build.sh` | Chrome dist 기반 Safari 전용 manifest/background/content script | Safari 전용 보안/호환 차이 유지 |
| legacy `/web` | `web/*.js`, `web/fonts` | legacy demo/runtime asset와 canonical font 위치 | #2124에서 제거하지 않음. #2125에서 font canonical 이전만 별도 검토 |

## 3. `@rhwp/editor` 계약

### 3.1 Package metadata

| 항목 | 값 |
|------|----|
| package | `@rhwp/editor` |
| version | `0.7.17` |
| module type | `module` |
| main | `index.js` |
| types | `index.d.ts` |
| package files | `index.js`, `index.d.ts` |
| runtime dependencies | 없음 |

`@rhwp/editor`는 다운스트림에서 React 등 특정 UI runtime 없이 사용할 수 있는 iframe wrapper다.
이 package의 무의존 계약은 유지 대상이다. 이 계약과 별개로 #2023 v2 승인에 따라 이번 프론트
리팩터링 전체에는 v1.0까지 React/Vue/Svelte 등 runtime UI framework를 도입하지 않는다. v1.0 이후
기술 선택은 별도 RFC/umbrella에서 재론한다.

### 3.2 `createEditor(container, options?)`

| 입력 | 현재 의미 |
|------|-----------|
| `container` | CSS selector 또는 `HTMLElement` |
| `options.studioUrl` | 기본값 `https://edwardkim.github.io/rhwp/` |
| `options.width` | iframe CSS width, 기본 `100%` |
| `options.height` | iframe CSS height, 기본 `100%` |

동작:

- iframe을 생성해 container에 append한다.
- iframe `allow`는 `clipboard-read; clipboard-write`다.
- iframe `load` 이벤트 이후 `ready` method를 최대 30회 재시도한다.
- `ready`는 500ms 간격으로 확인하며, 초기화 timeout은 wrapper 쪽에서 `Editor initialization timeout`으로
  노출된다.

### 3.3 Public methods

| Method | 요청 method | 입력 | 반환 |
|--------|-------------|------|------|
| `loadFile(data, fileName?)` | `loadFile` | `ArrayBuffer` 또는 `Uint8Array`, 기본 파일명 `document.hwp` | `{ pageCount: number }` |
| `pageCount()` | `pageCount` | 없음 | `number` |
| `getPageSvg(page?)` | `getPageSvg` | 0-based page, 기본 `0` | SVG string |
| `exportHwp()` | `exportHwp` | 없음 | `Uint8Array` |
| `exportHwpx()` | `exportHwpx` | 없음 | `Uint8Array` |
| `exportHwpVerify()` | `exportHwpVerify` | 없음 | `{ bytesLen, pageCountBefore, pageCountAfter, recovered }` |
| `element` | N/A | getter | `HTMLIFrameElement` |
| `destroy()` | N/A | 없음 | iframe 제거, pending request clear |

`exportHwp()`와 `exportHwpx()`는 iframe 응답이 plain array일 때도 `Uint8Array`로 변환한다. 이 shape는
다운스트림 소비자 호환성이 있으므로 임의 변경하지 않는다.

### 3.4 Message envelope

요청:

```ts
{
  type: 'rhwp-request',
  id: number,
  method: string,
  params: object
}
```

응답:

```ts
{
  type: 'rhwp-response',
  id: number,
  result?: unknown,
  error?: string
}
```

현재 wrapper는 `postMessage(..., '*')`를 사용하고, request timeout은 10초다. `'*'`는 유지해야 할
public contract가 아니라 현재 구현과 보안 부채의 snapshot이다. origin pinning을 도입하려면 custom
`studioUrl`, 배포 origin, 호환성 영향을 별도 이슈에서 설계하고 회귀 테스트해야 하므로 #2124에서는
동작을 바꾸지 않는다.

## 4. `rhwp-studio` iframe endpoint 계약

`rhwp-studio/src/main.ts`는 부모 페이지에서 들어오는 `message`를 받아 iframe 제어 API를 제공한다.

### 4.1 Legacy compatibility

| Message | 조건 | 동작 |
|---------|------|------|
| `hwpctl-load` | `msg.type === 'hwpctl-load' && msg.data` | `Uint8Array(msg.data)`를 `loadBytes()`로 로드하고 `{ pageCount }` 응답 |

legacy `hwpctl-load`는 기존 임베딩 호환을 위해 유지한다.

### 4.2 `rhwp-request` methods

| Method | 입력 | 반환 | 비고 |
|--------|------|------|------|
| `ready` | 없음 | `true` | `initPromise` 완료 후 응답 |
| `loadFile` | `params.data`, `params.fileName`, `params.skipUnsavedGuard?` | `{ pageCount }` | unsaved guard가 취소하면 error |
| `pageCount` | 없음 | `number` | `wasm.pageCount` |
| `getPageSvg` | `params.page ?? 0` | SVG string | `wasm.renderPageSvg()` |
| `exportHwp` | 없음 | number array | `Array.from(wasm.exportHwp())` |
| `exportHwpx` | 없음 | number array | `Array.from(wasm.exportHwpx())` |
| `exportHwpVerify` | 없음 | verify object | `JSON.parse(wasm.exportHwpVerify())` |
| unknown | method string | error | `Unknown method: ${method}` |

endpoint는 현재 요청 origin/source를 allowlist로 검증하지 않고, 응답도
`e.source?.postMessage(..., { targetOrigin: '*' })`로 반환한다. 이는 외부 계약이 아니라 추후 보안
설계 대상이다. #2124에서는 baseline만 고정하고 런타임 동작은 변경하지 않는다.

## 5. `@rhwp/core`와 WASM 소비 계약

`@rhwp/core` README와 Rust `#[wasm_bindgen]` export를 public contract의 source of truth로 삼는다.
`pkg/rhwp.d.ts`는 fresh `wasm-pack build`가 생성하는 검증 산출물이지 수동 편집 대상이나 commit 기준의
독립 authority가 아니다.

| 항목 | 계약 |
|------|------|
| WASM init | `await init({ module_or_path: '/rhwp_bg.wasm' })` 또는 번들러별 path |
| text measurement | WASM init 전에 `globalThis.measureTextWidth(font, text)` 등록 필요 |
| document object | `new HwpDocument(new Uint8Array(buffer))` |
| rendering | `renderPageSvg(page)`, `renderPageToCanvas(page, canvas, scale)` 계열 |
| page/document info | `getDocumentInfo()`, `getPageInfo(page)`는 JSON string 반환 |
| edit API | 다수 API가 JSON string 또는 primitive 반환 |
| large option API | `<name>Ex(optionsJson)` 패턴, key는 camelCase, binary data는 별도 인자 |

이 영역은 Rust 리팩터링(#1883)의 public API와도 연결된다. 프론트 리팩터링은 이 계약을 임의로 재정의하지
않고, 필요 시 typed adapter를 프론트 내부에 추가하는 방식으로 제한한다.

초기 기준선 검증(`782059d9`)의 repository Docker build 전 ignored `pkg/rhwp.d.ts`는 당시 Rust source보다 오래되어
`flushDeferredPagination`, `getCursorRectByPathNear`, `getStructure`,
`insertTextInCellDeferredPagination` 네 export가 빠져 있었다. binding test가 이 stale 상태를 정확히
탐지했으며, repository Docker service로 fresh WASM을 생성한 뒤 같은 test가 통과했다. Studio build와
VS Code compile도 fresh declaration 기준으로 통과했다. stale generated output은 계약 변경 근거나
결함 확정 근거로 사용하지 않는다.

최종 기준 `6f1bd284`에서는 pre-build binding도 통과했으며, upstream Rust 변경을 포함한 fresh WASM을
다시 생성해 같은 public surface와 consumer build를 재확인했다.

## 6. VS Code extension 계약

### 6.1 Provider 계약

| 항목 | 값 |
|------|----|
| provider type | `CustomReadonlyEditorProvider` |
| viewType | `rhwp.hwpViewer` |
| webview scripts | `enableScripts: true` |
| localResourceRoots | extension `dist` |
| multiple editor | `supportsMultipleEditorsPerDocument: false` |
| retain context | `retainContextWhenHidden: true` |

### 6.2 Extension host -> webview messages

| Message | Payload | 의미 |
|---------|---------|------|
| `load` | `fileName`, `fileData: Uint8Array` | HWP 파일 로드 |
| `exportSvg` | 없음 | 전체 페이지 SVG 내보내기 요청 |
| `exportDebugOverlay` | 없음 | debug overlay SVG 내보내기 요청 |

### 6.3 Webview -> extension host messages

| Message | Payload | 의미 |
|---------|---------|------|
| `ready` | 없음 | WASM 초기화 완료, 파일 전송 가능 |
| `loaded` | `pageCount` | 파일 로드 완료. 현재 provider의 핵심 제어 흐름은 `ready`와 export 응답에 의존 |
| `exportSvgDone` | `svgs` 또는 `error` | SVG export 결과 |
| `debugOverlaySvgs` | `svgs` 또는 `error` | debug overlay 결과 |

### 6.4 Asset/CSP 계약

| 항목 | 현재 경로/정책 |
|------|----------------|
| viewer bundle | `dist/webview/viewer.js` |
| WASM | `dist/media/rhwp_bg.wasm` |
| fonts | `dist/media/fonts` |
| WASM loader alias | webpack alias `@rhwp-wasm` -> `../pkg` |
| CSP script | nonce, webview cspSource, `unsafe-eval`, `wasm-unsafe-eval` |
| CSP font | webview cspSource, `https://cdn.jsdelivr.net` |

VS Code extension은 `../web/fonts`의 일부 font를 `dist/media/fonts`로 복사한다. #2125에서 font canonical
위치가 바뀌면 webpack copy source도 함께 갱신해야 한다.

## 7. Browser extension 계약

### 7.1 Manifest 공통 표면

| 브라우저 | manifest version | background | content scripts | 주요 permissions |
|----------|------------------|------------|-----------------|------------------|
| Chrome | 3 | module service worker `background.js` | all urls, `document_idle` | `activeTab`, `downloads`, `contextMenus`, `clipboardWrite`, `storage` |
| Firefox | 3 | module background script `background.js` | all urls, `document_idle` | `activeTab`, `downloads`, `contextMenus`, `clipboardWrite`, `storage` |
| Safari | 3 | non-persistent `background.js` | all urls, `document_idle` | `activeTab`, `contextMenus`, `storage` |

Chrome/Firefox는 `web_accessible_resources`에 `wasm/*`, `fonts/*`, `icons/*`, `dev-tools-inject.js`를
노출한다. Safari는 `wasm/*`, `fonts/*`, `icons/*`를 노출한다.

### 7.2 Content script -> background messages

| Message | Chrome/Firefox payload | Safari payload | 의미 |
|---------|------------------------|----------------|------|
| `get-settings` | 없음 | 없음 | badge/hover/auto-open 관련 설정 조회 |
| `open-hwp` | `url`, optional `filename` | `url`, `filename` | viewer tab 또는 overlay 열기 |
| `extract-thumbnail` | `url`, `allowDownloadUrl?` | `url` | HWP/HWPX preview thumbnail 추출 |

### 7.3 Viewer -> background message

| Message | Payload | Response | 의미 |
|---------|---------|----------|------|
| `fetch-file` | `url` | `{ data }` 또는 `{ error }` | 확장 권한으로 문서 bytes fetch |

Chrome/Firefox의 SW 라우터는 `Array.from(new Uint8Array(buffer))` shape로 bytes를 반환한다. Safari는
현재 background에서 `ArrayBuffer`를 직접 반환한다. `rhwp-studio/src/main.ts`의 `loadFromUrlParam()`은
`new Uint8Array(result.data)`로 받아들이므로 두 shape 모두 현재 호환된다. 이 차이는 Stage 4 보안 snapshot에서
브라우저별 차이로 다시 기록한다.

### 7.4 Build copy 계약

| 대상 | build source | 주요 copy |
|------|--------------|-----------|
| Chrome | `rhwp-chrome/build.mjs` | Vite studio build -> `viewer.html`, manifest/background/content script, `sw/`, options, icons, locales, theme-init, studio images, `pkg` WASM, `web/fonts/*.woff2` |
| Firefox | `rhwp-firefox/build.mjs` | Chrome과 같은 구조. Firefox manifest/background/options 사용 |
| Safari | `rhwp-safari/build.sh` | Chrome dist 생성 후 Safari dist로 복사, Safari `src/*`로 manifest/background/content/options 교체, `sw/`와 `dev-tools-inject.js` 제거 |

Chrome/Firefox는 `web/fonts`의 woff2 전체를 extension `dist/fonts`에 복사한다. Safari는 Chrome dist를 기반으로
하므로 같은 font copy 결과에 의존한다.

## 8. 리팩터링 금지선

Phase 0/Phase A에서 유지해야 할 금지선:

- `@rhwp/editor`에 runtime dependency를 추가하지 않는다.
- React/Vue/Svelte 등 runtime UI framework를 이번 프론트 리팩터링 전체에 도입하지 않는다(v1.0까지).
  이후 도입은 별도 RFC/umbrella에서 배포 크기, CSP, embed 무의존성 영향을 재검토한다.
- iframe `rhwp-request` / `rhwp-response` envelope와 public method 이름을 바꾸지 않는다.
- `loadFile`, `exportHwp`, `exportHwpx`의 byte transport shape를 별도 migration 없이 바꾸지 않는다.
- legacy `hwpctl-load`를 삭제하지 않는다.
- `@rhwp/core`의 JSON string / `*Ex(optionsJson)` 계약을 프론트 리팩터링에서 임의 변경하지 않는다.
- VS Code `rhwp.hwpViewer` viewType과 webview message 이름을 바꾸지 않는다.
- extension message type `open-hwp`, `fetch-file`, `extract-thumbnail`, `get-settings`를 별도 migration 없이
  바꾸지 않는다.
- `web/fonts` canonical 이동은 #2125 범위에서만 다룬다. #2124에서는 font path를 변경하지 않는다.

`postMessage('*')`와 origin/source 미검증은 위 금지선에 포함하지 않는다. 다만 보안 정책 변경은
별도 이슈, 호환성 설계, 자동 테스트를 갖춘 뒤 수행한다.

## 9. 이후 단계 연결

| 후속 작업 | 연결 기준 |
|-----------|-----------|
| Stage 4 font inventory | `web/fonts`, `rhwp-studio/public/fonts`, extension/vscode font copy 경로 |
| Stage 4 extension security snapshot | CSP, WAR, sender validation, URL/file validation, byte shape 차이 |
| #2125 assets/fonts canonical 이전 | build copy source와 symlink/source path 전체 갱신 필요 |
| Phase B 구조 리팩터링 | 이 문서의 message/API contract를 변경하지 않는 내부 분리부터 시작 |

## 10. #2186 이후 embed contract delta

#2124 Phase 0 snapshot은 당시 구현 기록으로 보존한다. #2186 이후에도 `createEditor(container,
options?)`, public method 이름과 반환 type, `@rhwp/editor`의 zero runtime dependency 계약은 바뀌지
않는다. 변경된 표면은 iframe transport와 lifecycle에 한정한다.

| 표면 | #2186 이후 계약 |
|------|-----------------|
| bootstrap | `studioUrl`에서 계산한 exact HTTP(S) origin으로 `rhwp-connect`를 보낸다. |
| v1 negotiation | `version: 1`, session ID, `transferable-array-buffer` capability와 단일 `MessagePort`를 사용한다. |
| request/response | v1 envelope는 version/session/id를 검증하며 malformed 또는 다른 session 응답을 무시한다. |
| binary | `loadFile`은 caller bytes를 복사한 `Uint8Array`를 transferable로 보내 원본 buffer를 detach하지 않는다. HWP/HWPX 응답도 transferable bytes로 받는다. |
| timeout | 일반 method는 기본 10초, load/export는 기본 60초다. timeout, send 실패, `destroy()`에서 pending timer와 port/listener를 정리한다. |
| legacy | v1 handshake timeout 전까지만 기존 `rhwp-request`/`rhwp-response` fallback을 허용한다. Studio의 `hwpctl-load`와 number-array 입력/응답 호환도 유지한다. |

`loadFile(): Promise<{ pageCount: number }>`, `pageCount(): Promise<number>`,
`getPageSvg(): Promise<string>`, `exportHwp()`/`exportHwpx(): Promise<Uint8Array>`,
`exportHwpVerify(): Promise<HwpVerifyResult>` 반환 계약은 그대로다. exact origin binding은 host 인증이나
allowlist/JWT를 뜻하지 않으며, 그 정책은 #2186 범위 밖이다.
