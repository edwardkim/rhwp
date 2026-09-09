# Task #1280 (v2) 구현 계획서 — 글상자 위 이미지 배치(plane)/클릭 우선순위

## 개요

원 #1280(stage1~4 완료)은 삽입 글상자가 `text_box` 없는 Rectangle로 생성되던 결함을
`shapeType:'textbox'`로 고쳐 텍스트 입력/붙여넣기를 복구했고, 그 과정에서 글상자를
**인라인(treat_as_char=true, offset=0)** 으로 배치하도록 정했다. 이 결정이 후속 결함을 낳았다:
글상자 위에 이미지를 삽입하면 (1) **렌더링** — 이미지가 글상자 *위*로 그려지고(한컴은 *뒤*),
(2) **히트테스트** — 보이는 이미지를 클릭해도 밑의 글상자가 선택된다(WYSIWYG 불일치).

한컴 권위 샘플 `samples/textbox-under-image.hwp` 실측: 글상자=**InFrontOfText·floating·z=0**,
이미지=**Square·z=1**. 원인은 z-order가 아니라 **plane** — 글상자가 `InFrontOfText`(plane 3)라
어울림 이미지(plane 2) 위에 그려진다. 렌더 정렬키 `(plane, z_order, stable_index)`에서 plane이
상위라 이긴다. rhwp-studio 글상자는 정반대(plane 2 + 인라인)라 이미지에 깔린다.

