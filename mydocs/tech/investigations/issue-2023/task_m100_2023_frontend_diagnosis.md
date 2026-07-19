---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-2023/README.md
last_verified: 2026-07-16
---

# Task M100 #2023 2단계 — 프론트 현황 재진단 (SOLID + 복잡도)

- 이슈: #2023 (umbrella: #2022)
- 브랜치: `local/task2023`
- 측정일: 2026-07-07
- 기준 커밋: `upstream/devel` `a23b8ae1c85fc80547c40d8e7bbbf37a07283468`
- 단계: 2/4 — 현황 재진단

## 1. 측정 범위와 방법

프론트 웹 리팩터링 계획 수립을 위해 `rhwp-studio`, `/web`, 브라우저 확장, VS Code extension,
npm editor wrapper를 함께 측정했다. Rust 쪽 #1883과 같은 취지로 SOLID와 복잡도 2축을 쓰되,
프론트에는 아직 공식 대시보드가 없으므로 이번 수치는 **계획 수립용 예비 baseline**이다.

측정 기준:

- 파일 목록: `git ls-files`
- LOC: 공백 제외 physical line
- 함수 크기: TypeScript/JavaScript 함수 선언과 brace depth 기반 근사
- CC: `if`, `else if`, `for`, `while`, `case`, `catch`, `?`, `&&`, `||` 토큰 기반 휴리스틱
- 제외: `dist`, `node_modules`, `icons`, `_locales`, `web/fonts`, `web/certs`, generated WASM glue
  (`web/rhwp.js`, `web/rhwp.d.ts`, `web/rhwp_bg.wasm.d.ts`)

주의: 이 CC는 Sonar/clippy cognitive complexity와 같은 공식 지표가 아니다. 같은 스크립트로
반복 측정해 추이를 비교하는 용도로만 사용한다.

## 2. 코드 규모

| 영역 | 추적 파일 | 코드 파일 | 코드 LOC | 스타일 파일 | 스타일 LOC | HTML 파일 | HTML LOC | JSON 파일 | JSON LOC |
|------|----------:|----------:|---------:|------------:|-----------:|----------:|---------:|----------:|---------:|
| `rhwp-studio/src` | 172 | 145 | **53,525** | 26 | 4,578 | 0 | 0 | 0 | 0 |
| `rhwp-studio/e2e` | 63 | 63 | 9,531 | 0 | 0 | 0 | 0 | 0 | 0 |
| `rhwp-studio/tests` | 30 | 30 | 3,167 | 0 | 0 | 0 | 0 | 0 | 0 |
| legacy `/web` | 12 | 6 | 4,550 | 2 | 841 | 3 | 560 | 0 | 0 |
| `rhwp-chrome` | 33 | 17 | 2,512 | 1 | 125 | 8 | 700 | 3 | 1,128 |
| `rhwp-firefox` | 31 | 16 | 2,365 | 1 | 125 | 8 | 700 | 3 | 1,241 |
| `rhwp-safari/src` | 6 | 3 | 1,136 | 1 | 114 | 1 | 336 | 1 | 53 |
| `rhwp-shared` | 13 | 12 | 1,221 | 0 | 0 | 0 | 0 | 1 | 5 |
| `rhwp-vscode` | 5 | 5 | 910 | 0 | 0 | 0 | 0 | 0 | 0 |
| `npm/editor` | 4 | 2 | 241 | 0 | 0 | 0 | 0 | 1 | 33 |

해석:

- `rhwp-studio/src`만 공백 제외 코드 LOC 53,525줄이다. 2026-02
  `rhwp-studio-code-review.md`의 17,524줄/70파일 기준과 직접 1:1 비교는 어렵지만, 규모가
  약 3배 수준으로 커진 것은 분명하다.
- legacy `/web`는 font 제외 후에도 4,550 LOC의 JS가 남아 있다. 다만 현재 생산 경로가
  의존하는 것은 주로 `web/fonts`이며, legacy JS/HTML/CSS는 별도 삭제 후보로 분리해야 한다.
- Chrome/Firefox 확장은 규모가 작아 보이지만, manifest/build/content/background 경로가
  브라우저별로 반복되어 변경 비용이 누적된다.

## 3. 파일 핫스팟

### 3.1 코드 파일 상위

