# Task M100 #2023 — 프론트 공개 계약·리팩터링 금지 목록 인벤토리

- 이슈: #2023 (umbrella: #2022)
- 브랜치: `local/task2023`
- 기준 커밋: `upstream/devel` `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`
- 작성일: 2026-07-07
- 단계: 1/4 — 공개 계약·금지 목록 인벤토리

## 1. 목적

이 문서는 프론트 웹 리팩터링 계획 수립 전에 보존해야 할 공개 계약과 금지 목록을 먼저
정리한다. 코드 변경 계획이 아니라, 후속 SOLID·복잡도 진단과 실행 하위 이슈 분리의
guardrail 이다.

핵심 결론은 `@rhwp/editor` 하나가 아니라 다음 전체 표면을 함께 보호해야 한다는 점이다.

- `rhwp-studio` 웹/PWA 앱
- `/web` 레거시 폴더와 `web/fonts` 자산 저장소
- Chrome/Firefox/Safari 브라우저 확장
- VS Code custom readonly editor webview
- npm 문서와 `@rhwp/editor` public embed API
- 폰트 라이선스·offline/self-hosted 배포 계약

## 2. 계약 지도

| 영역 | 현재 계약 | 근거 | 리팩터링 guardrail |
|------|-----------|------|--------------------|
| `@rhwp/editor` | iframe 기반 embed wrapper. `createEditor`, `loadFile`, `pageCount`, `getPageSvg`, `exportHwp`, `exportHwpx`, `exportHwpVerify`, `destroy` API 제공 | `npm/editor/package.json`, `npm/editor/index.js`, `npm/editor/README.md` | iframe/postMessage public contract 변경 금지. API 이름·반환 형태 변경은 별도 호환성 이슈 필요 |
| `@rhwp/editor` 의존성 | package 파일은 `index.js`, `index.d.ts`만 포함하며 런타임 dependency 없음 | `npm/editor/package.json` | 런타임 dependency 추가 금지. React/Vue/Svelte 등 UI framework 도입은 계획 리뷰에서 명시적으로 금지 여부 확정 |
| `@rhwp/core` 문서 계약 | WASM 초기화, `HwpDocument`, `measureTextWidth`, `renderPageSvg`, edit API, `*Ex(optionsJson)` 권장 | `npm/README.md` | WASM 파일명·초기화·필수 global callback·README 예제 변경은 npm API 변경으로 취급 |
| `rhwp-studio` 앱 | Vite 앱, PWA, `@wasm` alias, `fonts/*.woff2` 상대 경로 font loader | `rhwp-studio/vite.config.ts`, `rhwp-studio/src/core/font-loader.ts` | public asset 경로와 font loader 경로 변경은 확장/VS Code/npm 문서 영향까지 함께 검토 |
| `rhwp-studio` 폰트 | `rhwp-studio/public/fonts`는 `../../web/fonts` symlink | `ls -l rhwp-studio/public/fonts` | `/web` 삭제 전 font canonical 위치 이전이 선행되어야 함 |
| `/web` 폴더 | legacy JS/HTML/CSS와 현재 실사용 `web/fonts`가 같은 루트에 혼재 | `web/editor.js`, `web/editor.html`, `web/fonts/FONTS.md`, `rg web/fonts` | legacy 삭제와 font 이전을 같은 PR에서 섞지 않음. 먼저 실사용 자산 경계를 확정 |
| Chrome/Firefox 확장 빌드 | `rhwp-studio`를 확장용으로 빌드하고 WASM/font/public 자산을 `dist`로 개별 복사 | `rhwp-chrome/build.mjs`, `rhwp-firefox/build.mjs` | 새 public 자산은 Chrome/Firefox 양쪽 build copy 갱신 필요. `publicDir:false` 함정 재발 금지 |
| Safari 확장 빌드 | Chrome dist를 기반으로 Safari 전용 background/content/manifest를 교체 | `rhwp-safari/build.sh`, `rhwp-safari/src/manifest.json` | Chrome 확장 빌드 산출물 변경은 Safari에도 전파됨. Safari ES module 제약 별도 확인 |
| 확장 보안 | MV3 CSP, sender 검증, URL 검증, file signature, `web_accessible_resources` 축소 필요 | `mydocs/manual/browser_extension_dev_guide.md`, `mydocs/report/archives/browser_extension_security_audit.md`, `rhwp-shared/security/*` | CSP 완화, inline script, 검증 우회, `web_accessible_resources` 확대 금지 |
| VS Code extension | CustomReadonlyEditor webview, `postMessage`, WASM media, local fonts, CDN font 허용 CSP | `rhwp-vscode/src/hwp-editor-provider.ts`, `rhwp-vscode/webpack.config.js` | webview CSP/nonce/localResourceRoots/WASM/font 경로 변경은 VS Code smoke 필요 |
| 폰트 라이선스 | `web/fonts`에는 오픈 라이선스 폰트만 포함. 한컴/MS 저작권 폰트는 Git 미포함 | `web/fonts/FONTS.md`, `THIRD_PARTY_LICENSES.md`, `mydocs/tech/font_fallback_strategy.md` | 저작권 폰트 번들 금지. 새 폰트는 라이선스 문서와 함께 추가 |

