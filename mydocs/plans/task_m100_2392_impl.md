# Task M100 #2392 구현 계획서 — `picture-props-dialog` 적용 파이프라인 책임 분리

- 이슈: #2392
- 상위 추적: #2022
- 선행 계획: #2023 v2 / PR #2080
- 선행 기준선: #2124 / PR #2174
- 선행 Phase A: #2125 / PR #2254
- 선행 legacy 정리: #2313 / PR #2316
- 마일스톤: M100 / v1.0.0
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `af5902b659be9a4d86ad458d79c63353dba88167`
- 작성일: 2026-07-19
- 단계: 구현 계획서
- 승인 상태: 작업지시자 승인 완료
- 진행 상태: Stage 4 local gate·metrics·GitHub 초안 완료, 원격 게시 승인 대기
- 선행 수행 계획서: `mydocs/plans/task_m100_2392.md`

## 1. 구현 목표

`PicturePropsDialog.handleOk`에서 form snapshot 수집, patch 계산, target 선택, WASM mutation, undo/fallback과
dialog 종료를 분리한다.

구현은 다음 조건을 동시에 만족해야 한다.

1. `handleOk`는 입력 snapshot 생성, pure patch builder 호출, side-effect adapter 호출과 종료만 조정한다.
2. patch 계산과 target 선택은 DOM, `WasmBridge`, `EventBus`, `CommandServices`에 runtime 의존하지 않는다.
3. 5개 WASM setter 호출은 기존 `picture-props-dialog.ts`에 남겨 mutation surface를 새 파일로 확대하지 않는다.
4. image/shape/line/group/OLE의 field/default/diff/always-send 의미를 바꾸지 않는다.
5. header/footer, cell path, 일반 본문의 target 우선순위와 setter 인자 순서를 바꾸지 않는다.
6. snapshot undo, no-service fallback, empty patch와 dialog 종료 의미를 바꾸지 않는다.
7. `populateFromProps`, UI, public/WASM/package contract를 수정하지 않는다.
8. 함수 분할 뒤 frontend Total CC와 CC>25 합이 동일 base에서 순감소해야 한다.

## 2. 확정 구현 결정

| 항목 | 결정 |
|------|------|
| pure policy 위치 | `rhwp-studio/src/ui/picture-props-apply-model.ts` 신설 |
| runtime import | 없음. core type은 `import type`만 허용 |
| patch 입력 | DOM element가 아닌 책임군별 raw form snapshot |
| patch 출력 | internal `PicturePropsPatch = Record<string, unknown>` |
| patch 분리 | common / shape·OLE / image와 shared transform·margin·caption·line group |
| target 판정 | pure discriminated union으로 5개 target 반환 |
| actual setter | 기존 dialog의 private adapter method에서 호출 |
| undo/fallback | 기존 dialog의 별도 private method에서 수행 |
| dialog 진입점 | `handleOk`를 얇은 coordinator로 축소 |
| 단위 테스트 | `rhwp-studio/tests/picture-props-apply-model.test.ts` 신설 |
| source guard | `picture-props-undo.test.ts`를 새 orchestration method 기준으로 보강 |
| mutation 원장 | `picture-props-dialog.ts` 5회 baseline 유지, 새 mutation 파일·호출 추가 금지 |
| 기존 Through guard | `getSelectedWrap`과 `wrap-through-preserve.test.ts` 유지 |
| public surface | package/barrel export 없음. 새 export는 internal module 소비와 test 전용 |
| `core/types.ts` | 수정하지 않음 |
| lockfile/package | 수정하지 않음 |
| visual gate | DOM/CSS diff가 없으면 별도 sweep 미실행, undo E2E screenshot을 보조 evidence로 사용 |

별도 module 자체가 목적은 아니다. pure builder가 거대 flat context를 요구하거나 CC>25 helper를 만들면 구현을
중단하고 grouped snapshot 경계를 다시 설계한다.

## 3. 최신 upstream과 병행 작업 처리

Stage 1 최신 통합 시점의 관련 PR은 #2370, #2394와 merge된 #2395다.

| PR | #2392 영향 | 처리 |
|----|------------|------|
| #2370 | OPEN. `insert.ts`의 회전/대칭 중복 emit만 제거, dialog 생성 경로 미변경 | Stage 시작 전 files 재확인 |
| #2394 | 대상 dialog 미변경. `core/types.ts`, `wasm-bridge.ts`와 전역 metrics 변화 가능 | merge 시 최신 base 통합 후 pre metrics 재생성 |
| #2395 | MERGED `b507c1d0`. CI·문서만 변경, 제품 metrics 모집단 미변경 | upstream #2393·이월 내용과 #2392 행 병합 완료 |