| LOC | 파일 | 판정 |
|----:|------|------|
| **4,104** | `rhwp-studio/src/engine/input-handler.ts` | God coordinator. 입력·선택·편집·렌더 무효화 경계가 집중 |
| **2,907** | `rhwp-studio/src/compare/diff-engine.ts` | diff 책임 집중 |
| **2,466** | `rhwp-studio/src/ui/picture-props-dialog.ts` | UI 생성·상태 populate·검증·적용이 집중 |
| **1,902** | `web/editor.js` | legacy editor entry. 삭제/보존 여부 분리 필요 |
| **1,883** | `rhwp-studio/src/core/wasm-bridge.ts` | WASM 호출·JSON 변환·에러 처리 표면 집중 |
| 1,823 | `rhwp-studio/src/engine/input-handler-keyboard.ts` | keyboard routing 집중 |
| 1,791 | `rhwp-studio/src/engine/input-handler-mouse.ts` | mouse routing 집중 |
| 1,639 | `rhwp-studio/src/engine/cursor.ts` | cursor state machine |
| 1,383 | `rhwp-studio/src/engine/input-handler-table.ts` | table editing adapter |
| 1,361 | `rhwp-studio/src/ui/table-cell-props-dialog.ts` | dialog 생성·상태·적용 집중 |
| 1,288 | `rhwp-studio/src/core/types.ts` | public type surface 과대 |

임계값 집계:

| 영역 | 1,200 LOC 초과 파일 | 2,000 LOC 초과 파일 |
|------|-------------------:|-------------------:|
| `rhwp-studio/src` | **10** | **3** |
| legacy `/web` | 1 | 0 |
| 그 외 영역 | 0 | 0 |

### 3.2 스타일/HTML 상위

| LOC | 파일 | 판정 |
|----:|------|------|
| 861 | `rhwp-studio/src/styles/dialogs.css` | dialog 스타일 집중 |
| 601 | `web/editor.css` | legacy editor 스타일 |
| 336 | `rhwp-safari/src/options.html` | Safari options UI |
| 335 | `web/editor.html` | legacy editor HTML |
| 331 | `rhwp-studio/src/styles/style-bar.css` | style bar 스타일 |
| 316 | `rhwp-studio/src/styles/compare-dialog.css` | compare dialog 스타일 |

CSS는 TypeScript만큼 급하지 않지만, dialog 계열 분해 시 CSS 책임 분리도 함께 따라가야 한다.

## 4. 함수 복잡도 핫스팟

### 4.1 임계값 집계

| 영역 | 함수 수 | CC>15 | CC>25 | CC>100 | 최대 CC | 최대 함수 LOC |
|------|-------:|------:|------:|-------:|-------:|------------:|
| `rhwp-studio/src` | 1,758 | **113** | **49** | **5** | **321** | 989 |
| legacy `/web` | 140 | 12 | 5 | 1 | 122 | 317 |
| `rhwp-chrome` | 80 | 6 | 1 | 0 | 44 | 163 |
| `rhwp-firefox` | 79 | 6 | 1 | 0 | 49 | 174 |
| `rhwp-safari/src` | 43 | 4 | 2 | 0 | 35 | 136 |
| `rhwp-shared` | 30 | 0 | 0 | 0 | 15 | 40 |
| `rhwp-vscode` | 19 | 0 | 0 | 0 | 9 | 178 |
| `npm/editor` | 8 | 0 | 0 | 0 | 6 | 30 |

해석:

- 프론트 복잡도 문제는 압도적으로 `rhwp-studio/src`에 있다.
- `rhwp-shared`는 현재 CC>25가 없어서, 보안/다운로드 공통화의 방향 자체는 효과적이다.
- Chrome/Firefox/Safari 확장은 개별 파일 규모보다 중복·보안 계약 유지 비용이 핵심이다.

### 4.2 CC 상위 함수

