# Task #1280 구현 계획서

## 개요

`enterTextboxPlacementMode()`가 도형 타입을 `'rectangle'`로 하드코딩하여, 삽입한 글상자가
text_box 없는 Rectangle로 생성되는 버그(#1280)를 수정한다. **수정은 프런트엔드 1줄(+주석)이 핵심**이며,
회귀를 막기 위해 e2e(실제 삽입 경로)와 Rust 단위(백엔드 계약) 테스트를 추가한다. 백엔드 로직은 변경하지 않는다.

마우스 플로우(검증 완료):
`enterTextboxPlacementMode()` → mousedown(`input-handler-mouse.ts:86`, `textboxPlacementDrag` 설정) →
mousemove(`:1131`, 드래그 갱신) → mouseup(`:1484`) → `finishTextboxPlacement()`(`input-handler.ts:816`) →
`createShapeControl({ shapeType: this.shapePlacementType })`. 즉 `shapePlacementType`만 `'textbox'`로
바꾸면 전체 경로가 정상화된다.

## 단계 구성 (3단계)

### 1단계 — 프런트엔드 핵심 수정

**파일**: `rhwp-studio/src/engine/input-handler.ts`

1. L513 `enterTextboxPlacementMode()`: `this.shapePlacementType = 'rectangle'` → `'textbox'`.
2. L150 `shapePlacementType` 필드 주석: 가능한 값에 `'textbox'`(및 실제 사용되는 `'polygon'`, `'arc'`,
   `'connector-*'`) 반영하여 실제 값과 정합.

수정 후 자동 정상화:
- L874 `if (this.shapePlacementType !== 'textbox')` → textbox는 종이 기준 offset 계산 스킵(offset=0) →
  백엔드가 Para/Column 기준·treat_as_char=true로 커서 위치에 인라인 배치.
- L915 `shapeType: 'textbox'` 전달 → 백엔드 text_box 정상 구성.
- L922 `selType`: textbox는 `'line'`/`'connector-*'`가 아니므로 `'shape'` 분류(정상).

**검증**: `cd rhwp-studio && npx tsc --noEmit` (타입체크 통과).

**산출물**: `mydocs/working/task_m100_1280_stage1.md` + 소스 커밋(`Task #1280: 글상자 삽입 shapeType 'textbox' 수정`).

### 2단계 — Rust 단위 테스트 (백엔드 계약 고정)

**파일**: `src/document_core/commands/object_ops.rs` (`#[cfg(test)]`에 `issue_1280_textbox_creation_tests` 모듈 추가)

기존 `resize_clamp_tests`/`issue_1151_cell_picture_insert_tests`의 `make_test_core()`/`parse_idx()`
헬퍼 패턴을 재사용한다.

1. `create_textbox_has_textbox`: `create_shape_control_native(..., shape_type="textbox", ...)` 호출 →
   반환 JSON에서 (paraIdx, controlIdx) 파싱 → 해당 `Control::Shape`에 `get_textbox_from_shape(...).is_some()` 단언.
2. `insert_text_into_created_textbox`: 위 글상자에 `insert_text_in_cell_native(0, para, ctrl, 0, 0, 0, "테스트")`
   → 글상자 내부 첫 문단 텍스트가 `"테스트"`로 보존됨을 단언(수정 전 백엔드는 이미 정상이나 회귀 방지).
3. `create_rectangle_has_no_textbox`: `shape_type="rectangle"`은 `get_textbox_from_shape(...).is_none()` 단언
   (글상자/사각형 경로 분리 고정).

**검증**: `cargo test issue_1280 -- --nocapture` 통과, `cargo fmt --check`(신규 코드 한정).

**산출물**: `mydocs/working/task_m100_1280_stage2.md` + 소스 커밋(`Task #1280: 글상자 생성 백엔드 계약 회귀 테스트`).

### 3단계 — e2e 회귀 테스트 + 통합 검증

**파일**: `rhwp-studio/e2e/issue-1280-textbox-text-input.test.mjs` (신규)

`helpers.mjs`의 `runTest`/`createNewDocument`/`screenshot` 패턴 사용. `window.__inputHandler`로
**실제 삽입 경로**를 구동(WASM 직접 호출 아님 — 프런트 버그를 실제로 잡음):

1. 새 문서 생성 → `page.evaluate(() => window.__inputHandler.enterTextboxPlacementMode())`
2. 편집 영역 좌표로 마우스 드래그: `page.mouse.move(x1,y1); page.mouse.down(); page.mouse.move(x2,y2); page.mouse.up()`
   (메뉴/툴바 밖 좌표 사용 — `input-handler-mouse.ts:88`의 toolbar 가드 회피)
3. 생성된 글상자 (paraIdx, controlIdx) 확보(`finishTextboxPlacement` 결과 또는 IR 조회) →
   `window.__wasm.doc.insertTextInCell(...)`로 텍스트 입력 → **수정 전 throw(글상자 없음), 수정 후 성공** 검증
4. 글상자 내부 문단에 텍스트 보존 + (가능 시) 붙여넣기 round-trip 확인. 스크린샷 저장.
- 참고 기존 테스트: `shape-inline.test.mjs`(`__inputHandler`/`__wasm` 사용), `debug-textbox.test.mjs`,
  `textbox-picture-insert-1171.test.mjs`.

> 마우스 좌표 기반 드래그가 환경 편차로 불안정할 경우, 대안으로 `finishTextboxPlacement`에 도달하는
> 최소 이벤트 시퀀스를 직접 구성하되 **반드시 `enterTextboxPlacementMode()`를 거쳐** `shapePlacementType`
> 설정 버그를 타격하도록 한다(WASM `createShapeControl` 직접 호출은 이 버그를 못 잡으므로 금지).

**검증**:
```bash
docker compose --env-file .env.docker run --rm wasm   # e2e용 WASM 빌드
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &
node e2e/issue-1280-textbox-text-input.test.mjs
cargo test                                             # 전체 회귀 무손상 재확인
```

**산출물**: `mydocs/working/task_m100_1280_stage3.md` + 소스 커밋(`Task #1280: 글상자 삽입 텍스트 입력 e2e 회귀 테스트`).

## 단계 간 승인

각 단계 완료 시 단계별 완료보고서(`working/task_m100_1280_stage{N}.md`)를 해당 소스 커밋과 함께 커밋하고
승인을 요청한다. 승인 후 다음 단계로 진행한다.

## 최종 산출물

- 소스: `input-handler.ts`(수정), `object_ops.rs`(테스트 추가), `issue-1280-textbox-text-input.test.mjs`(신규)
- 문서: `plans/task_m100_1280.md`, `plans/task_m100_1280_impl.md`, `working/task_m100_1280_stage{1..3}.md`,
  `report/task_m100_1280_report.md`
- 통합: `origin`(Fork) push → `upstream` `devel`로 PR. merge·마일스톤·orders는 메인테이너 영역.

## 롤백 영향

프런트 1줄 변경이 핵심이라 영향 범위가 좁다. 일반 도형 삽입(`enterShapePlacementMode`)은 별도 경로라
무영향이며, 2단계 대조 테스트(`create_rectangle_has_no_textbox`)로 분리를 고정한다.