현재 numeric baseline 12,369/4,297은 제품 tree가 동일한 `af5902b6` pre snapshot에 적용한다. source 구현 전
또는 PR 직전 upstream이 바뀌면 target file 수치가 동일한지 먼저 확인하고 global pre 값을 최신 base에서
다시 고정한다. upstream 누적 delta를 #2392 개선으로 계상하지 않는다.

#2395 통합에서는 승인된 계획 문서를 먼저 commit한 뒤 사용자 승인 하에 rebase했다. orders add/add conflict는
upstream의 #2393·이월 항목과 #2392 행을 모두 남겨 해결했다. `eb9c7f1f..1cfb4273`의 frontend 제품 모집단
diff가 없음을 확인하고 pre metrics를 최신 commit metadata로 재생성했다.

Stage 2 시작 전 `1cfb4273..af5902b6`도 #2393 보고서 7줄만 변경하고 frontend 제품 모집단은 바꾸지 않음을
확인했다. rebase 후 계획 commit은 `293c7640`, Stage 1 commit은 `8773d6b4`이며 pre snapshot은 최신
upstream metadata로 다시 생성했다.

## 4. 변경 파일과 경로

### 4.1 product source

| 파일 | 정확한 변경 |
|------|-------------|
| `rhwp-studio/src/ui/picture-props-apply-model.ts` | grouped raw snapshot type, patch builder, target resolver 신설 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | raw snapshot capture, pure builder 호출, setter/undo adapter method 분리, `handleOk` 축소 |

다음 파일은 수정하지 않는다.

