# Task M100 #2124 — WASM JSON schema advisory snapshot

- 이슈: #2124
- 상위 umbrella: #2022
- 선행 계획 이슈: #2023
- Rust 리팩터링 기준 이슈: #1883
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `3077f96d1f9931c50d6d62be77b389d4f66470a9`
- 작성일: 2026-07-10
- 단계: Phase 0 baseline freeze / Stage 3

## 1. 목적

이 문서는 프론트 소비자 관점에서 WASM JSON 경계를 advisory snapshot으로 고정한다. `rhwp-studio`는
`HwpDocument` WASM API가 반환하는 JSON string을 `WasmBridge`에서 파싱하고, 일부 편집 API에는
`JSON.stringify(...)`로 options/properties/path를 전달한다.

이 문서는 Rust schema의 정식 정의가 아니다. 정식 schema와 semantics는 Rust/WASM API 쪽이 소유하며,
프론트 문서는 현재 소비 패턴과 리팩터링 guardrail을 기록한다.

## 2. 소유권 경계

| 영역 | 소유권 | #2124에서 하는 일 | #2124에서 하지 않는 일 |
|------|--------|-------------------|------------------------|
| WASM API schema/semantics | Rust/core | 프론트 소비자가 기대하는 shape 기록 | Rust API 변경, schema rename, serialization 변경 |
| fresh `pkg/rhwp.d.ts` generated API | wasm-bindgen output | Rust export와 생성 declaration 일치 검증 | generated file 수동 수정 |
| `rhwp-studio/src/core/wasm-bridge.ts` | frontend adapter | JSON parse/stringify boundary와 typed return 후보 기록 | adapter 구조 리팩터링 |
| `rhwp-vscode/src/webview/viewer.ts` | VS Code webview consumer | 핵심 JSON parse 지점 기록 | VS Code message/API 변경 |
| `npm/README.md` | public npm 문서 | `@rhwp/core` public guidance 상호 참조 | npm API 변경 |