## 3. `/web` 레거시와 `web/fonts` 분리 원칙

`/web` 폴더는 두 성격이 섞여 있다.

1. 레거시 웹 앱:
   - `web/index.html`
   - `web/app.js`
   - `web/editor.html`
   - `web/editor.js`
   - `web/text_selection.js`
   - `web/font_substitution.js`
   - `web/char_shape_dialog.js`
   - `web/format_toolbar.js`
   - `web/style.css`
   - `web/editor.css`
   - `web/https_server.py`, `web/certs/*`, `web/clipboard_test.html`

2. 현재 배포 경로가 의존하는 font asset root:
   - `web/fonts/*.woff2`
   - `web/fonts/FONTS.md`
   - `rhwp-studio/public/fonts -> ../../web/fonts`
   - Chrome/Firefox 확장 build script의 `ROOT/web/fonts` 복사
   - VS Code webpack의 `../web/fonts` 복사
   - npm editor README의 self-hosted `web/fonts/` 안내
   - third-party license 문서의 `web/fonts/FONTS.md` 참조

따라서 `/web` 리팩터링은 다음 순서를 지켜야 한다.

1. `web/fonts`의 새 canonical 위치 결정.
2. `rhwp-studio/public/fonts` symlink 또는 실제 asset 위치 갱신.
3. Chrome/Firefox/Safari/VS Code 빌드 경로 갱신.
4. `npm/editor/README.md`, `THIRD_PARTY_LICENSES.md`, `font_fallback_strategy.md`,
   `FONTS.md` 후속 경로 갱신.
5. 웹앱/확장/VS Code smoke 후 legacy `/web` JS/HTML/CSS 삭제 여부 검토.

금지:

- `web/fonts` 이전 없이 `/web` 전체 삭제 금지.
- font path 변경과 UI/engine 리팩터링을 같은 실행 PR에 혼합 금지.
- font loader의 `fonts/*.woff2` 경로를 바꾸면서 확장/VS Code build copy를 누락하는 변경 금지.
- 저작권 보호 대상 한컴/MS 폰트 파일을 Git 또는 배포 번들에 추가 금지.

## 4. `@rhwp/editor` public embed 계약

현재 `@rhwp/editor`는 다음 특성을 가진다.

- package metadata에 runtime `dependencies`가 없다.
- 배포 파일은 `index.js`, `index.d.ts`로 제한된다.
- default studio URL은 `https://edwardkim.github.io/rhwp/`다.
- host page에는 iframe을 삽입하고 `clipboard-read; clipboard-write` allow를 부여한다.
- iframe 내부 `rhwp-studio`와 `postMessage`로 `rhwp-request`/`rhwp-response`를 교환한다.
- public API는 README와 `index.js`가 함께 정의한다.

금지:

- `createEditor`의 iframe embed 모델을 일반 DOM mount 모델로 바꾸는 변경.
- React/Vue/Svelte 등 runtime UI framework를 `@rhwp/editor` dependency로 추가.
- `postMessage` message type, request id, method/params/result 기본 구조를 무계획으로 변경.
- `studioUrl`, `width`, `height` 옵션의 의미 변경.
- `exportHwp`/`exportHwpx`의 `Uint8Array` 반환 계약 변경.

검토 필요:

- `postMessage(..., '*')`는 기존 계약이다. 보안 강화를 검토할 수 있으나, origin 제한은
  self-hosted 사용자를 깨뜨릴 수 있으므로 별도 호환성 계획이 필요하다.
- timeout, error propagation, ready probe 개선은 API 호환 범위에서 별도 실행 이슈로 다룬다.

## 5. `@rhwp/core` npm 문서 계약

