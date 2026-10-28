# Task M100 #2392 Stage 3 완료 보고 - target와 undo orchestration 분리

- 이슈: #2392
- 상위 추적: #2022
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- source 기준: `upstream/devel@af5902b659be9a4d86ad458d79c63353dba88167`
- 선행 Stage 2 commit: `d4ec54fe9998e0ea7e10306a20d7dc045cd5e978`
- 작성일: 2026-07-19

## 1. 완료 요약

patch 계산 뒤 target 선택, WASM mutation과 undo/fallback을 독립 경계로 분리했다.

- pure model에 위치 정보를 보존하는 5개 target discriminated union과 resolver를 추가했다.
- `applyPropertyPatchToWasm`에 기존 setter 5회를 정확한 인자 순서로 이동했다.
- `applyPropertyPatch`에 target snapshot, snapshot undo와 no-service fallback을 이동했다.
- `handleOk`는 null guard, form capture, patch builder, non-empty apply와 hide만 남겼다.
- target 6개 fixture와 orchestration·mutation source guard를 추가했다.
- public/WASM/package/UI 계약과 mutation surface는 변경하지 않았다.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/picture-props-apply-model.ts` | target context·union과 `resolvePicturePropsApplyTarget` 추가 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | setter adapter와 undo orchestration method 분리, `handleOk` 축소 |
| `rhwp-studio/tests/picture-props-apply-model.test.ts` | target matrix 6개 추가 |
| `rhwp-studio/tests/picture-props-undo.test.ts` | 새 method 경계 기준 empty patch·undo·fallback·setter source guard 보강 |
| `mydocs/working/task_m100_2392_stage3.md` | 본 Stage 관문 결과 |
| `mydocs/orders/20260719.md`, 계획·Stage 2 문서 | Stage 4 승인 대기 상태 반영 |

## 3. target 계약

resolver는 DOM, WASM bridge, event bus와 command service를 받지 않는다. 다음 위치 정보만 target union에
복사하며 `cellPath` object identity를 유지한다.

model의 internal export는 target context·union·resolver 3개가 계획대로 늘었지만 소비자는 dialog와 test뿐이다.
package entry, npm/editor와 다른 barrel의 re-export는 0이므로 public package surface 증가는 아니다.

| target kind | 판정 | adapter setter |
|-------------|------|----------------|
| `cell-shape` | shape/line/group/OLE + cellPath | `setCellShapePropertiesByPath` |
| `body-shape` | shape/line/group/OLE body | `setShapeProperties` |
| `header-footer-picture` | image + header/footer | `setHeaderFooterPictureProperties` |
| `cell-picture` | image + cellPath | `setCellPicturePropertiesByPath` |
| `body-picture` | image body | `setPictureProperties` |

image에 header/footer와 cellPath가 함께 있으면 header/footer를 먼저 반환한다. target fixture는 위 5개 결과와
동시 marker 우선순위를 포함해 6개이며, header/footer의 5-tuple과 cellPath identity를 직접 단언한다.

## 4. side-effect 경계

### 4.1 `applyPropertyPatchToWasm`

target kind만 switch하며 setter 5회를 이 method에 한정했다. source guard는 단순 호출 수뿐 아니라 각 case의
메서드와 인자 순서를 검증한다.

- cell shape: `sec, para, cellPath, innerControlIdx, patch`
- body shape: `sec, para, ci, patch`
- header/footer picture: `sec, outerParaIdx, outerControlIdx, para, ci, patch`
- cell picture: `sec, para, cellPath, innerControlIdx, patch`
- body picture: `sec, para, ci, patch`

### 4.2 `applyPropertyPatch`

현재 dialog 위치에서 target을 한 번 resolve한 뒤 setter closure를 snapshot/fallback이 공유한다.

- InputHandler 있음: `kind: 'snapshot'`, `operationType: 'objectProps'`
- snapshot callback: setter 실행 후 `getCursorPosition()` 반환
- InputHandler 없음: setter 실행 후 `document-changed` 1회
- WASM setter 직접 호출: 0

### 4.3 `handleOk`

patch key가 0개면 `applyPropertyPatch`를 호출하지 않으며 마지막 `hide()`는 유지한다. setter와 undo 구현은
`handleOk` source에서 제거했고 source guard가 이 경계를 고정한다.

## 5. metrics

동일 pre snapshot 대비:

| 지표 | pre | Stage 2 | Stage 3 | pre 대비 delta |
|------|----:|--------:|--------:|---------------:|
| included files | 215 | 216 | 216 | +1 |
| reported functions | 2,386 | 2,405 | 2,407 | +21 |
| Total CC | 12,369 | 12,095 | 12,093 | -276 |
| Top 20 합 | 2,660 | 2,359 | 2,359 | -301 |
| CC>25 개수 | 70 | 69 | 69 | -1 |
| CC>25 합 | 4,297 | 3,949 | 3,949 | -348 |
| CC>100 개수 | 7 | 6 | 6 | -1 |
| Max CC | 453 | 453 | 453 | 0 |

대상 경계 상세:

| 항목 | pre dialog | Stage 2 dialog/model | Stage 3 dialog/model |
|------|-----------:|--------------------:|--------------------:|
| combined physical LOC | 2,825 | 3,057 | 3,167 |
| combined code LOC | 2,562 | 2,783 | 2,886 |
| combined reported functions | 35 | 54 | 56 |
| combined Total CC | 647 | 373 | 371 |
| combined CC>25 개수 / 합 | 2 / 560 | 1 / 212 | 1 / 212 |
| combined Max CC | 348 | 212 | 212 |
| `handleOk` CC / LOC | 348 / 381 | 5 / 59 | 2 / 11 |

Stage 3 신규·추출 함수:

| 함수 | CC / LOC |
|------|---------:|
| `resolvePicturePropsApplyTarget` | 5 / 48 |
| `applyPropertyPatchToWasm` | 1 / 29 |
| `applyPropertyPatch` | 2 / 27 |

신규 helper CC>25는 0이다. Stage 3는 target type과 검증 경계를 추가하면서도 Stage 2 대비 Total CC가 2
더 감소했다. 물리 LOC 증가는 target union과 fixture/source guard가 원인이며 고복잡도 이동은 없다.

- pre metrics SHA-256: `02ab67076683a091b1c77f1c9c9889867af42f100dc7fc6ef6092485a59f5a93`
- Stage 2 metrics SHA-256: `d8657044e924dda623054a2feaaed5e16100b21f27f4441a387c0fbf4f6a929c`
- Stage 3 metrics SHA-256: `563f221ba479a3ab1bbb1646a46a54847fc6880d06af8f6f5a4e9b5c0b44c260`
- measured source clean: false. Stage 3 source 2개가 dirty path로 기록된 구현 중간 snapshot

metrics output은 ignore된 `output/frontend-metrics/task2392/stage3/`에 두고 commit하지 않는다. Stage 4에서
최종 clean-source snapshot과 official baseline 비교를 다시 생성한다.

## 6. 검증

| Gate | 결과 |
|------|------|
| focused model/undo/Through/mutation tests | PASS, 38/38 |
| target fixture | PASS, 6/6 |
| `npm --prefix rhwp-studio test` | PASS, 390/390 |
| `npm --prefix rhwp-studio run build` | PASS |
| frontend metrics pre 비교 | PASS, Total CC·CC>25 합 순감소 |
| mutation routing 원장 | PASS, dialog setter baseline 5 유지 |
| setter method·인자 순서 source guard | PASS, 5/5 |
| `git diff --check` | PASS |

build의 CanvasKit `fs`/`path` externalize와 500 kB chunk 경고는 기존 비차단 경고다. Stage 3는 DOM/CSS와
렌더링 동작을 바꾸지 않아 visual/E2E는 계획대로 Stage 4 full gate에서 실행한다.

Stage 시작 시 `upstream/devel@af5902b6`과 #2370/#2394를 재확인했다. 두 PR 모두 OPEN·BEHIND이며 대상
dialog를 수정하지 않는다. #2394 merge 시 global metrics 변화 가능성은 Stage 4 시작 전에 다시 확인한다.

## 7. Stage 4 관문

- [x] target 5종과 header/footer 우선 parity
- [x] cellPath identity와 setter 인자 순서 보존
- [x] dialog mutation call 5회 유지
- [x] snapshot/fallback source guard PASS
- [x] empty patch mutation/event 0 경계 유지
- [x] `handleOk` CC 2 / LOC 11
- [x] 신규·추출 helper CC>25 0
- [x] 전체 suite·build PASS
- [x] 작업지시자의 Stage 4 승인

승인 후에만 headless undo E2E, final/official metrics, allowlist·public surface audit와 최종 보고서·PR 초안을
작성한다. 승인 전에는 push, PR 생성, GitHub comment·issue 편집을 수행하지 않는다.
