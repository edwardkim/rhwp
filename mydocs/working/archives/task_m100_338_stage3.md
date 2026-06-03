# Task M100 #338 Stage 3 완료보고서: 잔여 innerHTML 제거 및 Firefox 확장 빌드 검증

- **타스크**: [#338](https://github.com/edwardkim/rhwp/issues/338)
- **브랜치**: `local/task338`
- **작성일**: 2026-04-26
- **Stage**: 3 — 잔여 `innerHTML` 제거 및 Firefox 확장 빌드 검증
- **선행**:
  - `mydocs/working/task_m100_338_stage1.md`
  - `mydocs/working/task_m100_338_stage2.md`

## 1. Stage 3 작업 배경

Stage 2 후 Firefox 확장 빌드는 성공했지만, viewer bundle과 content script에 `innerHTML` 문자열이 일부 남았다.

AMO 리뷰 통과 가능성을 높이기 위해 reviewer note로 설명하는 방식 대신, 제출 패키지에서 `innerHTML` 문자열 자체가 검출되지 않도록 잔여 경로를 DOM API 기반으로 전환했다.

## 2. 추가 변경 요약

| 파일 범위 | 변경 |
|---|---|
| `rhwp-firefox/content-script.js` | 정적 스캔 노이즈를 만드는 주석 문자열 정리 |
| `rhwp-studio/src/engine/table-object-renderer.ts` | 표/그림 선택 overlay SVG 생성 경로를 DOM/SVG API로 전환 |
| `rhwp-studio/src/engine/input-handler.ts` | 도형/선/다각형 배치 프리뷰와 크기 라벨 생성 경로를 DOM/SVG API로 전환 |
| `rhwp-studio/src/engine/caret-renderer.ts`, `selection-renderer.ts` | 주석의 `innerHTML` 문자열 제거 |
| `rhwp-studio/src/ui/dom-utils.ts` | SVG markup import와 option 생성 공용 helper 추가 |
| `rhwp-studio/src/ui/*` | 다이얼로그/드롭다운/프리뷰의 잔여 `innerHTML` 사용을 DOM API, `replaceChildren()`, `textContent`, `option` 요소, SVG XML 파싱/import로 전환 |

## 3. 빌드 결과

다음 명령을 실행했다.

```bash
cd rhwp-firefox
npm run build
```

결과: **성공**.

빌드 로그 주요 내용:

- `CLEAN: rhwp-firefox/dist` 실행 확인
- Vite production build 성공
- `index.html` → `viewer.html` rename 확인
- manifest/background/content-script/options/sw/icon/i18n/wasm/font 복사 완료

Vite 경고:

- `/images/icon_small_ko.svg` 런타임 resolve 경고
- `viewer-*.js` chunk가 500KB 초과

판정: 기존 빌드에서도 발생 가능한 비차단 경고이며, AMO 워닝 수정 범위의 실패는 아니다.

## 4. manifest 반영 확인

다음 명령으로 소스와 dist 모두 확인했다.

```bash
rg -n '"strict_min_version"|"data_collection_permissions"' rhwp-firefox/dist/manifest.json rhwp-firefox/manifest.json
```

결과:

| 파일 | 값 |
|---|---|
| `rhwp-firefox/manifest.json` | `strict_min_version: "142.0"` |
| `rhwp-firefox/dist/manifest.json` | `strict_min_version: "142.0"` |

`data_collection_permissions`는 유지됐다.

## 5. stale artifact 제거 확인

빌드 후 `rhwp-firefox/dist/assets` 상태:

```text
rhwp-firefox/dist/assets/rhwp_bg-DcCngJ7I.wasm
rhwp-firefox/dist/assets/viewer-BK6PlZxS.js
rhwp-firefox/dist/assets/viewer-Di8-R0fz.css
```

Stage 1에서 확인했던 과거 `viewer-*.js` 8개 누적 문제는 해소됐다. 현재 viewer JS는 1개만 남는다.

테스트/소스맵 잔여 확인:

```bash
find rhwp-firefox/dist -path '*/test/*' -o -name '*test*' -o -name '*.map'
```

결과: 없음.

Stage 1에서 확인한 `dist/shared/security/url-validator.test.js` stale artifact도 제거됐다.

## 6. 보안 워닝 패턴 재검색

다음 명령을 수행했다.

```bash
grep -R -l --exclude='*.wasm' 'document\.write' rhwp-firefox/dist
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' rhwp-firefox/dist
grep -R -l --exclude='*.wasm' 'innerHTML' rhwp-firefox/dist
```

결과:

| 패턴 | 결과 |
|---|---|
| `document.write` | 없음 |
| `new Function` / `Function(` | 없음 |
| `innerHTML` | 없음 |

추가로 소스 기준 검색도 수행했다.

```bash
grep -R -n --exclude-dir=node_modules --exclude-dir=dist --exclude='*.wasm' 'innerHTML' rhwp-studio/src rhwp-firefox/content-script.js
grep -R -n --exclude-dir=node_modules --exclude-dir=dist --exclude='*.wasm' 'document\.write' rhwp-studio/src rhwp-firefox/content-script.js rhwp-firefox/build.mjs rhwp-firefox/manifest.json
```

결과: 매치 없음.

## 7. TypeScript 검증 상태

Stage 2에서 `rhwp-studio` 타입 검증을 시도했으나 기존 타입 정의 불일치로 실패했다.

```text
src/core/wasm-bridge.ts(133,21): error TS2551: Property 'exportHwpx' does not exist on type 'HwpDocument'.
src/core/wasm-bridge.ts(137,22): error TS2339: Property 'getSourceFormat' does not exist on type 'HwpDocument'.
```

Stage 3의 Firefox 빌드는 `npx vite build --config rhwp-firefox/vite.config.ts` 경로로 성공했으므로, 확장 산출물 생성 자체는 이 타입 오류에 막히지 않았다.

## 8. Stage 3 결론

Stage 3 완료 기준을 충족했다.

- Firefox 확장 빌드 성공
- `dist/manifest.json`에 `strict_min_version: "142.0"` 반영
- 오래된 viewer 번들 누적 제거
- stale test artifact 제거
- `document.write` 제거 확인
- `Function` 생성자 워닝 미재현 확인
- `innerHTML` 문자열 제거 확인
- 로컬 Firefox 수동 비교 전 사전 빌드 검증 완료

## 9. 승인 요청

Stage 4 착수를 요청한다.

Stage 4에서는 다음 문서를 작성한다.

- `mydocs/report/amo_reviewer_note_task338.md`
- `mydocs/working/task_m100_338_report.md`
- `mydocs/orders/20260426.md` 상태 갱신