- `rhwp-studio/src/core/types.ts`
- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler.ts`
- `rhwp-studio/src/command/types.ts`
- `rhwp-studio/src/ui/**/*.css`
- `rhwp-studio/package.json`, `package-lock.json`
- Rust/WASM source와 generated `pkg/`
- extension, VS Code, npm/editor source

### 4.2 tests

| 파일 | 정확한 변경 |
|------|-------------|
| `rhwp-studio/tests/picture-props-apply-model.test.ts` | patch와 target matrix data-driven test 신설 |
| `rhwp-studio/tests/picture-props-undo.test.ts` | `handleOk` 내부 문자열 위치 의존을 새 orchestration method 계약으로 갱신 |

다음 test는 검증만 하고 수정하지 않는다.

- `rhwp-studio/tests/wrap-through-preserve.test.ts`
- `rhwp-studio/tests/mutation-routing-guard.test.ts`
- `rhwp-studio/tests/user-settings.test.ts`
- `rhwp-studio/e2e/undo-contracts.test.mjs`

`mutation-routing-guard.test.ts`의 `src/ui/picture-props-dialog.ts: 5` baseline은 유지해야 한다. 새 pure module에
`wasm.*` mutation call이 생기거나 기존 dialog 호출 수가 증가하면 구현 오류로 처리한다.

### 4.3 task evidence

| 파일 | 변경 |
|------|------|
| `mydocs/orders/20260719.md` | Stage 진행·완료 상태와 upstream #2395 내용 통합 |
| `mydocs/plans/task_m100_2392.md` | assignee·승인 상태 보정 완료 |
| `mydocs/plans/task_m100_2392_impl.md` | 본 구현 계획 |
| `mydocs/tech/investigations/issue-2392/README.md` | investigation index와 front matter |
| `mydocs/tech/investigations/issue-2392/task_m100_2392_picture_props_apply_contract.md` | field/target/undo characterization |
| `mydocs/working/task_m100_2392_stage{1..4}.md` | 단계별 관문 결과 |
| `mydocs/report/task_m100_2392_report.md` | 완료 조건과 metrics 결산 |
| `mydocs/report/task_m100_2392_pr_draft.md` | PR·review 요청 초안 |

investigation 문서는 `mydocs/README.md` 규칙에 맞는 `kind/status/canonical/last_verified` front matter를
포함한다. 일반 plan/working/report에는 해당 front matter를 강제로 추가하지 않는다.

## 5. pure apply model 설계

### 5.1 exported internal API

`picture-props-apply-model.ts`는 다음 internal API만 export한다.

| API | 책임 |
|-----|------|
| `PicturePropsObjectType` | `'image' | 'shape' | 'line' | 'group' | 'ole'` type |
| `PicturePropsApplyForm` | 책임군별 raw value snapshot |
| `PicturePropsApplyTargetContext` | sec/para/ci와 header-footer/cell target context |
| `PicturePropsApplyTarget` | 5개 target discriminated union |
| `PicturePropsPatch` | internal patch map type |
| `buildPicturePropsPatch(...)` | current props와 form snapshot을 비교해 patch 생성 |
| `resolvePicturePropsApplyTarget(...)` | object type과 context를 5개 setter target으로 판정 |

package entry, barrel 또는 npm/editor type에서 재-export하지 않는다. metrics export surface 증가가 있으면 internal
test seam임을 보고서에 명시하되 public package API 변화로 표현하지 않는다.

### 5.2 grouped raw form snapshot

하나의 flat DOM context 대신 다음 group을 사용한다.

| group | 포함 raw state | 소비자 |
|-------|----------------|--------|
| `common` | size protect, width/height, treat-as-char, wrap, position, restrict/overlap, description | 모든 object type |
| `transform` | rotation value/disabled, horizontal/vertical flip value/disabled | non-OLE shape와 image |
| `outerMargin` | left/right/top/bottom input presence와 value | OLE, image |
| `caption` | active index, size/gap, include-margin, control presence | OLE, image |
| `line` | color, width, type, end, arrow와 arrow-size control presence/value | shape 계열, image 일부 |
| `shapeTextBox` | four margins와 vertical-align active value | non-OLE shape 계열 |
| `shapeCorner` | custom mode/value와 preset active index | non-OLE shape 계열 |
| `shapeFill` | fill mode, colors, pattern, gradient와 transparency controls | non-OLE shape 계열 |
| `shapeShadow` | active type, color와 offset values | non-OLE shape 계열 |
| `image` | scale, crop, padding, effect, brightness, contrast, transparency | image |

snapshot에는 `HTMLInputElement`나 class instance를 넣지 않는다. optional control은 presence와 raw value를
구분해 기록한다. current code가 optional chaining 뒤 `|| 0`을 적용하는 field는 absence도 0으로 정규화되는
현재 의미를 builder에서 명시적으로 재현한다.

### 5.3 conversion과 comparison

다음 conversion은 module 내부 pure helper로 옮기고 builder를 통해 간접 테스트한다.

- mm text -> HWPUNIT rounding
- HTML hex -> HWP ColorRef
- integer/float `||` fallback
- transparency percent clamp와 alpha 변환
- picture effect `Original` -> `RealPic`
- caption grid index -> direction/vertical alignment

reverse conversion인 `hwpToMm`, `colorRefToHex`, `captionGridIndex`는 `populateFromProps`가 계속 사용하므로
dialog에 남긴다. apply 전용 `mmToHwp`, `hexToColorRef`, `gridIndexToCaption`만 기존 위치에서 제거한다.

patch assignment helper는 changed-only와 always-send를 이름과 API로 구분한다. 두 정책을 하나의 generic
boolean option으로 숨겨 review가 어려워지게 하지 않는다.

### 5.4 patch precedence

현재 대입 순서를 보존한다.

1. common size/position/description
2. shape·OLE 또는 image 전용 field
3. image scale이 활성화되면 common width/height patch를 최종 값으로 덮어쓸 수 있음
4. later same-key assignment가 존재하면 현재 순서를 characterization 문서에 기록

특히 `sizeProtect`, `treatAsChar`, `TakePlace`, image scale precedence는 순서를 바꾸면 patch가 달라질 수
있으므로 data-driven test로 고정한다.

### 5.5 omitted/stale control 처리

dialog는 `build()`를 한 번만 수행하고 `rebuildTabs()`에서 object type별 panel을 다시 만든다. class field가
명시적으로 reset되지 않는 control은 이전 open의 element reference를 보유할 가능성이 있다. 또한 line처럼
일부 panel을 만들지 않는 type에서도 현재 optional chaining+fallback이 0/none을 만들 수 있다.

이 현상은 #2392에서 기능 수정하지 않는다.

1. Stage 1에서 fresh open과 type 전환 sequence를 구분해 실제 raw snapshot을 확인한다.
2. 명시된 product contract와 incidental stale state를 characterization 문서에서 분리한다.
3. refactor 전후 관찰 결과는 동일해야 한다.
4. 의도 오류가 확인돼도 이번 patch에서 reset/default를 고치지 않고 후속 이슈 초안을 제시한다.

## 6. target와 side-effect adapter 설계

### 6.1 pure target resolver

`resolvePicturePropsApplyTarget`은 다음 union 중 하나를 반환한다.

| discriminator | 필요한 위치 정보 | dialog setter |
|---------------|------------------|---------------|
| `cell-shape` | sec, para, cellPath, innerControlIdx | `setCellShapePropertiesByPath` |
| `body-shape` | sec, para, ci | `setShapeProperties` |
| `header-footer-picture` | sec, outerParaIdx, outerControlIdx, para, ci | `setHeaderFooterPictureProperties` |
| `cell-picture` | sec, para, cellPath, innerControlIdx | `setCellPicturePropertiesByPath` |
| `body-picture` | sec, para, ci | `setPictureProperties` |

resolver는 image에서 header/footer를 cell path보다 먼저 판정하고 shape 계열에서 cell path를 먼저 판정한다.
WASM method를 호출하지 않으며 5개 table test로 검증한다.

### 6.2 dialog private methods

`picture-props-dialog.ts`에는 다음 private boundary를 둔다. 최종 이름은 이 목록으로 고정한다.

| method | 책임 |
|--------|------|
| `captureApplyForm()` | current DOM/class state를 grouped raw snapshot으로 복사 |
| `applyPropertyPatch(patch)` | InputHandler snapshot 또는 direct fallback 선택 |
| `applyPropertyPatchToWasm(patch)` | pure target resolver 결과에 따라 기존 5개 setter 호출 |
| `handleOk()` | null guard, builder 호출, non-empty apply와 hide coordination |

`captureApplyForm`이 길어지면 group별 private capture method를 추가할 수 있지만 각 method는 해당 group의 DOM
read만 수행한다. patch comparison이나 WASM call을 capture method에 넣지 않는다.

### 6.3 undo와 fallback

`applyPropertyPatch`는 현재 구조를 그대로 유지한다.

- InputHandler 존재: `executeOperation({ kind: 'snapshot', operationType: 'objectProps', operation })`
- operation: `applyPropertyPatchToWasm` 실행 후 `ih.getCursorPosition()` 반환
- InputHandler 부재: `applyPropertyPatchToWasm` 직접 실행 후 `document-changed` 1회 emit
- empty patch: `applyPropertyPatch`를 호출하지 않음. 정상 UI에서는 type별 always-send group 때문에 보통
  non-empty이며, 이 분기는 control group이 없는 방어적 입력 fixture로 검증
- `hide()`: `handleOk` 끝에서 1회

공용 command/router 추출이나 error policy 변경은 하지 않는다.

## 7. test fixture 설계

### 7.1 pure patch fixtures

`picture-props-apply-model.test.ts`는 최소 다음 case를 포함한다.

| 범주 | case |
|------|------|
| unchanged | image/OLE은 `hasCaption`, shape/line/group은 shadow always-send key를 유지하고 changed-only key는 추가하지 않음 |
| empty | always-send control group도 없는 방어적 snapshot은 empty patch |
| size | `sizeProtect` 변경, true일 때 width/height 억제 |
| position | treat-as-char true일 때 wrap/position 억제 |
| wrap | `TakePlace` -> `TopAndBottom`, horizontal relative omit, `Through` 보존 |
| defaults | restrict/overlap의 nullish default와 changed-only |
| OLE | outer margin·caption·line만 허용, transform/fill/shadow/arrow 억제 |
| shape | text-box margin, vertical align, rotation/flip, line/corner/fill/shadow |
| line/group | current omitted-panel/default behavior와 object type 분기 |
| caption | `hasCaption` always-send와 direction/align/size/gap/include-margin |
| fill | none/solid/gradient, pattern fallback, alpha conversion |
| shadow | type 0 offset reset과 type>0 color/offset |
| image size | scale이 common width/height보다 우선, size protect 시 억제 |
| image geometry | crop/padding/outer margin/border |
| image effect | `Original` mapping, brightness/contrast, transparency clamp |

table fixture는 input과 expected patch를 함께 보유하고 key order가 아니라 deep equality로 의미를 검증한다.
floating conversion은 current `Math.round` 결과를 exact integer로 단언한다.

### 7.2 target fixtures

5개 target과 image header/footer + cellPath 동시 존재 시 header/footer 우선을 추가해 최소 6 case를 둔다.
`cellPath` object identity와 setter 인자 위치 정보가 union에 보존되는지 확인한다.

### 7.3 source guard 보강

`picture-props-undo.test.ts`의 기존 첫 test는 `handleOk` 안에 `executeOperation` 문자열이 직접 존재한다고
가정한다. 이를 다음 의미 중심 검사로 바꾼다.

1. `handleOk`가 non-empty patch를 `applyPropertyPatch`로 위임한다.
2. `applyPropertyPatch`가 `kind: 'snapshot'`, `operationType: 'objectProps'`를 유지한다.
3. fallback은 `document-changed`를 유지한다.
4. mutation setter가 snapshot callback 밖에서 추가되지 않는다.

검사를 삭제하거나 regex 범위를 파일 전체로 넓혀 우연히 통과시키지 않는다.

### 7.4 browser characterization

기존 `undo-contracts.test.mjs`를 수정하지 않고 다음을 재실행한다.

- picture restrict-in-page apply -> undo stack 1건 -> `performUndo` 복원
- picture apply -> Escape -> 실제 Ctrl+Z -> 복원
- `Through` 상태에서 확인만 눌러도 textWrap 유지

Stage 1의 omitted/stale control 조사가 자동 test 승격이 필요하면 기존 E2E에 섞지 않고 별도 focused fixture를
먼저 제안하고 작업지시자 승인을 받는다.

## 8. Stage 1 — 계획 commit, pre metrics와 characterization

### 8.1 계획 문서 commit

구현 계획 승인 후 orders·수행 계획·구현 계획을 docs commit으로 고정한다.

```bash
git status --short --branch
git rev-parse HEAD upstream/devel
git diff --check
git add mydocs/orders/20260719.md mydocs/plans/task_m100_2392.md mydocs/plans/task_m100_2392_impl.md
git commit -m "Task #2392: 개체 속성 apply 분리 계획 확정"
```

upstream drift가 있으면 자동 통합하지 않는다. #2395가 merge된 경우 §3 방식의 orders conflict 해결안을 먼저
제시하고 사용자 승인 후 rebase/merge한다.

### 8.2 pre metrics

```bash
npm --prefix scripts/frontend-metrics ci
node scripts/frontend-metrics.mjs \
  --out output/frontend-metrics/task2392/pre/metrics.json \
  --summary output/frontend-metrics/task2392/pre/summary.md
