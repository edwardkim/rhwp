# M100 #2308 Stage 1 — 현행 계약 특성화와 무효화 감사

## 기준

- 브랜치: `issue-2308-render-normalized-derived-state`
- 구현 기준선: `upstream/devel@29b5547e256a3d6a1f8c94c9434c14a351b5543a`
- 조사일: 2026-07-23

## 현재 소유권과 소비 경로

1. `DocumentCore.render_normalized`가 섹션별
   `Option<(Vec<Paragraph>, Vec<ComposedParagraph>)>`를 소유한다.
2. `paginate_pass()` 시작마다 `compute_render_normalized()`를 호출한다.
3. #2004 본문/셀 이미지 스택 또는 #2195 중첩 표 stretch가 하나라도 있으면
   `section.paragraphs.clone()`으로 섹션 전체를 복제한다.
4. 측정, typeset, page tree, dump 경로는 `section_render_paragraphs()`와
   `section_render_composed()`를 통해 복제본을 우선 사용한다.
5. deferred cell edit은 pagination을 실행하지 않으므로
   `refresh_render_normalized_cell_paragraph_after_edit()`가 복제 트리를 직접 수정한다.

## mirror 경로 계약

| 경로 | 현행 표현 | 실패 동작 |
|---|---|---|
| 일반 표 셀 | `control_idx/cell_idx/cell_para_idx` | index 불일치 시 조용히 반환 |
| 표 caption | `cell_idx == 65534` sentinel | caption 또는 index 불일치 시 조용히 반환 |
| 도형 textbox | control variant + textbox helper | textbox 또는 index 불일치 시 조용히 반환 |
| 그림 caption | control variant + caption | caption 또는 index 불일치 시 조용히 반환 |

mirror 성공 뒤에는 상위 복제 문단 전체를 다시 순회해 #2195 stretch를 재적용한다. 일반 셀은
원본 포인터 캐시와 복제본 포인터 캐시를 각각 무효화한다.

## 실제 정규화 delta

### #2004

- 그림/그림 도형의 effective `treat_as_char=true`
- 본문 스택의 그림별 합성 composed line
- 셀 스택의 그림 1개짜리 paragraph/LINE_SEG projection

### #2195

- 비-TAC 중첩 표의 effective width를 부모 셀 폭까지 확대
- 중첩 표 cell width를 같은 배율로 확대

text, char shape, 일반 LINE_SEG, caption 본문은 원본과 동일하며 별도 소유가 필요 없다.

## 기존 재사용 계약

- `LayoutEngine.cell_units_cache`: source 또는 normalized `Cell` 포인터 → `Arc<Vec<CellUnit>>`
- `table_nested_text_flag_cache`: `Table` 포인터 → bool
- #2214 deferred edit은 편집 셀 key만 제거하고 sibling cache identity를 보존한다.
- 전체 normalized section 재생성은 포인터를 모두 바꾸므로 sibling 재사용과 양립하지 않는다.

## mutation 분류

| 범주 | 대표 진입점 | 초기 normalization scope |
|---|---|---|
| 문서/구역 교체 | document load/create, undo/redo restore | Document |
| 문단/control/cell 구조 변경 | table ops, clipboard, split/merge, object insert/delete | Section |
| 페이지 기하 변경 | page/column/section definition | Section |
| 일반 셀 deferred text edit | `insert_text_in_cell_native_impl` | Path |
| caption/textbox deferred text edit | 같은 cell-path 편집 API의 특수 경로 | Path |
| 분류 불명 mutation | 기존 `mark_section_dirty()` 호출부 | Section |

초기 구현은 기존 `mark_section_dirty()`를 section-scope 안전 기본값으로 사용한다. path-scope는
구조가 불변인 deferred cell text edit 한 경로에서 먼저 활성화한다.

## 구현 경계

1. #2195 폭 변화는 source table clone 대신 logical path의 width-scale projection으로 옮긴다.
2. #2004는 전체 section clone이 아니라 영향 paragraph/control의 sparse projection으로 옮긴다.
3. renderer의 빠른 조회는 source pointer index를 사용할 수 있지만, cache identity와 revision
   판정의 권위 key는 `RenderPath`다.
4. path 해석 실패는 stale entry 재사용 없이 section rederive로 승격한다.
5. section rederive 실패는 source fallback 또는 명시적 `RenderError`로 표면화한다.

## RED guard

`tests/issue_2308_render_normalized_guard.rs`는 다음 구조가 제거될 때까지 실패해야 한다.

- `section.paragraphs.clone()`
- `refresh_render_normalized_cell_paragraph_after_edit`
- `cell_idx == 65534` caption sentinel
- `RenderNormalizationState` / `RenderPathEntry` 부재

실행 결과:

```text
cargo test --test issue_2308_render_normalized_guard
test issue_2308_uses_revision_overlay_without_clone_mirror ... FAILED
DocumentCore must own an explicit render normalization derived-state cache
```

첫 assertion에서 의도대로 RED가 확인됐다. 이후 assertion은 구현 단계에서 순차적으로
GREEN으로 전환한다.