| CC | LOC | 함수 | 위치 | 판정 |
|---:|----:|------|------|------|
| **321** | 909 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` | 최우선 해체 후보 |
| **275** | 327 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1921` | dialog validate/apply 혼재 |
| **227** | 989 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:234` | 최우선 해체 후보 |
| 159 | 257 | `populateFromProps` | `rhwp-studio/src/ui/picture-props-dialog.ts:2253` | dialog state mapping 혼재 |
| 132 | 159 | `collectMods` | `rhwp-studio/src/ui/para-shape-dialog.ts:747` | dialog diff/apply 혼재 |
| 122 | 317 | `setupEventListeners` | `web/editor.js:77` | legacy 삭제/보존 판단 필요 |
| 99 | 113 | `hasSupportedColrv1GraphContract` | `rhwp-studio/src/view/glyph-outline-payload-status.ts:128` | 계약 판정 로직 집중 |
| 97 | 328 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1371` | mouse state machine |
| 93 | 124 | `collectMods` | `rhwp-studio/src/ui/char-shape-dialog.ts:923` | dialog diff/apply 혼재 |
| 87 | 354 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:707` | table resize state 집중 |

### 4.3 함수 LOC 상위

| LOC | CC | 함수 | 위치 |
|----:|---:|------|------|
| 989 | 227 | `onClick` | `rhwp-studio/src/engine/input-handler-mouse.ts:234` |
| 909 | 321 | `onKeyDown` | `rhwp-studio/src/engine/input-handler-keyboard.ts:371` |
| 354 | 87 | `finishResizeDrag` | `rhwp-studio/src/engine/input-handler-table.ts:707` |
| 328 | 97 | `onMouseMove` | `rhwp-studio/src/engine/input-handler-mouse.ts:1371` |
| 327 | 275 | `handleOk` | `rhwp-studio/src/ui/picture-props-dialog.ts:1921` |
| 317 | 122 | `setupEventListeners` | `web/editor.js:77` |

결론: 함수 단위 최우선 후보는 `onKeyDown`, `onClick`, `PicturePropsDialog.handleOk`다.
파일 단위 최우선 후보인 `input-handler.ts` 자체는 이미 일부 위임 파일로 분리되어 있지만, 위임
파일의 핵심 함수가 다시 God function이 되어 있다.

## 5. 타입·인터페이스 표면

### 5.1 `any` / `this: any` hotspot

| `this: any` | `any` | 파일 | 판정 |
|-----------:|------:|------|------|
| 18 | 36 | `rhwp-studio/src/engine/input-handler-picture.ts` | 위임 모듈이 coordinator 내부 상태에 강결합 |
| 15 | 46 | `rhwp-studio/src/engine/input-handler-table.ts` | table handler context 불명확 |
| 14 | 51 | `rhwp-studio/src/engine/input-handler-mouse.ts` | mouse handler context 불명확 |
| 13 | 20 | `rhwp-studio/src/engine/input-handler-keyboard.ts` | keyboard handler context 불명확 |
| 12 | 14 | `rhwp-studio/src/engine/input-handler-text.ts` | text handler context 불명확 |
| 7 | 12 | `rhwp-studio/src/engine/input-handler-connector.ts` | connector context 불명확 |
| 0 | 151 | `rhwp-studio/src/core/wasm-bridge.ts` | WASM JSON boundary 타입 약함 |

`this: any`는 InputHandler 분해가 아직 실질적인 의존성 분리를 완료하지 못했음을 보여준다.
실행 이슈에서는 함수 추출보다 먼저 typed context를 설계해야 한다.

### 5.2 import/export hotspot

| imports | exports | 파일 | 판정 |
|--------:|--------:|------|------|
| 0 | **105** | `rhwp-studio/src/core/types.ts` | 프론트 타입 표면 과대 |
| **44** | 0 | `rhwp-studio/src/main.ts` | composition root 비대 |
| 3 | 28 | `rhwp-studio/src/engine/command.ts` | command surface 집중 |
| 29 | 1 | `rhwp-studio/src/engine/input-handler.ts` | 많은 하위 책임 조립 |
| 0 | 25 | `rhwp-studio/src/core/hwp-constants.ts` | 상수 surface |

`core/types.ts`는 인터페이스 분리 원칙(I)의 주요 진단 대상이다. 공개 타입 표면을 무작정 쪼개기보다,
호출자별 실제 사용률을 확인하고 도메인별 type module로 이동하는 계획이 필요하다.

## 6. `/web` 레거시 진단

`web/fonts`와 generated WASM glue를 제외하면 `/web`에는 legacy 웹 앱 파일이 남아 있다.

핵심 관찰:

- production 경로에서 확인된 주요 실사용은 `web/fonts`다.
- `rhwp-studio/public/fonts`는 `../../web/fonts` symlink다.
- Chrome/Firefox build script는 `ROOT/web/fonts`의 모든 `woff2`를 `dist/fonts`로 복사한다.
- VS Code webpack은 `../web/fonts`에서 필수 폰트만 `dist/media/fonts`로 복사한다.
- `rhwp-studio/src/core/font-loader.ts`와 `font-substitution.ts`는 legacy `web/editor.html`,
  `web/font_substitution.js`에서 포팅된 코드라는 주석을 가진다.
- `web/editor.js::setupEventListeners`는 CC 122로 legacy 내부에서도 고복잡도다.

판정:

- `/web` 전체를 단번에 삭제하면 안 된다.
- 먼저 `web/fonts` canonical 이전을 별도 실행 이슈로 분리해야 한다.
- 그 뒤 legacy JS/HTML/CSS 삭제 또는 archives 이동을 검토한다.

## 7. 확장 계층 진단

### 7.1 중복과 공통화 상태

이미 공통화된 파일:

- `rhwp-shared/security/constants.js`
- `rhwp-shared/security/file-signature.js`
- `rhwp-shared/security/filename-sanitizer.js`
- `rhwp-shared/security/security-log.js`
- `rhwp-shared/security/sender-validator.js`
- `rhwp-shared/security/url-validator.js`
- `rhwp-shared/sw/document-url-resolver.js`
- `rhwp-shared/sw/download-interceptor-common.js`
- `rhwp-shared/sw/download-observer-state.js`

Chrome/Firefox는 다음 symlink를 통해 일부 SW 로직을 공유한다.

- `rhwp-chrome/sw/document-url-resolver.js`
- `rhwp-chrome/sw/download-interceptor-common.js`
- `rhwp-chrome/sw/download-observer-state.js`
- `rhwp-firefox/sw/document-url-resolver.js`
- `rhwp-firefox/sw/download-interceptor-common.js`
- `rhwp-firefox/sw/download-observer-state.js`

남은 중복 후보:

- `content-script.js` 계열
- `background.js` 계열
- `build.mjs` 계열
- `sw/message-router.js`, `sw/fetch-security.js`, `sw/viewer-launcher.js`, `sw/context-menus.js`

### 7.2 보안 계약

확장 리팩터링은 구조 공통화보다 보안 계약 유지가 우선이다.

고위험 guardrail:

- inline script 금지
- CSP 완화 금지
- sender 검증 유지
- URL 검증 우회 경로 금지
- magic number/size limit 유지
- `web_accessible_resources` 확대 금지
- `publicDir:false` 환경에서 public asset copy 누락 금지

현재 `rhwp-shared`는 CC>25가 0이라, 공통화 자체는 낮은 복잡도로 유지되고 있다. 다음 실행 이슈는
보안 helper를 더 공유하되 Safari/Firefox lifecycle 차이를 억지로 없애지 않는 방향이어야 한다.

## 8. VS Code와 npm wrapper 진단

### 8.1 VS Code

`rhwp-vscode`는 규모와 CC 측면에서는 현재 큰 리스크가 아니다.

- 코드 LOC: 910
- 함수 수: 19
- CC>25: 0
- 최대 함수 LOC: 178 (`getHtml`)

다만 webview 계약은 민감하다.

- nonce 기반 CSP
- `dist/media/rhwp_bg.wasm`
- `dist/media/fonts`
- `font-src ${cspSource} https://cdn.jsdelivr.net`
- extension host ↔ webview `postMessage`

