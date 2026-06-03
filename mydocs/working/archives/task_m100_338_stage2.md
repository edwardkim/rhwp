# Task M100 #338 Stage 2 완료보고서: manifest 및 소스 보안 워닝 수정

- **타스크**: [#338](https://github.com/edwardkim/rhwp/issues/338)
- **브랜치**: `local/task338`
- **작성일**: 2026-04-26
- **Stage**: 2 — manifest 및 소스 수정
- **선행**: `mydocs/working/task_m100_338_stage1.md`

## 1. 변경 요약

Stage 1에서 확인한 수정 대상 중 우선순위가 높은 항목을 수정했다.

| 파일 | 변경 |
|---|---|
| `rhwp-firefox/manifest.json` | `strict_min_version`을 `112.0`에서 `142.0`으로 상향 |
| `rhwp-firefox/build.mjs` | 빌드 시작 시 기존 `dist/`를 삭제해 stale bundle/test artifact 제거 |
| `rhwp-studio/src/command/commands/file.ts` | 인쇄 팝업 생성의 `document.write(...)` 제거, DOM API 기반 문서 구성으로 전환 |
| `rhwp-studio/src/command/extension-api.ts` | 외부 커맨드 메뉴 항목 생성의 `innerHTML` 제거, `textContent` 기반 DOM API로 전환 |
| `rhwp-studio/src/view/canvas-view.ts` | 컨테이너 초기화의 `innerHTML = ''`를 `replaceChildren()`으로 전환 |

## 2. 상세 변경

### 2.1 Firefox manifest 최소 버전 상향

`browser_specific_settings.gecko.data_collection_permissions`를 유지하면서 AMO의 Firefox/Android 지원 버전 워닝을 닫기 위해 다음처럼 변경했다.

```json
"strict_min_version": "142.0"
```

Stage 3 빌드 후 `rhwp-firefox/dist/manifest.json`에도 동일 값이 반영되어야 한다.

### 2.2 Firefox dist stale artifact 제거

현재 `dist/assets`에는 과거 빌드의 `viewer-*.js`가 8개 누적되어 있었고, `dist/shared/security/url-validator.test.js`도 남아 있었다.

`build.mjs`에 빌드 시작 전 정리 단계를 추가했다.

```js
if (existsSync(DIST)) {
  rmSync(DIST, { recursive: true, force: true });
  console.log(`  CLEAN: ${DIST}`);
}
```

기대 효과:

- 오래된 `viewer-*.js`가 AMO 스캔 대상에 남지 않음
- 과거 복사 흔적인 test 파일 제거
- 실제 제출 패키지가 현재 소스 기준 산출물만 포함

### 2.3 인쇄 팝업 `document.write` 제거

기존 `file:print`는 `printWin.document.write(...)`로 전체 HTML 문자열을 삽입했다.

변경 후:

- `setupPrintDocument()`에서 `doc.head`/`doc.body`를 DOM API로 구성
- 문서명과 페이지 수는 `textContent`로 삽입
- 스타일은 `style.textContent`로 삽입
- SVG 페이지는 `DOMParser().parseFromString(svg, 'image/svg+xml')` 후 `importNode()`로 삽입
- 버튼 이벤트는 기존처럼 DOM API로 바인딩

효과:

- AMO `document.write` 워닝 제거 대상
- 파일명 문자열이 HTML 문맥에 직접 삽입되지 않음
- SVG 페이지 문자열은 HTML 파싱이 아니라 SVG XML 파싱 경로로 제한

### 2.4 외부 커맨드 메뉴 `innerHTML` 제거

`StudioExtensionAPI.addMenuItem()`은 외부에서 등록한 command label/shortcut을 `innerHTML`로 삽입했다.

변경 후:

- `.md-label`, `.md-shortcut` span을 직접 생성
- `def.label`, `def.shortcutLabel`은 `textContent`로 삽입

외부 확장 API 경로이므로 이번 Stage 2에서 우선 수정했다.

### 2.5 CanvasView 초기화 `innerHTML` 제거

`scrollContent.innerHTML = ''`는 단순 비우기 용도였으므로 `replaceChildren()`으로 치환했다.

## 3. 정적 검색 결과

수정 후 다음 검색을 수행했다.

```bash
rg -n "document\\.write" rhwp-studio/src rhwp-firefox --glob '!node_modules' --glob '!dist'
rg -n "innerHTML" rhwp-studio/src/command/commands/file.ts rhwp-studio/src/view/canvas-view.ts rhwp-studio/src/command/extension-api.ts
rg -n '"strict_min_version"|"data_collection_permissions"' rhwp-firefox/manifest.json
```

결과:

- `document.write`: 소스 기준 검색 결과 없음
- 수정한 세 파일의 `innerHTML`: 검색 결과 없음
- `rhwp-firefox/manifest.json`: `strict_min_version: "142.0"` 확인

## 4. 타입 검증

`rhwp-studio` 의존성이 없어 `npm install`을 실행한 뒤 타입 검증을 수행했다.

```bash
cd rhwp-studio
./node_modules/.bin/tsc --noEmit
```

결과: 실패.

실패 원인:

```text
src/core/wasm-bridge.ts(133,21): error TS2551: Property 'exportHwpx' does not exist on type 'HwpDocument'. Did you mean 'exportHwp'?
src/core/wasm-bridge.ts(137,22): error TS2339: Property 'getSourceFormat' does not exist on type 'HwpDocument'.
```

판정:

- 이번 Stage 2 변경 파일이 아닌 `src/core/wasm-bridge.ts`와 현재 `pkg/rhwp.d.ts` 사이의 기존 타입 정의 불일치
- Stage 3의 Firefox 확장 빌드에서도 같은 문제가 재현될 수 있으므로 별도 확인 필요
- 이번 Stage 2 변경 자체는 정적 검색 기준 목표를 충족

## 5. 남은 항목

Stage 3에서 다음을 확인한다.

- `npm run build` 실행 시 `dist/` 정리 후 산출물이 정상 재생성되는지
- `dist/manifest.json`의 `strict_min_version`이 `142.0`인지
- `dist/assets`에 최신 `viewer-*.js`만 남는지
- `dist/shared/security/url-validator.test.js` 같은 stale test 파일이 제거되는지
- 재빌드 후 `document.write`, `Function`, `innerHTML` 워닝 잔여가 어느 파일에 남는지
- TypeScript 실패가 `pkg/rhwp.d.ts` 갱신으로 해결되는지 또는 별도 조치가 필요한지

## 6. 변경 파일

```text
rhwp-firefox/build.mjs
rhwp-firefox/manifest.json
rhwp-studio/src/command/commands/file.ts
rhwp-studio/src/command/extension-api.ts
rhwp-studio/src/view/canvas-view.ts
```

## 7. 승인 요청

Stage 2 완료 기준을 충족했다.

Stage 3 착수를 요청한다. Stage 3에서는 Firefox 확장 빌드와 dist 갱신을 수행한다.