shasum -a 256 scripts/frontend-metrics.mjs scripts/frontend-metrics/package-lock.json \
  output/frontend-metrics/task2392/pre/metrics.json
```

Stage 1 문서에는 head/upstream commit, measured-source clean, script/lock hashes, global/file aggregate와 모든
`picture-props-dialog.ts` function entry를 기록한다.

### 8.3 baseline gates

```bash
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

E2E는 terminal A에서 다음 server를 유지한다.

```bash
npm --prefix rhwp-studio run dev -- --host 127.0.0.1 --port 7700
```

terminal B에서 local Chrome path를 지정하고 실행한다.

```bash
CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  npm --prefix rhwp-studio run e2e:undo -- --mode=headless
```

Linux/CI는 환경의 Chrome executable을 사용한다. host CDP 기본값을 macOS headless 검증에 사용하지 않는다.

### 8.4 characterization 문서

- `mydocs/tech/investigations/issue-2392/README.md`
- `mydocs/tech/investigations/issue-2392/task_m100_2392_picture_props_apply_contract.md`
- `mydocs/working/task_m100_2392_stage1.md`

field별 source line, object type, control presence, conversion, baseline value, diff/always-send와 patch key를 기록한다.
target/undo matrix와 current test coverage도 같은 문서에 연결한다.

