# Task M100 #2392 Stage 2 완료 보고 - 개체 속성 patch 계산 책임 분리

- 이슈: #2392
- 상위 추적: #2022
- 브랜치: `issue-2392-picture-props-apply-pipeline`
- source 기준: `upstream/devel@af5902b659be9a4d86ad458d79c63353dba88167`
- 선행 계획 commit: `293c764020488963113630994689c3dac3614860`
- 선행 Stage 1 commit: `8773d6b4310eb4057f13bd8d7af60c8e3c701831`
- 작성일: 2026-07-19

## 1. 완료 요약

`PicturePropsDialog.handleOk`의 DOM 읽기·변환·diff 정책 중 patch 계산 책임을 pure model로 분리했다.

- `picture-props-apply-model.ts`에 grouped raw snapshot type과 pure patch builder를 추가했다.
- dialog에는 DOM element가 아닌 raw 값만 수집하는 `captureApplyForm`을 추가했다.
- `handleOk`는 null guard, snapshot, builder 호출 뒤 기존 setter/undo block을 그대로 조정한다.
- apply 전용 mm/HWPUNIT, hex/ColorRef, caption grid 변환은 pure model로 이동했다.
- image/shape/line/group/OLE의 changed-only·always-send·fallback·대입 순서를 20개 fixture로 고정했다.
- target resolver와 setter/undo method 분리는 계획대로 Stage 3에 남겼다.

## 2. 변경 파일

| 파일 | 변경 |
|------|------|
| `rhwp-studio/src/ui/picture-props-apply-model.ts` | 10개 책임군 snapshot type, 변환 helper, common/shape·OLE/image patch builder 신설 |
| `rhwp-studio/src/ui/picture-props-dialog.ts` | `captureApplyForm` 추가, 기존 patch 계산 block을 builder 호출로 교체 |
| `rhwp-studio/tests/picture-props-apply-model.test.ts` | data-driven patch fixture 20개 신설 |
| `mydocs/working/task_m100_2392_stage2.md` | 본 Stage 관문 결과 |
| `mydocs/orders/20260719.md` | Stage 3 승인 대기 상태 반영 |
| `mydocs/plans/task_m100_2392*.md` | 최신 upstream·rebase provenance와 Stage 상태 보정 |
| `mydocs/working/task_m100_2392_stage1.md` | 최신 base·commit·pre snapshot hash 보정 |

`core/types.ts`, `wasm-bridge.ts`, InputHandler·command type, package/lock, Rust/WASM/generated pkg,
CSS·DOM builder, extension·VS Code·npm/editor source는 변경하지 않았다.

## 3. 책임 경계

### 3.1 pure model

모델의 runtime import는 0이며 `PictureProperties`, `ShapeProperties`만 `import type`으로 참조한다. package
entry나 barrel에서 재-export하지 않아 public API는 늘리지 않았다.

snapshot은 `common`, `transform`, `outerMargin`, `caption`, `line`, `shapeTextBox`, `shapeCorner`,
`shapeFill`, `shapeShadow`, `image`로 묶었다. HTML element, dialog instance, WASM bridge, event bus와 command
service는 snapshot 또는 builder에 들어가지 않는다.

builder는 다음 기존 의미를 유지한다.

1. common size·position·description을 먼저 계산한다.
2. shape/OLE 또는 image 전용 field를 이어서 계산한다.
3. image scale은 활성 시 앞서 계산한 common width/height를 덮어쓴다.
4. caption·fill·shadow의 always-send와 나머지 changed-only 정책을 구분한다.
5. `parseInt`/`parseFloat`의 `||` fallback, transparency clamp, alpha rounding, `Original → RealPic`,
   `TakePlace → TopAndBottom`을 바꾸지 않는다.

### 3.2 dialog와 side effect

`captureApplyForm`은 현재 control reference의 raw 값과 presence를 기록한다. singleton을 shape/group 뒤 line으로
재사용할 때 detached textbox/fill reference가 남을 수 있는 현재 동작도 초기화하지 않고 그대로 snapshot한다.

Stage 2에서는 `handleOk` 안의 다음 block을 이동하지 않았다.

- shape cell/body와 image header-footer/cell/body의 WASM setter 5회
- image의 header/footer 우선순위와 setter 인자 순서
- `kind: 'snapshot'`, `operationType: 'objectProps'` undo
- services 미주입 fallback의 `document-changed`
- empty patch 무적용과 마지막 `hide()`

따라서 기존 `picture-props-undo.test.ts` source guard도 이번 Stage에서는 수정하지 않았다.

## 4. fixture 범위

focused test 20개는 다음 계약을 직접 검증한다.