폰트 canonical 이전이나 CDN policy 변경 시 반드시 VS Code packaging/smoke를 포함해야 한다.

### 8.2 `@rhwp/editor`

`npm/editor`는 규모와 복잡도 문제가 아니다.

- 코드 LOC: 241
- 함수 수: 8
- CC>25: 0
- runtime dependency 없음

따라서 `@rhwp/editor`는 리팩터링의 주 타깃이 아니라 public contract guardrail이다. 이 이슈가
`@rhwp/editor` 중심으로 보이면 범위가 좁아지므로, 본 계획에서는 보조 계약으로만 둔다.

## 9. SOLID 예비 진단

정식 r-code-review 점수가 아니라 #2023 계획 수립용 예비 점수다. 근거 없는 점수화를 피하기 위해
각 원칙마다 파일·지표를 함께 남긴다.

| 원칙 | 예비 점수(/20) | 근거 | 판정 |
|------|---------------:|------|------|
| S — 단일 책임 | **8** | `input-handler.ts` 4,104 LOC, `onKeyDown` CC 321, `onClick` CC 227, dialog `handleOk` CC 275, `/web` legacy와 font asset 혼재 | God coordinator/function이 다수. 최우선 개선 축 |
| O — 개방-폐쇄 | **12** | 새 public asset은 Chrome/Firefox build copy 동시 수정 필요, font path는 studio/extension/VS Code/npm 문서 동시 수정 필요, 브라우저별 build/content 중복 | 확장 포인트는 있으나 변경 시 다중 표면 수정 반복 |
| L — 리스코프 치환 | **14** | renderer/e2e contract는 존재. 다만 studio/VS Code/editor embed message·font 계약이 표면별로 다름 | 즉시 붕괴는 아니나 계약 문서화 부족 |
| I — 인터페이스 분리 | **8** | `core/types.ts` exports 105, `wasm-bridge.ts` `any` 151, InputHandler 위임 모듈의 `this: any` 다수 | 표면이 넓고 호출자별 interface가 불분명 |
| D — 의존성 역전 | **12** | `@rhwp/editor` iframe wrapper는 얇음. 반면 `rhwp-studio` font/DOM/extension build path, VS Code webview font path가 자산 위치에 직접 의존 | 계층 의존은 일부 좋지만 asset/platform detail 의존이 강함 |

