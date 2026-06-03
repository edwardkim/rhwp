# Task M100 #338 Stage 1 완료보고서: AMO 워닝 후보 재현 및 분류

- **타스크**: [#338](https://github.com/edwardkim/rhwp/issues/338)
- **브랜치**: `local/task338`
- **작성일**: 2026-04-26
- **Stage**: 1 — 워닝 후보 재현 및 원본 매핑

## 1. 수행 내용

다음 항목을 기준으로 `rhwp-firefox` 소스와 현재 `dist` 산출물을 조사했다.

- `strict_min_version`
- `data_collection_permissions`
- `new Function`
- `Function(`
- `innerHTML`
- `document.write`
- 테스트 파일의 dist 포함 여부
- 오래된 번들 산출물 잔존 여부

## 2. manifest 워닝 재현

현재 소스와 dist 모두 동일하게 AMO 워닝 조건을 가진다.

| 파일 | 현재 값 | 판정 |
|---|---:|---|
| `rhwp-firefox/manifest.json` | `strict_min_version: "112.0"` + `data_collection_permissions` | 수정 필요 |
| `rhwp-firefox/dist/manifest.json` | `strict_min_version: "112.0"` + `data_collection_permissions` | 빌드 후 갱신 필요 |

결론: Stage 2에서 `rhwp-firefox/manifest.json`의 `strict_min_version`을 `142.0`으로 상향한다. Stage 3 빌드로 `dist/manifest.json`을 갱신한다.

## 3. 보안 워닝 후보 분류

### 3.1 `Function` / `new Function`

현재 로컬 산출물 기준 검색 결과:

- `rhwp-firefox/dist/wasm/rhwp.js`: `Function` / `new Function` 검색 결과 없음
- `pkg/rhwp.js`: `Function` / `new Function` 검색 결과 없음
- `rhwp-firefox/dist/assets/*.js`: `Function(` 검색 결과 없음

이슈 본문에는 `wasm/rhwp.js:5825:25`의 `Function` 생성자 워닝이 기록되어 있으나, 현재 checkout의 `pkg/rhwp.js` 및 `dist/wasm/rhwp.js`에서는 재현되지 않았다. Stage 3의 재빌드 후 다시 확인하고, 재현되면 wasm-bindgen 생성 코드로 reviewer note에 분류한다.

### 3.2 `document.write`

현재 dist의 `document.write`는 `viewer-*.js` 번들에서 확인된다. 원본은 다음 코드다.

- `rhwp-studio/src/command/commands/file.ts`: 인쇄 전용 팝업 창 생성 후 HTML 전체를 `printWin.document.write(...)`로 작성

판정:

- 사용자 입력 경로가 일부 포함된다: `wasm.fileName`, `svgPages`
- 팝업 인쇄 기능의 독립 문서 생성 목적이지만 AMO 정적 분석에서는 워닝 대상
- Stage 2에서 DOM API 기반 생성 또는 안전한 escaping/구조화 방식으로 교체 대상

### 3.3 `innerHTML`

현재 dist의 `innerHTML`은 대부분 `rhwp-studio` 번들에서 발생한다. 원본 후보는 다음처럼 분류했다.

| 원본 | 용도 | 판정 |
|---|---|---|
| `rhwp-studio/src/view/canvas-view.ts` | 컨테이너 비우기 (`''`) | 안전 API 치환 가능 (`replaceChildren`) |
| `rhwp-studio/src/engine/table-object-renderer.ts` | 내부 계산 좌표 기반 SVG overlay 생성 | 사용자 입력 직접 경로 아님, DOM/SVG API 치환 가능 |
| `rhwp-studio/src/engine/input-handler.ts` | 다각형/텍스트박스 배치 프리뷰 overlay 생성 | 마우스 좌표 기반, DOM/SVG API 치환 가능 |
| `rhwp-studio/src/command/extension-api.ts` | 메뉴 항목 label/shortcut 삽입 | 확장 API 정의값 경로. `textContent` 치환 권장 |
| `rhwp-studio/src/ui/*` 다수 | 다이얼로그 옵션/프리뷰/목록 렌더링 | Stage 2에서 우선순위 선별 필요 |
| `rhwp-firefox/content-script.js` | 주석 문자열의 `innerHTML` 언급만 검색됨 | 수정 불필요 |
| `rhwp-firefox/test/03-dynamic-content.html` | 테스트 페이지 | dist 미포함, 수정 불필요 |

Stage 2에서는 AMO가 실제로 지적한 `assets/viewer-*.js` 경로를 줄이는 것이 목표다. 우선순위는 `document.write`, 사용자 입력/외부 데이터 가능성이 있는 `innerHTML`, 단순 비우기 치환 순서로 잡는다.

## 4. dist 산출물 상태

현재 `rhwp-firefox/dist/assets`에는 `viewer-*.js`가 8개 존재한다.

| 항목 | 결과 |
|---|---:|
| `viewer-*.js` 개수 | 8개 |
| 총 크기 | 약 5.35MB |
| sourcemap | 없음 |

이는 여러 빌드 세대의 산출물이 누적된 상태로 보인다. AMO는 패키지 내 모든 JS 파일을 스캔하므로, 오래된 `viewer-*.js`가 남아 있으면 이미 수정된 워닝도 계속 보고될 수 있다.

Stage 2/3 조치 필요:

- `rhwp-firefox/build.mjs`가 빌드 전 `dist`를 정리하는지 확인 및 보강
- Stage 3 빌드 후 `dist/assets`의 오래된 viewer 번들이 제거되었는지 확인

## 5. 테스트 파일 포함 여부

`rhwp-firefox/test/*.html`은 현재 dist에 포함되지 않는다.

다만 현재 dist에는 다음 테스트 관련 파일이 남아 있다.

- `rhwp-firefox/dist/sw/test-mode.js`
- `rhwp-firefox/dist/shared/security/url-validator.test.js`

`url-validator.test.js`는 `rhwp-shared/security` 쪽 파일이 dist에 과거 복사된 흔적으로 보이며, 현재 `rhwp-firefox/build.mjs`에는 `shared` 복사 단계가 없다. stale artifact 가능성이 높다.

Stage 2/3 조치 필요:

- 빌드 전 dist 정리로 stale test 파일 제거
- `sw/test-mode.js`가 런타임 필수인지 확인. 개발/테스트 전용이면 dist 제외 검토

## 6. 수정 대상과 reviewer note 대상

| 대상 | 분류 | Stage 2 조치 |
|---|---|---|
| `rhwp-firefox/manifest.json` `strict_min_version` | 수정 대상 | `142.0` 상향 |
| `rhwp-firefox/dist/manifest.json` | 빌드 산출물 | Stage 3 빌드로 갱신 |
| `rhwp-studio/src/command/commands/file.ts` `document.write` | 수정 대상 | DOM API 또는 안전 작성 방식으로 교체 |
| `rhwp-studio/src/view/canvas-view.ts` 단순 `innerHTML = ''` | 수정 대상 후보 | `replaceChildren()` 치환 |
| overlay SVG용 `innerHTML` | 수정 대상 후보 | SVG DOM API 치환 또는 안전성 근거 기록 |
| 다이얼로그/UI 템플릿 `innerHTML` | 수정 대상 후보 | 사용자 입력 경로 우선 선별 |
| `wasm/rhwp.js` `Function` | 현재 미재현 | Stage 3 재검색 후 reviewer note 여부 결정 |
| Vite/의존성 코드 | 현재 명확한 별도 후보 없음 | Stage 3 재검색 후 reviewer note 여부 결정 |
| stale dist 파일 | 수정 대상 | 빌드 전 dist 정리 보강 |

## 7. 검증 명령

실행한 주요 명령:

```bash
rg -n '"strict_min_version"|"data_collection_permissions"' rhwp-firefox/manifest.json rhwp-firefox/dist/manifest.json
rg -n "new Function|Function\\(|innerHTML|document\\.write" rhwp-firefox rhwp-studio/src --glob '!node_modules'
rg -n "new Function|Function\\(|innerHTML|document\\.write" rhwp-firefox/dist --glob '!*.wasm'
rg -n "Function" rhwp-firefox/dist/wasm/rhwp.js pkg/rhwp.js rhwp-firefox/dist/assets/*.js
find rhwp-firefox/dist/assets -maxdepth 1 -type f -name 'viewer-*.js' -exec wc -c {} +
find rhwp-firefox/dist -path '*/test/*' -o -name '*test*' -o -name '*.map'
```

## 8. Stage 1 결론

Stage 1 완료 기준을 충족했다.

- manifest 워닝 원인 확인 완료
- 수정 대상과 reviewer note 대상 1차 분리 완료
- dist에 오래된 viewer 번들과 테스트 파일 흔적이 남는 문제 확인
- `Function` 생성자 워닝은 현재 로컬 산출물에서 미재현, Stage 3 재빌드 후 재확인 필요

## 9. 승인 요청

Stage 2 착수를 요청한다.

Stage 2에서는 다음 파일 수정이 예상된다.

- `rhwp-firefox/manifest.json`
- `rhwp-firefox/build.mjs`
- `rhwp-studio/src/command/commands/file.ts`
- `rhwp-studio/src/view/canvas-view.ts`
- 필요 시 일부 overlay/UI 렌더링 파일