Stage 1 commit: `Task #2392: 개체 속성 apply 계약 기준선 고정`

관문: Stage 1 문서와 기준선 결과를 작업지시자가 승인한 뒤 Stage 2로 넘어간다.

## 9. Stage 2 — patch 계산 책임 분리

### 9.1 구현 순서

1. `picture-props-apply-model.ts`에 grouped raw snapshot type과 conversion helper를 추가한다.
2. common/shared/shape·OLE/image patch helper를 구현한다.
3. `buildPicturePropsPatch` coordinator를 구현한다.
4. `picture-props-dialog.ts`에 `captureApplyForm`을 추가하고 기존 `handleOk` 계산 block을 builder 호출로 교체한다.
5. actual setter와 undo block은 이 Stage에서 기존 dialog에 그대로 둔다.
6. pure patch fixture를 추가하고 기존 Studio suite를 실행한다.

### 9.2 focused 검증

```bash
node --test rhwp-studio/tests/picture-props-apply-model.test.ts
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
node scripts/frontend-metrics.mjs \
  --compare output/frontend-metrics/task2392/pre/metrics.json \
  --out output/frontend-metrics/task2392/stage2/metrics.json \
  --summary output/frontend-metrics/task2392/stage2/summary.md
```

expected gate:

- 모든 patch fixture PASS
- `handleOk` CC/LOC 감소
- 새 helper CC>25 0
- global/file Total CC와 CC>25 합이 pre보다 증가하지 않음
- mutation-routing baseline 변화 0