로컬 `npm/` 폴더에는 `@rhwp/core` package metadata가 없고 README 중심으로 계약이 남아 있다.
문서가 안내하는 사용 표면은 다음이다.

- `import init, { HwpDocument } from '@rhwp/core'`
- `await init({ module_or_path: '/rhwp_bg.wasm' })`
- WASM 초기화 전에 `globalThis.measureTextWidth` 등록
- `new HwpDocument(new Uint8Array(buffer))`
- `renderPageSvg`, `pageCount`, edit API
- 인자가 많은 edit API는 `*Ex(optionsJson)` 권장

금지:

- `measureTextWidth` 필수 등록 시점 변경을 문서·마이그레이션 없이 수행 금지.
- `rhwp_bg.wasm` 배치/로드 예제를 깨는 npm packaging 변경 금지.
- `*Ex(optionsJson)` 권장 방향과 반대로 positional API만 확장하는 변경 금지.
- README 예제와 실제 타입 정의가 어긋나는 상태로 배포 금지.

## 6. `rhwp-studio` 앱·자산 계약

`rhwp-studio`는 프론트 리팩터링의 주 대상이지만, 다음 계약은 계획 없이 깨면 안 된다.

- Vite entry와 PWA manifest/scope/start URL.
- `@wasm` alias가 `../pkg`를 가리키는 구조.
- `font-loader.ts`가 생성하는 `@font-face`와 `fonts/*.woff2` 상대 경로.
- `disableExternalWebFonts` 옵션.
- OS font detection이 `@font-face` 등록 전에 수행되어야 하는 순서.
- Workbox가 JS/CSS/HTML/image/font를 precache하고 WASM은 runtime cache로 다루는 정책.

금지:

- `fonts/*.woff2` 경로 변경을 단독으로 수행 금지.
- 폰트 fallback을 바꾸면서 `dump-pages` 또는 render-diff 영향 검증 없이 병합 금지.
- CDN font 사용 정책을 확장/VS Code/offline 옵션과 분리해서 변경 금지.
- `rhwp-studio/package.json` dependency 변경을 단순 리팩터링 PR에 혼합 금지.

주의:

- 과거 `rhwp-studio-code-review.md`는 runtime dependency 0이라고 평가했지만 현재
  `rhwp-studio/package.json`에는 `canvaskit-wasm`, `pixelmatch`, `pngjs` runtime dependencies가 있다.
  따라서 프론트 전체에 "dependency 0"을 일반화하면 안 된다. 무의존 계약은 `@rhwp/editor`
  package에 한정해서 다룬다.

## 7. 브라우저 확장 계약

확장 쪽 리팩터링은 구조 공통화보다 security baseline 유지가 우선이다.

### 7.1 공통 보안 guardrail

금지:

- inline script 추가.
- 일반 `eval` 허용.
- CSP 완화.
- `web_accessible_resources` 확대.
- sender 검증 제거 또는 우회.
- URL 검증을 caller별로 흩뜨려 우회 경로를 만드는 변경.
- fetch 시 credential 전송.
- HWP/HWPX magic number와 size limit 검증 약화.
- DOM API 대신 사용자 입력을 포함한 `innerHTML` 조립.
- drag & drop 로컬 파일을 명시적 opt-in 없이 즉시 로드.

유지:

- 검증 로직은 가능하면 `rhwp-shared/security/*` 또는 공통 helper에 둔다.
- Chrome/Firefox/Safari 차이는 manifest/background lifecycle 차이로 제한하고 보안 정책 의미는
  같게 유지한다.

### 7.2 Chrome/Firefox 빌드 guardrail

현재 Chrome/Firefox 확장용 Vite config는 `publicDir:false`다. 따라서 `rhwp-studio/public`
자산은 자동 복사되지 않는다.

금지:

- `rhwp-studio/public`에 새 viewer 필수 자산을 추가하고 `rhwp-chrome/build.mjs`와
  `rhwp-firefox/build.mjs` 복사를 누락.
- Chrome build만 고치고 Firefox build를 누락.
- viewer 정상 여부를 웹앱에서만 확인하고 확장 unpacked smoke를 생략.

### 7.3 Safari guardrail

Safari는 Chrome dist를 복사한 뒤 Safari 전용 파일을 덮어쓴다. 따라서 Chrome dist의 구조 변경은
Safari에도 영향을 준다.

금지:

