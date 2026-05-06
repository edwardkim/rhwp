# Task #598 Stage 1 완료보고서 — 본문 각주 마커 hit test + 커서 이동 진단

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **기준 커밋**: `upstream/devel` `9b49063`
- **진단 범위**: 본문 각주 마커 hit test 및 좌우 커서 이동 단위
- **대표 샘플**: `samples/footnote-01.hwp`

본 단계에서는 소스 수정 없이 재현 명령과 코드 대조만 수행했다.

## 실행 명령

```bash
cargo run --bin rhwp -- dump-pages samples/footnote-01.hwp -p 0
cargo run --bin rhwp -- dump samples/footnote-01.hwp
target/debug/rhwp dump samples/footnote-01.hwp -s 0 -p 3
target/debug/rhwp dump samples/footnote-01.hwp -s 0 -p 7
target/debug/rhwp dump-pages samples/footnote-01.hwp -p 1
target/debug/rhwp export-svg samples/footnote-01.hwp -o output/debug/task598_stage1 --debug-overlay -p 0
```

초기 `cargo run` 은 바이너리 지정 누락으로 실패했으며, 이후 `--bin rhwp` 로 재실행했다. crates.io 인덱스 접근이 sandbox 네트워크 제한에 막혀 승인 후 재실행했고, 그 뒤 빌드 및 진단 명령은 정상 완료했다.

## 샘플 재현 결과

`samples/footnote-01.hwp` 는 6페이지 문서다. 1페이지에서 본문 각주가 있는 대표 문단은 다음과 같다.

```text
문단 0.3: cc=56, text_len=47, controls=1
  텍스트: "플라스틱 액체와 같은 원료를 ..."
  [0] 각주: paragraphs=1

문단 0.7: cc=22, text_len=13, controls=1
  텍스트: "3D 프린팅 기술의 장점"
  [0] 각주: paragraphs=1
```

`dump-pages -p 0` 기준 1페이지 본문 배치에는 해당 문단이 정상 포함된다.

```text
FullParagraph pi=3 ... "플라스틱 액체와 같은 원료를 ..."
FullParagraph pi=7 ... "3D 프린팅 기술의 장점"
```

SVG debug overlay 출력에서도 본문 각주 마커가 실제 렌더링된다.

```text
output/debug/task598_stage1/footnote-01_001.svg
  text x=263.25 y=384.07 "1)"
  text x=212.92 y=702.25 "2)"
  s0:pi=3 y=376.1
  s0:pi=7 y=694.3
```

따라서 문제는 마커 렌더링 부재가 아니라, 렌더된 `FootnoteMarker` 를 편집 hit test 및 cursor navigation 이 소비하지 않는 구조로 판단된다.

## 코드 대조 결과

### 1. 본문 각주 마커 렌더 노드에는 필요한 메타가 있다

`src/renderer/render_tree.rs` 의 `FootnoteMarkerNode` 는 번호, 텍스트, 구역, 문단, 컨트롤 인덱스를 가진다.

`src/renderer/layout/paragraph_layout.rs` 는 `ComposedParagraph.footnote_positions` 를 기반으로 `RenderNodeType::FootnoteMarker` 를 생성한다.

```rust
RenderNodeType::FootnoteMarker(FootnoteMarkerNode {
    number: fnum,
    text: fn_text,
    section_index,
    para_index,
    control_index: fni,
})
```

단, 여기서 `control_index` 로 들어가는 `fni` 는 현재 `footnote_positions` 배열 내 인덱스다. 문단에 비각주 컨트롤이 섞이는 경우 실제 `para.controls` 의 control index 와 달라질 수 있으므로 구현 계획에서 확인 및 보정이 필요하다.

### 2. 일반 hit test 는 `FootnoteMarker` 를 수집하지 않는다

`src/document_core/queries/cursor_rect.rs` 의 `hit_test_native()` 는 `TextRun`, 안내문 TextRun, TableCell bbox 를 수집한다. 이후 인라인 Shape/Picture 전용 bbox 검사는 있지만 `RenderNodeType::FootnoteMarker` 검사는 없다.

현재 구조에서는 본문 각주 마커를 클릭해도 별도 hit 결과가 생성되지 않고, 주변 `TextRun` hit 로 흡수되거나 본문 위치 계산으로만 처리된다.

### 3. 각주 영역 hit test 는 이미 별도 구현되어 있다

각주 영역 내부는 다음 API로 처리된다.

- `hit_test_footnote_native(page, x, y)`
- `hit_test_in_footnote_native(page, x, y)`
- `get_page_footnote_info_native(page, footnoteIndex)`
- `get_cursor_rect_in_footnote_native(page, footnoteIndex, fnParaIdx, charOffset)`