Stage 2 보고: `mydocs/working/task_m100_2392_stage2.md`

Stage 2 commit: `Task #2392: 개체 속성 patch 계산 책임 분리`

관문: patch fixture parity와 중간 metrics를 작업지시자가 승인한 뒤 Stage 3으로 넘어간다.

## 10. Stage 3 — target와 undo orchestration 분리

### 10.1 구현 순서

1. pure target union과 `resolvePicturePropsApplyTarget`을 구현한다.
2. target fixture 6개 이상을 추가한다.
3. dialog에 `applyPropertyPatchToWasm`을 만들고 기존 5개 setter를 그대로 이동한다.
4. dialog에 `applyPropertyPatch`를 만들고 snapshot/fallback을 그대로 이동한다.
5. `handleOk`를 null guard, capture, builder, non-empty apply와 hide coordinator로 마무리한다.
6. `picture-props-undo.test.ts`의 source guard를 새 method 경계로 갱신한다.

### 10.2 focused 검증

```bash
node --test \
  rhwp-studio/tests/picture-props-apply-model.test.ts \
  rhwp-studio/tests/picture-props-undo.test.ts \
  rhwp-studio/tests/wrap-through-preserve.test.ts \
  rhwp-studio/tests/mutation-routing-guard.test.ts
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

expected gate:

- 5개 setter target과 header/footer 우선 parity
- `picture-props-dialog.ts` mutation call count 5 유지
- snapshot/fallback source guard PASS
- empty patch에서 mutation/event 0
- `handleOk` preferred target CC<=25, LOC<=60
- 신규/추출 helper CC>25 0. 불가피하면 승인 예외 절차 적용

Stage 3 보고: `mydocs/working/task_m100_2392_stage3.md`

Stage 3 commit: `Task #2392: 개체 속성 적용 라우팅 분리`

관문: focused gate와 중간 metrics를 작업지시자가 승인한 뒤 Stage 4로 넘어간다.

## 11. Stage 4 — 회귀, metrics 결산과 PR 준비

### 11.1 full local gate

```bash
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
npm --prefix rhwp-studio run e2e:undo -- --mode=headless
git diff --check
```

E2E 실행 전 Stage 1과 같은 Vite server/Chrome 환경을 준비한다. package script만 실행해 host CDP 실패를
제품 회귀로 오분류하지 않는다.

### 11.2 final metrics

```bash
node scripts/frontend-metrics.mjs \
  --compare output/frontend-metrics/task2392/pre/metrics.json \
  --out output/frontend-metrics/task2392/post/metrics.json \
  --summary output/frontend-metrics/task2392/post/summary.md
node scripts/frontend-metrics.mjs \
  --compare mydocs/metrics/frontend/2026-07-11/metrics.json \
  --out output/frontend-metrics/task2392/post-vs-official/metrics.json \
  --summary output/frontend-metrics/task2392/post-vs-official/summary.md
```

직접 완료 기준:

| 지표 | `1cfb4273` baseline | target |
|------|---------------------:|--------|
| file Total CC | 647 | <647 |
| file Max CC | 348 | <348. preferred 212(`populateFromProps`) |
| file CC>25 개수 / 합 | 2 / 560 | preferred 1 / 212, 승인 예외 없으면 신규 초과 0 |
| `handleOk` CC / LOC | 348 / 381 | preferred <=25 / <=60 |
| global Total CC | 12,369 | same-base pre보다 순감소 |
| global CC>25 합 | 4,297 | same-base pre보다 순감소 |