**작업지시자 확정 방향(수행계획서): 한컴 동일(이미지가 글상자 뒤로).** 두 축으로 수정한다.
- **축 A — 히트테스트 "클릭 = 최상단 개체 선택"** (배치와 무관하게 공통 필요).
- **축 B — 글상자 삽입 기본값을 한컴 정답값(`InFrontOfText` + `floating`)으로 교정**
  (#1280의 인라인 결정을 floating으로 되돌림).

## 현 상태 검증 (코드 확증, file:line)

- 렌더 정렬키 — `src/renderer/layout.rs:744-782`: `render_layer_plane`(BehindText→1,
  InFrontOfText→3, 그 외→2), `paper_node_sort_key`=`(plane, z_order, stable_index)`.
  `RenderNode.layer`(`RenderLayerInfo`: text_wrap/z_order/stable_index)와 `.id` 보유. **재사용**.
- 레이아웃 쿼리 — `src/document_core/queries/rendering.rs:1504-1819` `get_page_control_layout_native`:
  render tree 순회로 컨트롤 JSON을 `format!()` 수동 빌드. 노출: type/x/y/w/h/secIdx/paraIdx/
  controlIdx/cell정보/cellPath, `wrap`은 **이미지만**. `zOrder/plane/stableIndex` **미노출**.
  WASM 경계: `src/wasm_api.rs:613-617` `getPageControlLayout`.
- 프런트 타입 — `rhwp-studio/src/core/types.ts:520-537` `ControlLayoutItem`:
  zOrder/plane/stableIndex 없음. `wrap`/`cellPath`는 런타임 동적 접근(미선언).
- 히트테스트 — `rhwp-studio/src/engine/input-handler-picture.ts`: Pass 0(#1171 중첩 picture 우선,
  L73-95), **Pass 1(첫 bbox 적중 반환, L96-163, 반환 L158-161)**, Pass 2(#516 BehindText 폴백,
  L164-187). `layout.controls`는 WASM emit 순서(z-order 미지원, L75/L866 주석).
- 글상자 삽입 백엔드 — `src/document_core/commands/object_ops.rs`: `== "textbox"` 게이트
  L3676(attr=**0x0A0210**), L3767(margin 283), L3783(vertRel=Para), L3789(horzRel=Column),
  L3811(has_textbox). attr bit0=treat_as_char, bit21-23=text_wrap(3=InFrontOfText) —
  `src/document_core/converters/common_obj_attr_writer.rs:82-111`.
- WASM 기본값 — `src/wasm_api.rs:2855-2890`: `default_tac = (shape_type=="textbox")`(true),
  `textWrap` 기본 `"Square"`. 프런트는 treatAsChar/textWrap **미전달**.
- 프런트 삽입 offset — `rhwp-studio/src/engine/input-handler.ts:818-939` `finishTextboxPlacement`:
  L876 `if (this.shapePlacementType !== 'textbox')` 안에 **paper-relative offset 계산이 이미 존재**
  (사각형용). textbox는 스킵 → offset=0(인라인). createShapeControl(L909-920)은 treatAsChar/textWrap 미전달.
- 선택 ref 소비처 **30+곳** (insert.ts/format.ts/cursor.ts/input-handler{,-mouse,-keyboard,-table}.ts) —
  delete/cut/copy/properties/resize/move/rotate/context-menu/multi-select. 메모리 룰
  `audit-selection-ref-consumers`(PR #1254 교훈) 적용 대상.
- 권위 샘플 `samples/textbox-under-image.hwp` 존재(확인).

## 단계 구성 (5단계 — 축 A 먼저[공통·저위험], 축 B 마지막[#1280 인라인 결정 되돌림·고위험])

### Stage 1 — Rust 레이아웃 쿼리에 `plane/zOrder/stableIndex` 노출 (+ wrap을 shape에도)

**파일**: `src/document_core/queries/rendering.rs` `get_page_control_layout_native`.

1. 각 컨트롤 JSON에 `plane`, `zOrder`, `stableIndex` 추가. **렌더 정렬키와 동일 의미**가 되도록
   `src/renderer/layout.rs:744-782`의 `render_layer_plane`/`paper_node_sort_key` 로직을 재사용
   (또는 동일 산출 보장). `RenderNode.layer`(z_order/text_wrap/stable_index)·`.id`에서 값을 취한다.
2. `wrap`을 이미지뿐 아니라 shape/group/line 컨트롤에도 방출(히트테스트 plane 비교 일관성).

**Rust 단위**: 같은 페이지 컨트롤들의 `(plane, zOrder, stableIndex)`가 `paper_node_sort_key`
순서와 정합함을 단언(권위 샘플 또는 합성 문서 사용).

**검증**: `cargo test`. **산출물**: `working/task_m100_1280_v2_stage1.md` + 소스 커밋.

### Stage 2 — 프런트 `ControlLayoutItem` 필드 확장 (동작 무변화)

**파일**: `rhwp-studio/src/core/types.ts:520-537`.

`plane?: number`, `zOrder?: number`, `stableIndex?: number`, `wrap?: string` 선언(현 동적 접근 정식화).
동작 변화 없음 — 타입 정합만.

**검증**: `cd rhwp-studio && npx tsc --noEmit`, `npm test`.
**산출물**: `working/task_m100_1280_v2_stage2.md` + 소스 커밋.

### Stage 3 — `findPictureAtClick` Pass 1을 "최상단 적중" 선택으로 교체

**파일**: `rhwp-studio/src/engine/input-handler-picture.ts` Pass 1(L96-163).

첫-적중-반환을, 적중 후보를 모아 `(plane, zOrder, stableIndex)` **최대값**(최상단) 선택으로 교체한다.
**Pass 0(#1171 중첩 picture)·Pass 2(#516 BehindText)는 그대로 보존** — 회귀 금지.

**신규 e2e**: `rhwp-studio/e2e/topmost-hittest.test.mjs` — 겹친 두 개체에서 위 개체가 선택됨을
검증(권위 샘플 또는 합성 셋업). `helpers.mjs`/`window.__inputHandler` 패턴 사용.

**검증**: `docker compose --env-file .env.docker run --rm wasm` → `node e2e/topmost-hittest.test.mjs`.
**산출물**: `working/task_m100_1280_v2_stage3.md` + 소스 커밋.

### Stage 4 — 정합 회귀 e2e 확장 + 선택 ref 소비처 lifecycle 검증 (메모리 룰)

**파일**: e2e 확장 + `#1171`(`textbox-picture-1171.test.mjs`)·`#516`·line·다중구역 회귀.

**메모리 룰 `audit-selection-ref-consumers` 적용**: 히트테스트가 겹침에서 *다른 개체*(최상단)를
선택하게 되므로,
1. `getSelectedPictureRef`/`getSelectedPictureRefs` **소비처 전수 grep 감사**(30+곳)로 반환 ref가
   완전(sec/ppi/ci/cellPath/type)함을 확인.
2. **"선택(최상단)→삭제 / 리사이즈 / 오려두기" lifecycle e2e** 추가(happy-path만 테스트 금지).

**검증**: 신규+기존 e2e 통과, `cargo test` 전체 무손상.
**산출물**: `working/task_m100_1280_v2_stage4.md` + 소스 커밋.

### Stage 5 — 글상자 삽입 기본값 `floating + InFrontOfText` 교정 (#1280 인라인 결정 되돌림)

**백엔드 정합**: floating textbox의 정확한 `vertRel/horzRel`/attr 값을
`rhwp dump samples/textbox-under-image.hwp` 실측으로 확정한다(인라인 0x0A0210 → floating 값).
`src/document_core/commands/object_ops.rs`에서 textbox일 때 attr 비트가 정합
(treat_as_char=0, bit21-23=3=InFrontOfText)하도록 확인/보정.

**프런트**: `rhwp-studio/src/engine/input-handler.ts:876-897` — textbox도 paper-relative offset
계산을 타도록(기존 사각형 코드 재사용) 하고, createShapeControl에 `treatAsChar:false` +
`textWrap:"InFrontOfText"`를 명시 전달.

**회귀 위험 완화**: 기존 HWP/HWPX 글상자는 이미 floating이고 텍스트 입력이 정상 동작 → 삽입 글상자를
로드된 글상자와 정합시키는 방향이라 #1280 텍스트 입력 회귀 위험이 낮다. 단, **#1280 텍스트 입력/
붙여넣기 회귀 e2e 재실행은 필수**.

**검증**: `cargo test`(전체) + `npx tsc --noEmit` + e2e(`issue-1280-textbox-text-input.test.mjs`
회귀, `topmost-hittest`) + HWP5 round-trip 무손상.
**산출물**: `working/task_m100_1280_v2_stage5.md` + 소스 커밋.

## 검증 계획 (전체)

```bash
cargo test                                            # Stage 1,5 Rust 단위 + 전체 회귀
cd rhwp-studio && npx tsc --noEmit && npm test        # Stage 2 타입체크/단위
docker compose --env-file .env.docker run --rm wasm   # e2e용 WASM 빌드
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &
node e2e/topmost-hittest.test.mjs                     # 신규 (Stage 3/4)
node e2e/issue-1280-textbox-text-input.test.mjs       # #1280 회귀 (Stage 5)
node e2e/textbox-picture-1171.test.mjs                # #1171 회귀
```

최종 시각 판정: **한컴 편집기(Windows)** / `samples/textbox-under-image.hwp` 와 렌더 스택 +
겹침 클릭 우선순위 직접 대조.

## 완료 기준

1. 글상자 위 이미지 삽입 시 **이미지가 글상자 뒤**(한컴 동일), 겹침 클릭 시 **위 개체(글상자) 선택**.
2. 삽입 글상자가 `InFrontOfText` + `floating(treat_as_char=false)`로 생성, attr 비트·
   `common.text_wrap` 정합(HWP5 round-trip 무손상).
3. 히트테스트가 겹침에서 `(plane, zOrder, stableIndex)` 최상단 선택, **선택 후 삭제/리사이즈/
   오려두기 lifecycle 정상**(메모리 룰).
4. #1280 본편(텍스트 입력/붙여넣기), #1171/#516 회귀 없음.
5. 신규 e2e + Rust 단위 통과.

## 범위 밖 (별도 이슈)

- **글상자 안 이미지 붙여넣기** 무음 실패(`merge_from` 컨트롤 누락) — #1280 stage4에서 별개 결함으로
  분리됨. 본 v2 비대상.

## 단계 간 승인 / 권한 메모

- 각 단계 완료 시 단계별 완료보고서(`working/task_m100_1280_v2_stage{N}.md`)를 해당 소스 커밋과
  함께 커밋하고 승인을 요청한다. 승인 후 다음 단계 진행.
- `mydocs/orders/`(오늘 할일)·GitHub 이슈 마일스톤·이슈 클로즈는 **메인테이너 영역**(에이전트 미수정).
- 통합: Fork `origin` push → upstream `devel` PR (메인테이너 merge).