따라서 본문 마커 hit API는 기존 각주 영역 API를 대체하지 않고, 본문 마커 클릭 시 `pageFootnoteIndex` 또는 source 정보를 찾아 기존 `enterFootnoteMode()` 경로로 연결하는 역할이면 충분하다.

### 4. 좌우 이동 길이 계산에서 `Control::Footnote` 가 빠져 있다

`src/document_core/helpers.rs` 의 `navigable_text_len()` 은 Shape/Table/Picture/Equation 및 CharOverlap 일부만 반영한다.

```rust
matches!(c, Control::Shape(_) | Control::Table(_) | Control::Picture(_) | Control::Equation(_))
```

`Control::Footnote` 는 여기 포함되지 않는다.

`src/document_core/queries/doc_tree_nav.rs` 의 `classify_navigable()` 도 `Control::Footnote` 를 `Some(false)` 로 분류하지 않는다. 결과적으로 `navigate_next_editable()` 은 각주 컨트롤 위치를 1칸 inline unit 으로 취급하지 못한다.

### 5. 현재 offset 체계는 마커 앞/뒤를 분리하지 못한다

문단 0.3의 첫 각주는 SVG상 `플라스틱 액체` 뒤, `와` 앞에 렌더링된다. 이 위치는 텍스트 기준 char offset 7이다.

현재 렌더링은 각주 마커를 `TextRun` 사이에 끼워 넣지만, 각주 마커 자체는 char offset 을 소비하지 않는다. 따라서 offset 7은 마커 앞 위치로도, 마커 뒤의 다음 `TextRun` 시작 위치로도 해석될 수 있다.

문단 0.7처럼 각주가 문단 끝에 있는 경우도 `text_len=13` 이고, `navigable_text_len()` 이 13으로 유지되면 마커 뒤 위치인 14를 만들 수 없다. 이 상태에서는 이슈 #598의 Delete/Backspace 삭제 전제인 “각주 앞/뒤 위치”가 성립하지 않는다.

## Stage 1 결론

1차 구현은 다음 두 축을 함께 처리해야 한다.

1. **본문 각주 마커 hit test**
   - 렌더 트리에서 `FootnoteMarker` bbox 를 수집한다.
   - 클릭 좌표가 bbox 안이면 `sectionIndex`, `paragraphIndex`, 실제 `controlIndex`, `footnoteNumber`, `pageFootnoteIndex`, `cursorRect` 를 반환한다.
   - `pageFootnoteIndex` 는 현재 페이지의 `page.footnotes` 와 source 를 대조해 산출하는 방식이 적합하다.

2. **본문 각주 마커의 logical cursor unit 반영**
   - `Control::Footnote` 를 본문 inline control 1칸으로 취급한다.
   - `navigable_text_len()` 또는 신규 footnote-aware length helper 에서 footnote 위치 뒤 offset 을 허용한다.
   - `navigate_next_editable()` 에서 forward/backward 이동 시 footnote 위치에서 멈추고, 한 번 더 이동하면 footnote 뒤 위치로 이동하도록 조정한다.
   - `get_cursor_rect_native()` 는 마커 앞/뒤 offset 을 구분할 수 있도록 `FootnoteMarker` bbox 또는 주변 TextRun 좌표를 사용해야 한다.

## 구현 계획서에 반영할 결정 사항

| 항목 | Stage 1 판단 |
|------|--------------|
| 본문 hit API | 신규 API가 필요하다. 기존 `hitTestFootnote` 는 각주 영역 전용으로 유지한다. |
| 반환값 | `hit`, `sectionIndex`, `paragraphIndex`, `controlIndex`, `footnoteNumber`, `footnoteIndex`, `cursorRect` 권장 |
| control index | `FootnoteMarkerNode.control_index` 를 실제 `para.controls` 인덱스로 보정하거나, hit API 내부에서 source 대조로 보정한다. |
| 커서 단위 | Footnote를 Shape/Picture/Equation과 같은 1칸 inline unit 으로 취급하되, 각주 영역 진입과 삭제 전제를 위해 앞/뒤 offset 분리 필요 |
| 1차 범위 | body source 우선. 표 셀/글상자 내부 각주는 후속 확장 대상으로 분리 |

## 검증 산출물

- `output/debug/task598_stage1/footnote-01_001.svg`
- 문단 0.3, 0.7 `dump` 결과
- 1페이지 `dump-pages` 결과

`output/` 하위 SVG는 `.gitignore` 대상이므로 커밋 대상에서 제외한다.

## 다음 단계

작업지시자 승인 후 Stage 2 구현 계획서를 작성한다.