upstream base가 달라지면 baseline 열을 최신 Stage 1 값으로 교체하되 target 판단 원칙은 바꾸지 않는다. Top 20,
CC>25/100 count·sum, Max와 모든 stable function diff를 함께 공개한다.

### 11.3 allowlist와 contract audit

- product diff는 `picture-props-dialog.ts`, `picture-props-apply-model.ts`로 제한
- test diff는 신규 model test와 `picture-props-undo.test.ts`로 제한
- DOM builder, CSS, `populateFromProps`, core/WASM/package files diff 0
- package/lock/runtime dependency delta 0
- target 외 stable function complexity delta 0 또는 명시적 설명
- mutation surface count 5 유지

### 11.4 문서와 GitHub 초안

- `mydocs/working/task_m100_2392_stage4.md`
- `mydocs/report/task_m100_2392_report.md`
- `mydocs/report/task_m100_2392_pr_draft.md`
- `mydocs/orders/20260719.md` 진행 상태 갱신

Stage 4 commit: `Task #2392: 적용 파이프라인 검증 결과 정리`

PR body, review request, #2392/#2022 진행 댓글 초안을 작업지시자에게 제시한다. 승인 전에는 push, PR 생성,
댓글 게시와 issue checklist 편집을 수행하지 않는다.

## 12. commit 계획

| 순서 | commit | 포함 |
|------|--------|------|
| 1 | `Task #2392: 개체 속성 apply 분리 계획 확정` | orders, 수행 계획, 구현 계획 |
| 2 | `Task #2392: 개체 속성 apply 계약 기준선 고정` | investigation, Stage 1 report |
| 3 | `Task #2392: 개체 속성 patch 계산 책임 분리` | pure model patch, dialog capture, unit test, Stage 2 report |
| 4 | `Task #2392: 개체 속성 적용 라우팅 분리` | target resolver, dialog adapter, source guard, Stage 3 report |
| 5 | `Task #2392: 적용 파이프라인 검증 결과 정리` | Stage 4, final report, PR draft, orders |

각 commit은 독립 review·revert 가능해야 한다. Stage 2와 3 사이에 기능 변경 commit을 끼우지 않는다.
upstream conflict resolution은 별도 merge/rebase 결과로 명확히 보존하고 #2392 metric delta에서 제외한다.

## 13. failure와 예외 처리

| 상황 | 처리 |
|------|------|
| target dialog file을 수정하는 새 PR 발견 | 구현 중단, overlap과 우선순위를 작업지시자에게 보고 |
| #2394 merge로 global metrics 변경 | latest base pre 재생성, issue/PR 숫자 보정 초안 제시 |
| #2395 merge로 orders conflict | upstream rows와 #2392 row를 모두 보존해 해결, 내용 삭제 금지 |
| current stale control behavior가 결함으로 확인 | 이번 refactor에서는 parity 유지, 별도 후속 이슈 초안 작성 |
| patch fixture가 current behavior와 불일치 | source와 runtime을 재확인하고 characterization 승인 전 구현 금지 |
| 신규 helper CC>25 | group 재분리. 불가피하면 function id/사유/해소 anchor reviewer 승인 |
| global Total CC 또는 CC>25 합 증가 | 이동으로 간주하고 Stage 완료 금지 |
| headless E2E 환경 실패 | server/Chrome/CDP 문제와 product failure 분리, 필수 시 재실행 |
| UI/DOM/CSS diff 필요 | scope drift로 중단하고 별도 승인 또는 이슈 분리 |
| public/WASM type 변경 필요 | #2392 범위 외로 분리, 이번 PR에 포함 금지 |

## 14. 구현 승인 반영 후 첫 실행 순서

1. latest `upstream/devel`과 열린 PR files를 재확인한다.
2. drift가 없으면 계획 문서 commit 1을 생성한다.
3. drift가 있으면 통합 방식과 orders conflict 초안을 작업지시자에게 먼저 제시한다.
4. Stage 1 pre metrics, characterization과 baseline gate를 실행한다.
5. Stage 1 문서와 결과를 제시하고 다음 Stage 승인을 기다린다.

이 구현 계획 승인만으로 전체 Stage를 자동 진행하지 않는다. 각 Stage 결과와 정량 gate를 검토받은 뒤 다음
단계로 넘어간다.
