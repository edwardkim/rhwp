# Task #598 Stage 3-1 완료보고서 — Rust marker metadata + hit API

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **단계 범위**: 본문 각주 마커 메타데이터 정합 및 Rust/WASM hit test API 추가

본 단계에서는 구현 계획서의 Stage 3-1 범위만 처리했다. 커서 좌우 이동 단위 보정, rhwp-studio 마우스 연결, 삭제 API/UI는 다음 단계로 남겨두었다.

## 변경 파일

| 파일 | 변경 내용 |
|------|-----------|
| `src/renderer/composer.rs` | `ComposedParagraph.footnote_positions` 를 `(position, number, control_index)` 형태로 확장 |
| `src/renderer/layout/paragraph_layout.rs` | `FootnoteMarkerNode.control_index` 에 배열 순번이 아니라 실제 `para.controls` 인덱스를 전달 |
| `src/document_core/queries/cursor_rect.rs` | `hit_test_body_footnote_marker_native()` 및 페이지 각주 source 대조 helper 추가 |
| `src/wasm_api.rs` | `hitTestBodyFootnoteMarker(pageNum, x, y)` WASM export 추가 |

## 구현 내용

### 1. 각주 마커 control index 보정

기존에는 `footnote_positions` 배열 내 순번이 `FootnoteMarkerNode.control_index` 로 전달됐다.

이제 `compose_paragraph()` 단계에서 `para.controls` 의 실제 인덱스를 함께 보존하고, 렌더링 단계에서 해당 값을 그대로 `FootnoteMarkerNode.control_index` 로 전달한다.

이 변경으로 문단 안에 각주보다 앞선 그림/표/수식 등 다른 컨트롤이 있어도 본문 마커 hit 결과가 실제 `Control::Footnote` 인덱스를 가리킬 수 있다.

### 2. 본문 각주 마커 hit test API 추가

`DocumentCore::hit_test_body_footnote_marker_native(page_num, x, y)` 를 추가했다.

처리 흐름은 다음과 같다.

1. `build_page_tree_cached(page_num)` 으로 페이지 렌더 트리를 얻는다.
2. 트리를 재귀 순회해 `RenderNodeType::FootnoteMarker` bbox 를 찾는다.
3. hit 된 마커의 `section_index`, `para_index`, `control_index` 를 현재 페이지의 `page.footnotes` 와 대조한다.
4. `FootnoteSource::Body` 와 일치하는 항목만 `hit=true` 로 반환한다.
5. 표 셀/글상자 등 body source 가 아닌 항목은 1차 범위 밖이므로 `hit=false` 로 처리한다.

반환 JSON 형태:

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

### 3. WASM export 추가

`src/wasm_api.rs` 에 `hitTestBodyFootnoteMarker` 를 추가했다. rhwp-studio 연결은 Stage 3-3에서 `WasmBridge` 래퍼와 마우스 입력 처리 순서에 반영한다.

## 검증

실행 결과:

```bash
cargo build
git diff --check
```

결과:

- `cargo build` 통과
- `git diff --check` 통과

참고:

- 전역 `cargo fmt --check` 는 기존 저장소의 다수 파일 포맷 차이를 함께 보고하므로 이번 단계 검증 기준에서 제외했다.
- 포맷 명령이 하위 모듈까지 따라가며 불필요한 포맷 변경을 만든 부분은 되돌렸고, 최종 diff 는 Stage 3-1 대상 4개 파일로 제한했다.

## 남은 작업

다음 승인을 받은 뒤 Stage 3-2에서 진행한다.

1. `Control::Footnote` 를 본문 navigable inline unit 으로 분류
2. 각주 마커 앞/뒤 offset 을 좌우 커서 이동에 반영
3. `get_cursor_rect_native()` 에서 `FootnoteMarker` bbox 기반 caret rect 반환