| 책임 | 검증 |
|------|------|
| common | 방어적 empty snapshot, size·position 변환, size lock, treat-as-char, TakePlace |
| transform | disabled 보존, enabled rotation/flip changed-only |
| OLE | outer margin, caption, line만 저장하고 arrow/fill/shadow 제외 |
| line/shape/group | 누락 textbox의 0/Top 정규화, arrow, corner, solid/gradient fill, alpha, shadow |
| always-send | image caption false, shape shadow type 0과 offset 0 |
| stale reference | 재사용 line snapshot이 detached textbox/fill 값을 현재처럼 유지 |
| image | scale precedence, crop/padding, border, effect, brightness/contrast, transparency clamp |

stale reference 자체가 의도된 제품 계약이라는 뜻은 아니다. #2392는 리팩터링 전후 parity만 유지하며,
초기화 정책 수정은 별도 기능 이슈 후보로 남긴다.

## 5. metrics

동일 pre snapshot과 비교한 결과다.

| 지표 | pre | Stage 2 | delta |
|------|----:|--------:|------:|
| included files | 215 | 216 | +1 |
| reported functions | 2,386 | 2,405 | +19 |
| Total CC | 12,369 | 12,095 | -274 |
| Top 20 합 | 2,660 | 2,359 | -301 |
| CC>25 개수 | 70 | 69 | -1 |
| CC>25 합 | 4,297 | 3,949 | -348 |
| CC>100 개수 | 7 | 6 | -1 |
| Max CC | 453 | 453 | 0 |

대상 경계 상세:

| 항목 | pre dialog | Stage 2 dialog | 새 pure model |
|------|-----------:|---------------:|----------------:|
| physical/code LOC | 2,825 / 2,562 | 2,599 / 2,363 | 458 / 420 |
| reported functions | 35 | 35 | 19 |
| Total CC | 647 | 298 | 75 |
| CC>25 개수 / 합 | 2 / 560 | 1 / 212 | 0 / 0 |
| Max CC | 348 | 212 | 13 |
| `handleOk` CC / LOC | 348 / 381 | 5 / 59 | - |

새 helper의 최대는 `captionFromGrid` CC 13이며 CC>25 helper는 0이다. dialog와 model을 합친 Total CC도
373으로 기존 dialog 647보다 274 감소했다. `populateFromProps` CC 212는 범위 제외라 변하지 않았다.

- pre metrics SHA-256: `02ab67076683a091b1c77f1c9c9889867af42f100dc7fc6ef6092485a59f5a93`
- Stage 2 metrics SHA-256: `d8657044e924dda623054a2feaaed5e16100b21f27f4441a387c0fbf4f6a929c`
- metrics script SHA-256: `5d100c90f47671240f463b0a48fe61d34eb8aedbf8c22bbe333f31241f11d087`
- metrics lock SHA-256: `a7ae3c1a0f3c94700cfe29dc9c363657cb1f675c988446d5dc81b7eeecace5dd`
- measured source clean: false. 구현 중간 snapshot이므로 dialog와 새 model이 dirty path로 정확히 기록됨

metrics output은 ignore된 `output/frontend-metrics/task2392/stage2/`에 두고 commit하지 않는다. Stage 4 final
snapshot에서는 clean source provenance를 다시 고정한다.

## 6. 검증

| Gate | 결과 |
|------|------|
| `node --test rhwp-studio/tests/picture-props-apply-model.test.ts` | PASS, 20/20 |
| `npm --prefix rhwp-studio test` | PASS, 382/382 |
| `npm --prefix rhwp-studio run build` | PASS |
| frontend metrics pre 비교 | PASS, Total CC·CC>25 합 순감소 |
| mutation routing 원장 | PASS, dialog setter call baseline 5 유지 |
| 금지 경로 diff | PASS, core/WASM/package/extension 0 |
| `git diff --check` | PASS |

build에는 기존 CanvasKit `fs`/`path` browser externalize와 500 kB chunk 경고가 있었으나 실패는 없었다.
DOM/CSS와 렌더링 결과를 바꾸지 않는 Stage라 visual/E2E는 계획대로 Stage 4에서 실행한다.

Stage 종료 직전 `git fetch upstream` 결과 `upstream/devel@af5902b6`은 변하지 않았다. #2370은 `insert.ts`
1개 파일, #2394는 core/renderer 중심으로 OPEN이며 둘 다 대상 dialog를 수정하지 않는다. #2394가 merge되면
global metrics 모집단이 바뀔 수 있으므로 다음 Stage 시작 전에 다시 확인한다.

## 7. Stage 3 관문

- [x] pure model에 runtime service·DOM 의존 없음
- [x] patch fixture 20/20 PASS
- [x] 기존 Studio suite와 build PASS
- [x] `handleOk` CC/LOC 감소
- [x] 신규 helper CC>25 0
- [x] global/file Total CC와 CC>25 합 순감소
- [x] mutation setter 5회와 undo/fallback block 유지
- [x] 범위 밖 stale-control 결함 후보를 기능 수정하지 않음
- [x] 작업지시자의 Stage 3 승인

Stage 3 결과는 `mydocs/working/task_m100_2392_stage3.md`에서 이어서 추적한다.