- Safari의 ES module 미지원, `storage.local` 우선, downloads API 부재를 무시한 공통화.
- Chrome 전용 파일(`sw`, `dev-tools-inject.js`) 제거 규칙을 깨는 build 변경.
- Safari manifest의 CSP와 `web_accessible_resources`를 Chrome 기준으로 단순 복사.

## 8. VS Code extension 계약

VS Code extension은 브라우저 앱/확장과 다른 webview 보안 모델을 가진다.

현재 계약:

- `CustomReadonlyEditorProvider`로 HWP/HWPX를 연다.
- Extension host와 webview가 `postMessage`로 통신한다.
- webview는 `dist/webview/viewer.js`와 `dist/media/rhwp_bg.wasm`을 사용한다.
- webpack copy plugin이 `web/fonts`에서 필수 오픈소스 폰트만 `dist/media/fonts`로 복사한다.
- CSP는 nonce 기반 script/style과 `font-src ${cspSource} https://cdn.jsdelivr.net`를 사용한다.

금지:

- webview CSP에서 nonce 제거.
- `localResourceRoots`를 과도하게 확장.
- WASM/font asset 경로 변경 후 VS Code packaging 확인 생략.
- CDN font 허용 여부를 `rhwp-studio`와 동기화하지 않고 단독 변경.
- webview message type 변경을 extension host와 viewer 양쪽에 반영하지 않는 변경.

## 9. 런타임 UI framework 도입 금지 후보

React/Vue/Svelte 등 runtime UI framework 도입은 계획 리뷰에서 명시적으로 금지 항목으로 확정하는
편이 적절하다. 이유는 `@rhwp/editor` 하나 때문이 아니라 다음 전체 계약 때문이다.

- `@rhwp/editor`는 dependency 없는 iframe wrapper로 배포된다.
- 브라우저 확장은 CSP와 packaging 제약이 강하다.
- VS Code webview는 nonce/CSP/localResourceRoots 제약이 있다.
- `rhwp-studio`는 이미 Vite/TypeScript/순수 DOM 기반으로 구성되어 있고, framework 도입은
  단순 리팩터링을 넘어 UI runtime migration이 된다.

권고 문구:

> 이번 프론트 리팩터링 범위에서는 React/Vue/Svelte 등 런타임 UI framework 도입을 금지한다.
> UI framework 도입이 필요하다고 판단되면 별도 RFC/umbrella 이슈에서 배포 크기, CSP,
> `@rhwp/editor`, 확장, VS Code 영향을 먼저 검토한다.

## 10. 금지 목록 요약

### 절대 금지

- 리팩터링 계획 이슈에서 코드 동작 변경.
- `/web` 전체 삭제를 `web/fonts` canonical 이전 없이 수행.
- 저작권 보호 대상 한컴/MS 폰트 파일 번들.
- 확장 inline script 추가.
- 확장 CSP 완화.
- sender/URL/file signature/size 검증 약화.
- `web_accessible_resources` 확대.
- `@rhwp/editor` runtime dependency 추가.
- `@rhwp/editor` iframe/postMessage public contract 변경.
- 폰트 fallback 변경을 render/layout 검증 없이 수행.

### 계획 리뷰 후 결정

- React/Vue/Svelte 등 runtime UI framework 도입 금지를 공식 금지 목록에 넣을지.
- `web/fonts`의 새 canonical 위치.
- `/web` legacy JS/HTML/CSS 삭제 범위.
- VS Code webview의 CDN font 허용 정책을 유지할지.
- extension `web_accessible_resources` 축소를 프론트 리팩터링 실행 범위에 포함할지.

### 실행 이슈별 분리 필요

- `/web` legacy 삭제.
- font canonical 위치 이전.
- `rhwp-studio` 대형 모듈 분해.
- 확장 공통화.
- VS Code webview font/CSP 정리.
- npm 문서/API 계약 갱신.

## 11. 후속 단계 입력

2단계 현황 재진단에서는 이 문서의 guardrail을 기준으로 다음을 실측한다.

- `rhwp-studio/src` 파일 LOC, 함수 LOC, CC>25 함수.
- `InputHandler`, dialog, `diff-engine`, `wasm-bridge`, `core/types.ts`의 SOLID 위반 지점.
- `web/fonts` 외 `/web` legacy 파일의 실사용 여부.
- Chrome/Firefox/Safari content/background/build 중복.
- `rhwp-shared`로 이미 공통화된 보안 helper와 아직 browser별로 남은 검증 로직.
- VS Code와 npm package 문서 계약의 실제 코드 추적 가능성.
