# Task M100 #338 최종 보고서: Firefox AMO 워닝 해결

- **타스크**: [#338](https://github.com/edwardkim/rhwp/issues/338)
- **브랜치**: `local/task338`
- **작성일**: 2026-04-26
- **마일스톤**: M100 (v1.0.0)

## 1. 목표

Firefox AMO 제출 시 발생한 다음 워닝을 해결 또는 설명 가능 상태로 만든다.

- `strict_min_version`과 `data_collection_permissions` 지원 버전 모순
- 빌드 산출물의 `Function`, `innerHTML`, `document.write` 정적 분석 워닝
- 오래된 dist 산출물이 AMO 스캔 대상에 남는 문제

## 2. 완료 내용

### 2.1 manifest 워닝 해결

`rhwp-firefox/manifest.json`의 Gecko 최소 버전을 상향했다.

```json
"strict_min_version": "142.0"
```

`data_collection_permissions`는 AMO 제출 요구에 따라 유지했다.

빌드 후 `rhwp-firefox/dist/manifest.json`에도 동일하게 반영됨을 확인했다.

### 2.2 stale dist 제거

`rhwp-firefox/build.mjs`에 빌드 시작 전 `dist/` 정리 단계를 추가했다.

효과:

- 과거 `viewer-*.js` 누적 제거
- stale test artifact 제거
- AMO 제출 패키지가 최신 산출물만 포함

빌드 후 `dist/assets`에는 현재 viewer JS 1개만 남았다.

### 2.3 `document.write` 제거

`rhwp-studio/src/command/commands/file.ts`의 인쇄 팝업 생성 로직에서 `document.write`를 제거했다.

변경 후:

- print window 문서를 DOM API로 구성
- 파일명/라벨은 `textContent` 사용
- SVG 페이지는 `DOMParser`로 SVG XML 파싱 후 import
- 버튼 이벤트는 DOM API로 바인딩

빌드 후 `rhwp-firefox/dist`에서 `document.write` 검색 결과 없음.

### 2.4 `innerHTML` 경로 제거

다음 코드를 DOM API 기반으로 변경했다.

- `rhwp-studio/src/command/extension-api.ts`: 외부 커맨드 메뉴 label/shortcut 삽입
- `rhwp-studio/src/view/canvas-view.ts`: 컨테이너 비우기
- `rhwp-firefox/content-script.js`: 정적 스캔 노이즈를 만드는 주석 문자열 정리
- `rhwp-studio/src/engine/table-object-renderer.ts`: 표/그림 선택 오버레이 SVG 생성
- `rhwp-studio/src/engine/input-handler.ts`: 다각형/도형 배치 프리뷰 SVG 생성
- `rhwp-studio/src/ui/*`: 다이얼로그/드롭다운/프리뷰의 잔여 `innerHTML` 사용 제거

최종 빌드 후 `rhwp-firefox/dist`에서 `innerHTML` 검색 결과가 없음.

## 3. 검증 결과

### 3.1 빌드

```bash
cd rhwp-firefox
npm run build
```

결과: 성공.

비차단 Vite 경고:

- `/images/icon_small_ko.svg` 런타임 resolve 경고
- viewer chunk 500KB 초과 경고

### 3.2 패턴 검색

```bash
rg -l "document\\.write" rhwp-firefox/dist --glob '!*.wasm'
rg -l "new Function|Function\\(" rhwp-firefox/dist --glob '!*.wasm'
rg -l "innerHTML" rhwp-firefox/dist --glob '!*.wasm'
```

결과:

| 패턴 | 결과 |
|---|---|
| `document.write` | 없음 |
| `new Function` / `Function(` | 없음 |
| `innerHTML` | 없음 |

AMO 제출용 설명은 `mydocs/report/amo_reviewer_note_task338.md`에 작성했다.

### 3.3 TypeScript 검증

`rhwp-studio`에서 타입 검증을 시도했다.

```bash
./node_modules/.bin/tsc --noEmit
```

결과: 실패.

원인:

```text
src/core/wasm-bridge.ts(133,21): error TS2551: Property 'exportHwpx' does not exist on type 'HwpDocument'.
src/core/wasm-bridge.ts(137,22): error TS2339: Property 'getSourceFormat' does not exist on type 'HwpDocument'.
```

판정:

- 이번 수정 파일이 아니라 기존 `pkg/rhwp.d.ts`와 `wasm-bridge.ts`의 타입 정의 불일치
- Firefox 확장 빌드는 성공했으므로 AMO 산출물 생성은 완료
- 별도 후속으로 `pkg/rhwp.d.ts` 갱신 또는 WASM 타입 생성 절차 정비 필요

### 3.4 브라우저 수동 검증

Firefox 확장을 로드한 상태에서 변경 전/후 산출물을 기준으로 주요 DOM API 전환 경로를 수동 확인했다.

최종 `innerHTML` 제거 후에는 인앱 브라우저가 아니라 로컬 Firefox 창을 직접 조작했다. 변경 전 산출물과 변경 후 산출물은 서로 다른 로컬 빌드로 제공해 비교했으며, 구체적인 로컬 포트는 최종 보고서의 재현 필수 정보가 아니므로 생략한다.

| 항목 | 결과 |
|---|---|
| 콘텐츠 스크립트 배지 표시 및 viewer 열기 | 정상 |
| HWP 문서 로드 (`samples/복학원서.hwp`) | 정상 |
| 인쇄 팝업 (`document.write` 제거 경로) | 정상 |
| 새 문서 열기 후 기존 문서 제거 | 정상 |
| 다른 HWP 문서로 교체 후 이전 페이지 제거 | 정상 |
| 표 삽입 후 표 객체 선택 overlay 표시 | 정상 |
| 그림 선택 overlay 표시 | 정상 |
| 선/사각형/타원/호/다각형 선택 overlay 표시 | 정상 |
| 도형 배치 프리뷰 및 크기 라벨 표시 | 정상 |
| 도형 선택 팝업 변경 전/후 비교 | 정상 |
| 문자표 다이얼로그 변경 전/후 비교 | 정상 |
| 표 선택 팝업 변경 전/후 비교 | 정상 |
| 문단 모양 다이얼로그 변경 전/후 비교 | 정상 |

특이사항:

- 표 객체 선택 핸들은 표시되지만 핸들 드래그로 표 전체 크기가 바뀌지는 않았다.
- `upstream/devel` 최신 코드(`42744b9`)를 확인한 결과, 표 객체 핸들은 hover cursor 변경까지만 연결되어 있고 mouse down 시 resize 상태로 진입하는 구현이 없다.
- 표 내부 셀/행/열 경계선 resize와 키보드 resize는 별도 구현되어 있으므로, 위 현상은 이번 DOM API 전환의 회귀가 아니라 기존 미구현 기능으로 판단했다.

## 4. 변경 파일

```text
rhwp-firefox/build.mjs
rhwp-firefox/content-script.js
rhwp-firefox/manifest.json
rhwp-studio/src/command/commands/file.ts
rhwp-studio/src/command/extension-api.ts
rhwp-studio/src/engine/input-handler.ts
rhwp-studio/src/engine/table-object-renderer.ts
rhwp-studio/src/engine/caret-renderer.ts
rhwp-studio/src/engine/selection-renderer.ts
rhwp-studio/src/ui/about-dialog.ts
rhwp-studio/src/ui/bookmark-dialog.ts
rhwp-studio/src/ui/char-shape-dialog.ts
rhwp-studio/src/ui/command-palette.ts
rhwp-studio/src/ui/dom-utils.ts
rhwp-studio/src/ui/equation-editor-dialog.ts
rhwp-studio/src/ui/font-set-dialog.ts
rhwp-studio/src/ui/formula-dialog.ts
rhwp-studio/src/ui/goto-dialog.ts
rhwp-studio/src/ui/numbering-dialog.ts
rhwp-studio/src/ui/page-setup-dialog.ts
rhwp-studio/src/ui/para-shape-dialog.ts
rhwp-studio/src/ui/para-shape-tab-builders.ts
rhwp-studio/src/ui/picture-props-dialog.ts
rhwp-studio/src/ui/shape-picker.ts
rhwp-studio/src/ui/style-dialog.ts
rhwp-studio/src/ui/style-edit-dialog.ts
rhwp-studio/src/ui/symbols-dialog.ts
rhwp-studio/src/ui/table-cell-props-dialog.ts
rhwp-studio/src/ui/table-create-dialog.ts
rhwp-studio/src/ui/toolbar.ts
rhwp-studio/src/view/canvas-view.ts
mydocs/orders/20260426.md
mydocs/plans/task_m100_338.md
mydocs/plans/task_m100_338_impl.md
mydocs/working/task_m100_338_stage1.md
mydocs/working/task_m100_338_stage2.md
mydocs/working/task_m100_338_stage3.md
mydocs/report/amo_reviewer_note_task338.md
mydocs/report/amo_reviewer_note_task338_en.md
mydocs/working/task_m100_338_report.md
```

## 5. 잔여 리스크

| 항목 | 상태 | 대응 |
|---|---|---|
| viewer bundle의 잔여 `innerHTML` | 해결 | 최종 dist 검색 결과 없음 |
| content-script의 `innerHTML` 문자열 | 해결 | 주석 문구 정리 완료 |
| TypeScript 타입 불일치 | 남음 | 별도 타입 생성/갱신 작업 필요 |
| AMO 실제 재스캔 결과 | 미확인 | 본 dist로 AMO 제출 후 결과 확인 필요 |
| 표 객체 핸들 drag resize | 기존 미구현 | #338 범위 밖, overlay 표시 정상 확인 |

## 6. 결론

Task #338의 핵심 워닝 대응을 완료했다.

- manifest 버전 모순 해결
- stale dist 산출물 제거
- `document.write` 제거
- `Function` 생성자 워닝 미재현 확인
- `innerHTML` 정적 분석 워닝 제거
- reviewer note 작성 완료

AMO 재제출은 `rhwp-firefox/dist` 기준으로 진행하면 된다.

`rg`가 없는 환경에서는 다음 대체 명령으로 dist 패턴을 확인할 수 있다.

```bash
grep -R -l --exclude='*.wasm' 'document\.write' rhwp-firefox/dist
grep -R -l -E --exclude='*.wasm' 'new Function|Function\(' rhwp-firefox/dist
grep -R -l --exclude='*.wasm' 'innerHTML' rhwp-firefox/dist
```
