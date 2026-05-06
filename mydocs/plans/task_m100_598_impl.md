# Task #598 구현 계획서 — 1차: 본문 각주 마커 hit test + 커서 이동 정합

## 이슈 정합

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **마일스톤**: M100 — v1.0.0 조판 엔진 체계화
- **작업 브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **선행 문서**:
  - 수행계획서: `mydocs/plans/task_m100_598.md`
  - Stage 1 완료보고서: `mydocs/working/task_m100_598_stage1.md`
- **구현 범위**: 이슈 #598 전체 중 1차 작업인 본문 각주 마커 hit test 및 좌우 커서 이동 단위 정합

## Stage 1 진단 요약

`samples/footnote-01.hwp` 1페이지에서 본문 각주 마커는 이미 SVG에 정상 렌더링된다.

```text
문단 0.3: text_len=47, controls=1, [0] 각주
  SVG: x=263.25 y=384.07 "1)"

문단 0.7: text_len=13, controls=1, [0] 각주
  SVG: x=212.92 y=702.25 "2)"
```

그러나 일반 hit test는 `RenderNodeType::FootnoteMarker` 를 수집하지 않으며, `navigable_text_len()` / `classify_navigable()` 도 `Control::Footnote` 를 본문 inline unit 으로 취급하지 않는다.

## 구현 원칙

1. 기존 각주 영역 API는 유지한다.
   - `hitTestFootnote`
   - `hitTestInFootnote`
   - `getPageFootnoteInfo`
   - `getCursorRectInFootnote`

2. 본문 마커 전용 API를 새로 추가한다.
   - 기존 `hitTestFootnote` 는 각주 영역 zone hit test 라는 의미가 이미 있으므로 재사용하지 않는다.

3. 1차 작업은 body source 각주를 우선 대상으로 한다.
   - 표 셀/글상자 내부 각주는 후속 확장 대상으로 남긴다.
   - 단, Rust 내부 데이터 구조는 후속 확장이 가능하도록 source 식별을 명확히 둔다.

4. 본문 마커의 앞/뒤 위치를 삭제 UX가 사용할 수 있게 안정화한다.
   - Delete/Backspace 삭제 자체는 2차 작업에서 구현한다.
   - 1차 작업에서는 커서가 마커 앞/뒤 위치로 이동하고 caret rect 가 일관되게 표시되는 것까지 처리한다.

## API 설계

### 신규 WASM API

```rust
#[wasm_bindgen(js_name = hitTestBodyFootnoteMarker)]
pub fn hit_test_body_footnote_marker(
    &self,
    page_num: u32,
    x: f64,
    y: f64,
) -> Result<String, JsValue>
```

반환 JSON:

```json
{
  "hit": true,
  "sectionIndex": 0,
  "paragraphIndex": 3,
  "controlIndex": 0,
  "footnoteNumber": 1,
  "footnoteIndex": 0,
  "bbox": { "x": 263.3, "y": 376.1, "w": 9.0, "h": 20.0 },
  "cursorRect": { "pageIndex": 0, "x": 272.3, "y": 376.1, "height": 20.0 }
}
```

miss:

```json
{ "hit": false }
```

필드 의미:

| 필드 | 의미 |
|------|------|
| `sectionIndex` | 본문 각주 마커가 속한 구역 |
| `paragraphIndex` | 본문 각주 마커가 속한 문단 |
| `controlIndex` | 실제 `para.controls` 내 `Control::Footnote` 인덱스 |
| `footnoteNumber` | 렌더링된 각주 번호 |
| `footnoteIndex` | 해당 페이지 `page.footnotes` 배열 내 인덱스 |
| `bbox` | 본문 마커 bbox |
| `cursorRect` | 본문 마커 오른쪽 기준 caret 후보. 실제 각주 영역 진입은 기존 `enterFootnoteMode()` 경로 사용 |

### WasmBridge 래퍼

```ts
hitTestBodyFootnoteMarker(
  pageNum: number,
  x: number,
  y: number,
): {
  hit: boolean;
  sectionIndex?: number;
  paragraphIndex?: number;
  controlIndex?: number;
  footnoteNumber?: number;
  footnoteIndex?: number;
  bbox?: { x: number; y: number; w: number; h: number };
  cursorRect?: { pageIndex: number; x: number; y: number; height: number };
}
```

## 구현 상세

### 1. 본문 각주 마커 메타데이터 정합

대상 파일:

- `src/renderer/composer.rs`
- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/render_tree.rs`

현재 `ComposedParagraph.footnote_positions` 는 `(position, number)` 형태다. `paragraph_layout.rs` 는 이 배열의 순번 `fni` 를 `FootnoteMarkerNode.control_index` 로 넣고 있다. 문단에 비각주 컨트롤이 섞이면 실제 control index 와 불일치할 수 있다.

수정 방향:

- `footnote_positions` 를 `(position, number, control_index)` 형태로 확장한다.
- `paragraph_layout.rs` 의 모든 `FootnoteMarkerNode.control_index` 를 실제 control index 로 채운다.
- 기존 번호/위치 기반 로직은 동일하게 유지한다.

검증:

- `samples/footnote-01.hwp` 의 문단 0.3, 0.7에서 `controlIndex=0` 반환
- 문단에 footnote 이전 비각주 컨트롤이 있는 synthetic 테스트 또는 단위 테스트를 추가할 수 있으면 실제 control index 보정 확인

### 2. Rust 본문 마커 hit test 추가

대상 파일:

- `src/document_core/queries/cursor_rect.rs`
- `src/wasm_api.rs`

구현 방향:

1. `build_page_tree_cached(page_num)` 또는 기존 page tree 경로를 사용한다.
2. 렌더 트리를 순회해 `RenderNodeType::FootnoteMarker` 를 수집한다.
3. `x/y` 가 marker bbox 안에 들어오면 hit 처리한다.
4. 해당 marker 의 `section_index`, `para_index`, `control_index` 와 현재 페이지 `page.footnotes` 의 `FootnoteSource::Body` 를 대조해 `footnoteIndex` 를 찾는다.
5. body source 가 아닌 경우 1차 작업에서는 miss 또는 `sourceType` 포함 후 TS에서 무시하는 방식 중 하나로 제한한다. 구현은 body source hit 만 `hit=true` 로 반환하는 쪽을 기본값으로 한다.

추가 helper 후보:

```rust
fn find_page_body_footnote_index(
    &self,
    page_num: u32,
    section_idx: usize,
    para_idx: usize,
    control_idx: usize,
) -> Option<usize>
```

### 3. 본문 각주 마커 cursor unit 반영

대상 파일:

- `src/document_core/helpers.rs`
- `src/document_core/queries/doc_tree_nav.rs`
- `src/document_core/queries/cursor_rect.rs`

수정 방향:

- `Control::Footnote` 를 본문 inline control 1칸으로 취급한다.
- `navigable_text_len()` 에서 Footnote 위치 뒤 offset 을 허용한다.
- `classify_navigable()` 은 `Control::Footnote(_) => Some(false)` 로 분류한다.
- `navigate_next_editable()` 은 기존 Shape/Picture/Equation 경로와 같은 방식으로 footnote 위치에서 한 번 멈추고, 다음 이동에서 마커 뒤 위치로 이동하도록 한다.
- `get_cursor_rect_native()` 는 `FootnoteMarker` 에 대해 다음을 처리한다.
  - offset == marker logical position: marker 왼쪽 caret
  - offset == marker logical position + 1: marker 오른쪽 caret

주의점:

- 현 구조는 텍스트 offset 과 inline control logical offset 이 완전히 분리되어 있지 않다. 따라서 구현 시 `find_control_text_positions()` 를 그대로 사용하되, Footnote가 있는 문단에 한해 marker 앞/뒤 caret rect 를 우선 반환하도록 한다.
- 텍스트 삽입/삭제의 logical offset 전환은 2차 삭제 API/UI 작업에서 별도 검토한다. 1차 작업은 방향키 및 hit test 안정화에 집중한다.

### 4. rhwp-studio 마우스 처리 연결

대상 파일:

- `rhwp-studio/src/core/wasm-bridge.ts`
- `rhwp-studio/src/engine/input-handler-mouse.ts`

수정 방향:

1. `WasmBridge.hitTestBodyFootnoteMarker()` 추가
2. `input-handler-mouse.ts` 에서 일반 `wasm.hitTest()` 보다 먼저 본문 각주 마커 hit 를 검사한다.
3. hit 시:
   - `pageInfo` 를 별도 조회하지 않고 hit 결과의 `sectionIndex/paragraphIndex/controlIndex/footnoteIndex` 를 사용한다.
   - `cursor.enterFootnoteMode(section, para, control, footnoteIndex, pageIdx)` 호출
   - `cursor.setFnCursorPosition(0, 0)` 또는 기존 `getCursorRectInFootnote` 기본 위치 사용
   - `footnoteModeChanged` 이벤트 emit
   - caret 갱신 후 return

기존 각주 영역 클릭 처리 순서는 유지한다.

```text
1. 각주 편집 모드에서 클릭 처리
2. 본문 각주 마커 hit test
3. 각주 영역 클릭 → 각주 편집 모드 진입
4. 일반 본문 hitTest
```

### 5. 검증 및 e2e

대상 파일 후보:

- `src/wasm_api/tests.rs`
- `rhwp-studio/e2e/footnote-marker-nav.test.mjs` 신규

검증 항목:

1. Rust/WASM 단위 또는 통합 검증
   - `samples/footnote-01.hwp` 로드
   - 페이지 0 marker hit API가 문단 0.3 / control 0 / footnoteIndex 0을 반환하는지 확인
   - 페이지 0 두 번째 marker가 문단 0.7 / control 0 / footnoteIndex 1을 반환하는지 확인

2. 커서 이동 검증
   - 문단 0.3의 footnote 위치 앞에서 `navigateNextEditable(..., +1)` 호출
   - 다음 위치가 marker 뒤 위치로 진행되는지 확인
   - 반대 방향도 동일 확인

3. rhwp-studio e2e
   - `footnote-01.hwp` 로드
   - 본문 각주 마커 클릭 좌표를 API 또는 SVG 좌표 기반으로 계산
   - 클릭 후 `cursor.isInFootnote()` 또는 UI 상태 이벤트가 true 인지 확인
   - 좌우 방향키 이동 시 marker 앞/뒤 caret x 좌표가 marker bbox 좌/우로 이동하는지 확인

4. 회귀 검증
   - `cargo test`
   - `cargo build`
   - `cd rhwp-studio && npm run build`
   - 가능하면 `cd rhwp-studio && node e2e/footnote-marker-nav.test.mjs --mode=headless`

## 구현 단계 (4 stages)

### Stage 3-1 — Rust marker metadata + hit API

- `footnote_positions` 를 실제 control index 포함 형태로 확장
- `FootnoteMarkerNode.control_index` 정합 보정
- `hit_test_body_footnote_marker_native()` 추가
- `wasm_api.rs` export 추가

완료 기준:

- cargo build 통과
- 신규 API가 `samples/footnote-01.hwp` 의 본문 각주 마커 source 를 정확히 반환

### Stage 3-2 — Cursor navigation / rect 보정

- `Control::Footnote` 를 navigable inline control 로 분류
- footnote marker 앞/뒤 caret rect 계산 추가
- `navigateNextEditable` 좌우 이동에서 각주 마커를 1칸 단위로 처리

완료 기준:

- ArrowLeft/ArrowRight 기반으로 문단 0.3, 0.7 각주 마커 앞/뒤 위치 도달
- 기존 Shape/Picture/Equation navigation 회귀 없음

### Stage 3-3 — WasmBridge + mouse 연결

- `WasmBridge.hitTestBodyFootnoteMarker()` 추가
- `input-handler-mouse.ts` 본문 마커 클릭 분기 추가
- 본문 마커 클릭 시 각주 영역 편집 모드 진입

완료 기준:

- 본문 각주 마커 클릭 후 각주 영역 cursor 표시
- 기존 각주 영역 직접 클릭 진입 동작 유지

### Stage 3-4 — 테스트 및 완료보고

- Rust 테스트 또는 wasm_api 테스트 추가
- rhwp-studio e2e 추가
- `cargo test`, `cargo build`, `npm run build`, e2e 실행
- Stage 3 완료보고서 작성

완료 기준:

- 검증 게이트 통과
- 남은 이슈는 2차 삭제 API/UI 작업 범위로 정리

## 위험 및 대응

| 위험 | 대응 |
|------|------|
| `control_index` 정합 변경으로 렌더링 회귀 | `footnote_positions` 확장은 marker 메타만 바꾸고 번호/위치 렌더링은 유지 |
| offset 체계 혼동 | 1차는 marker 앞/뒤 caret rect 와 navigation 에 집중하고, 텍스트 편집 변환은 2차에서 명시적으로 다룸 |
| body 외 source 각주 혼입 | hit API는 `FootnoteSource::Body` 만 true 반환 |
| 기존 각주 영역 클릭 회귀 | 기존 `hitTestFootnote` / `hitTestInFootnote` 경로는 수정하지 않음 |
| e2e 좌표 불안정 | marker hit API 또는 bbox 반환값을 사용해 클릭 좌표 계산 |

## 최종 검증 명령

```bash
cargo test
cargo build
cd rhwp-studio && npm run build
cd rhwp-studio && node e2e/footnote-marker-nav.test.mjs --mode=headless
```

headless Chrome 경로 또는 환경 문제로 e2e 실행이 불가능하면, 실패 원인을 Stage 3 완료보고서에 기록하고 WASM API 직접 검증 결과를 함께 제출한다.

## 다음 승인 지점

본 구현 계획서 승인 후 Stage 3-1 구현을 시작한다.