예비 합계: **54/100**

해석:

- 이 점수는 프론트 전체 범위의 계획용 영점이며, Rust r-code-review 공식 점수와 직접 비교하지 않는다.
- S/I 축은 정량 신호가 강하게 낮다. `solid_scoring_guide.md` 기준으로 CC>25가 수십 개면 S는
  16을 넘기 어렵다. 프론트는 `rhwp-studio/src`에만 CC>25 함수가 49개다.
- O/D 축은 `/web` font ownership과 extension build copy가 핵심 병목이다.

## 10. 실행 계획 입력

후속 프론트 리팩터링 계획서는 다음 순서를 기본안으로 검토한다.

1. **`/web` font ownership 정리**
   - `web/fonts` canonical 위치 결정
   - studio/extension/VS Code/npm/license 문서 경로 일괄 갱신
   - legacy `/web` 삭제는 그 다음 단계

2. **InputHandler typed context 설계**
   - `this: any` 위임 구조 제거
   - keyboard/mouse/table/text/picture handler별 필요한 최소 context 정의
   - `onKeyDown`, `onClick`, `onMouseMove` 해체

3. **Dialog state/apply 분리**
   - `PicturePropsDialog.handleOk`, `populateFromProps`
   - `CharShapeDialog.collectMods`, `ParaShapeDialog.collectMods`
   - DOM build, state hydrate, validation, command apply를 분리

4. **WasmBridge와 type surface 분리**
   - `wasm-bridge.ts` `any` boundary 정리
   - `core/types.ts` domain별 type module 검토
   - npm/extension/VS Code에 노출되는 계약과 내부 타입 분리

5. **Diff engine 분해**
   - `diff-engine.ts` 2,907 LOC
   - snapshot extraction, identity matching, text diff, visual diff 책임 분리

6. **Extension 공통화**
   - `rhwp-shared`의 낮은 복잡도 유지
   - 보안 검증 로직 우선 공통화
   - Safari lifecycle 차이는 억지 공통화하지 않음

## 11. 결론

프론트 웹 리팩터링은 적합하다. 다만 `@rhwp/editor` 리팩터링이 아니라 다음 3축으로 접근해야 한다.

1. `rhwp-studio` 대형 모듈·고복잡도 함수 해체.
2. `/web` legacy와 `web/fonts` asset ownership 분리.
3. 확장/VS Code/npm 공개 계약을 깨지 않는 guardrail 유지.

계획서의 성공 기준은 "프레임워크 도입"이나 "폴더 이동"이 아니라, 각 실행 이슈가 다음을 동시에
만족하는지로 잡아야 한다.

- public contract 불변
- render/layout 회귀 0
- 확장 CSP/security 회귀 0
- font license/offline/self-hosted 계약 유지
- CC>25 및 1,200 LOC 초과 hotspot 감소