Rust 리팩터링(#1883)의 SOLID/복잡도 개선과 맞물릴 수 있으나, #2124는 frontend baseline freeze이므로
계약 변경을 하지 않는다.

## 3. JSON boundary 분류

| 분류 | 예시 API/위치 | 현재 shape | 리팩터링 guardrail |
|------|---------------|------------|--------------------|
| document/page info | `getDocumentInfo()`, `getPageInfo()` | JSON string -> `DocumentInfo`, `PageInfo` | field name/casing 변경 금지 |
| layout tree/plan | `getPageLayerTree*()`, `getCanvasKitReplayPlan()` | JSON string, object tree | parse fallback과 shape 검증 유지 |
| cursor/hit/selection | `getCursorRect*()`, `hitTest*()`, `getSelectionRects*()` | JSON string -> typed position/rect/result | nullable/fallback semantics 보존 |
| table/cell | `getTableDimensions()`, `getCellInfo()`, `getTableCellBboxes()`, `setCellProperties()` | JSON string input/output | cell path key shape 변경 금지 |
| object properties | picture/shape/equation/note APIs | `props` object -> JSON string, result JSON string | property key를 별도 migration 없이 rename 금지 |
| formatting/style | char/para/style/numbering/bullet APIs | properties/list/detail JSON | HWPUNIT/px 단위 의미 보존 |
| header/footer | header/footer rect/list/hit/template APIs | JSON result, primitive args | applyTo/isHeader semantics 보존 |
| fields/forms/search | field list/value/info, form value/info, search/replace | JSON result, 일부 valueJson input | form/field result envelope 유지 |
| large option APIs | `*Ex(options_json)` in `pkg/rhwp.d.ts` | camelCase options JSON string | binary data는 별도 인자로 유지 |

## 4. 주요 TypeScript 소비 타입

`rhwp-studio/src/core/types.ts`가 프론트 소비자 관점의 대표 shape를 담고 있다.

| 타입 | 용도 | 중요 필드/주의 |
|------|------|----------------|
| `DocumentInfo` | 문서 로드/상태 | `version`, `sectionCount`, `pageCount`, `encrypted`, `fallbackFont`, `fontsUsed` |
| `PageInfo` | 페이지 레이아웃 | `pageIndex`, `pageNumber?`, `width`, `height`, margins, `columns?` |
| `PageDef`, `SectionDef` | 쪽/구역 설정 | HWPUNIT 원본값 포함 |
| `CursorRect` | 커서 표시 | `pageIndex`, `x`, `y`, `height` |
| `HitTestResult` | 클릭 위치 -> 문서 위치 | 본문/셀/글상자/필드 context optional field |
| `CellPathEntry`, `CellPathSegment` | 중첩 표 path | long key와 short key 형식이 공존 |
| `TableDimensions`, `CellInfo`, `CellBbox` | 표 구조와 bbox | row/col/span/page coordinate |
| `CellProperties`, `TableProperties` | 표/셀 속성 | HWPUNIT, border/fill, layout 옵션 |
| `CharProperties`, `ParaProperties` | 글자/문단 서식 | 일부 px resolved value와 HWPUNIT value가 혼재 |
| `ControlLayoutItem`, `ObjectRef` | 그림/도형/수식 선택 | z-order, plane, note/header-footer context |
| `FieldInfoResult` | 누름틀/필드 | `fieldId`, `fieldType`, guide/editable flags |

Phase B에서 typed adapter를 분리하더라도 이 타입 의미를 우선 보존해야 한다.

## 5. 현재 `WasmBridge` JSON 입출력 패턴

### 5.1 JSON parse output

대표 output:

- `loadDocument()` / `createNewDocument()`는 `getDocumentInfo()` 또는 `createBlankDocument()` JSON을
  `DocumentInfo`로 파싱한다.
- `getPageInfo()`, `getDocumentInfo()`, `getPageDef()`, `getSectionDef()`, `getPageBorderFill()`은 JSON string을
  typed object로 반환한다.
- `getPageLayerTreeObject()`는 `root.kind` 등 최소 shape를 검증하고, legacy `layers` shape에 fallback한다.
- 표/셀/개체/필드/검색 API 다수가 Rust JSON string을 `JSON.parse(...)` 후 typed object로 반환한다.
- 일부 getter는 API 부재 또는 parse 실패 시 `null`, `[]`, `{ ok: false }`, `{ hit: false }` 등 fallback을 반환한다.

### 5.2 JSON stringify input

대표 input:

- `setPageDef()`, `setSectionDef()`, `setPageBorderFill()`은 settings object를 JSON string으로 전달한다.
- `setCellProperties()`, `setTableProperties()`, `resizeTableCells()`는 props/update list를 JSON string으로 전달한다.
- by-path API는 `cellPath`를 JSON string으로 전달한다.
- object property API는 picture/shape/equation/note props를 JSON string으로 전달한다.
- style/format APIs 중 일부는 이미 JSON string을 받으며, 일부 wrapper는 object를 stringify한다.
- `createTableEx(options)`는 options object를 JSON string으로 전달한다.

## 6. `*Ex(optionsJson)` 계약

Rust `src/wasm_api.rs`의 export와 fresh `pkg/rhwp.d.ts`에는 여러 `*Ex(options_json: string)` API가
존재해야 한다. `npm/README.md`는 인자가 많은 편집 API에서 options object 변형을 권장하며, 다음
계약을 제시한다.

| 항목 | 계약 |
|------|------|
| 인자 | `optionsJson` string |
| key style | camelCase |
| binary data | JSON에 넣지 않고 별도 `Uint8Array` 인자로 전달 |
| 반환 | 기존 positional API와 동일한 의미의 string/object result |
| API discovery | `rhwp.d.ts`에서 `Ex(options` 검색 |

확인된 `*Ex` 예시는 다음 범주에 걸쳐 있다.

- 셀 텍스트/선택/삭제/붙여넣기: `insertTextInCellEx`, `deleteTextInCellEx`, `getTextInCellEx`,
  `copySelectionInCellEx`, `pasteHtmlInCellEx`, `deleteRangeInCellEx`
- 이동/선택: `moveVerticalEx`, `getSelectionRectsInCellEx`, `moveLineEndpointEx`
- 표: `createTableEx`, `mergeTableCellsEx`, `splitTableCellIntoEx`, `splitTableCellsInRangeEx`,
  `evaluateTableFormulaEx`
- 그림/필드/양식: `insertPictureEx`, `insertClickHereField*Ex`, `removeFieldAtInCellEx`,
  `setActiveFieldInCellEx`, `setFormValueInCellEx`
- 서식/기타: `applyCharFormatInCellEx`, `setCharShapeIdInCellEx`, `setNoteEquationPropertiesEx`,
  `setPageHideEx`

Phase B에서 options object를 typed wrapper로 감싸더라도 public `options_json` WASM 계약을 변경하지 않는다.

## 7. Cross-surface 소비자

### 7.1 `rhwp-studio`

`WasmBridge`가 가장 큰 JSON boundary다. 현재는 프론트 전체에 WASM raw call이 일부 남아 있더라도,
핵심 `HwpDocument` lifecycle, render, page info, edit command는 `WasmBridge`를 중심으로 소비된다.

리팩터링 방향:

- JSON parse/stringify를 더 흩뿌리지 않는다.
- 새 기능은 가능한 한 `WasmBridge` 또는 별도 typed adapter를 통과시킨다.
- `any` cast를 늘리지 않는다. API availability fallback이 필요하면 좁은 interface cast를 우선한다.

### 7.2 VS Code webview

VS Code webview는 별도 consumer로 `HwpDocument`를 직접 사용한다.

현재 핵심 소비:

- `await init({ module_or_path: buf })`
- `new HwpDocument(fileBytes)`
- `JSON.parse(hwpDoc.getDocumentInfo())`
- `JSON.parse(hwpDoc.getPageInfo(i))`
- `renderPageSvg(i)`

VS Code viewer는 `docInfo.page_count ?? docInfo.pageCount ?? 0` fallback을 갖고 있다. `pageCount` casing은
front/core 간 호환 이슈가 있었던 흔적으로 보이며, 임의 제거하지 않는다.

### 7.3 `@rhwp/core` README

public guidance는 다음을 전제로 한다.

- `measureTextWidth`는 WASM init 전에 등록한다.
- `HwpDocument` 생성자는 `Uint8Array` 문서 bytes를 받는다.
- 문서 정보와 편집 결과 중 다수는 JSON string이다.
- 큰 옵션 API는 `*Ex(optionsJson)`를 사용한다.

이 guidance와 어긋나는 frontend-only wrapper 변경은 downstream 혼란을 만들 수 있으므로 별도 issue가 필요하다.

## 8. 리팩터링 금지선

Phase 0/Phase A에서 유지해야 할 금지선:

- Rust/WASM JSON field 이름, casing, 단위(HWPUNIT/px), fallback semantics를 변경하지 않는다.
- `getDocumentInfo()` / `getPageInfo()` 등 JSON string 반환 API를 raw object 반환으로 바꾸지 않는다.
- `*Ex(options_json)` public WASM API를 임의로 제거하거나 positional-only API로 되돌리지 않는다.
- `CellPathEntry`/`CellPathSegment`처럼 이미 두 key 형식이 공존하는 path 계약을 단일 형식으로 강제하지 않는다.
- parse 실패 fallback을 제거하지 않는다. 제거가 필요하면 해당 fallback이 감추던 브라우저/문서 사례를 먼저 확인한다.
- typed adapter를 추가하더라도 `@rhwp/core` public 문서와 `pkg/rhwp.d.ts` generated API를 변경하지 않는다.
- VS Code webview의 direct `HwpDocument` 소비를 `rhwp-studio` 내부 adapter에 강제로 의존시키지 않는다.

## 9. Phase B 후보 작업

이 snapshot 이후 가능한 리팩터링 후보:

| 후보 | 목적 | 선행 조건 |
|------|------|-----------|
| `WasmBridge` JSON result adapter 분리 | parse/stringify 책임 축소 | Stage 4 smoke gate 확정 |
| Rust-generated schema 문서화 | frontend/Rust field mismatch 감소 | Rust maintainer와 schema 소유권 합의 |
| `*Ex` typed option builders | optionsJson 생성 중복 감소 | public WASM API shape 유지 |
| VS Code viewer WASM consumer 문서화 | studio와 별도 소비자 회귀 방지 | VS Code smoke manifest |
| JSON fallback audit | nullable/fallback semantics 명시 | sample 문서와 regression test 확보 |

## 10. #1883과의 관계

#1883의 SOLID/복잡도 리팩터링은 Rust/core를 중심으로 진행된다. frontend 리팩터링은 동일하게
정량 baseline과 review-gated 단계화를 따르되, WASM JSON schema 자체를 frontend가 소유한다고 가정하지
않는다.

따라서 #2124의 결론은 다음이다.

- 프론트는 JSON boundary의 소비 책임과 adapter 책임을 문서화한다.
- Rust-owned API 변경은 #1883 계열 또는 별도 core issue에서 다룬다.
- 프론트 Phase B는 contract-preserving adapter/refactor부터 시작한다.

## 11. Generated binding 일치 gate

저장소에서 `pkg/`는 generated·ignored output이므로 로컬 파일의 시점만으로 public API를 판정하지 않는다.
초기 기준선 검증(`782059d9`)의 repository Docker build 전 stale `pkg/rhwp.d.ts`에는 당시 기준 Rust
source의 다음 네 export가 없었다.

- `flushDeferredPagination`
- `getCursorRectByPathNear`
- `getStructure`
- `insertTextInCellDeferredPagination`

`scripts/frontend-wasm-bindings.test.mjs`는 `src/wasm_api.rs`의 명시적
`#[wasm_bindgen(js_name = ...)]` 목록을 fresh declaration과 비교한다. build 전 검사가 네 누락을
탐지한 뒤 `docker-compose --env-file .env.docker run --rm wasm`으로 release output을 다시 생성했다.
동일 검사는 fresh output에서 통과했고 Studio build와 VS Code compile도 통과했다. 이 전후 결과는
generated output을 source of truth로 오인하지 않으면서 stale 상태를 조기에 찾는 gate의 근거다.

최종 기준 `6f1bd284`로 동기화한 뒤에는 pre-build declaration도 explicit export 검사를 통과했다.
upstream Rust 변경을 반영한 fresh WASM을 다시 생성하고 동일 gate와 consumer build를 재실행해 최종
snapshot의 binding 일치를 확인했다.
