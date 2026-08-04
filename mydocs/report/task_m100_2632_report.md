# task_m100_2632 처리결과 보고서 — HeightMeasurer 본문 재래핑 래퍼 정합

- **이슈**: [#2632](https://github.com/edwardkim/rhwp/issues/2632)
- **브랜치**: `task/m100-2632-height-measurer-body-recompose` (base `devel` @ `3c54abfd`)
- **범위**: `src/renderer/height_measurer.rs` 한 지점
- **분류**: 결함 수정 (측정↔렌더 불일치)

## 1. 문제

`HeightMeasurer::measure_paragraph` 의 본문(column) NO_LS 재래핑 분기가 형제 두 경로
(`typeset.rs::format_paragraph`, `paragraph_layout.rs::layout_partial_paragraph`)와 **다른
래퍼 함수**를 썼다.

| 파일 | 함수 | 사용 래퍼(수정 전) |
|---|---|---|
| `height_measurer.rs:527` | `measure_paragraph` | `recompose_for_cell_width` |
| `typeset.rs:11162` | `format_paragraph` | `recompose_for_body_width` |
| `paragraph_layout.rs:1911` | `layout_partial_paragraph` | `recompose_for_body_width` |

`recompose_for_body_width`(`composer.rs:1419-1427`)는 `recompose_for_cell_width` 의
superset이다:
```rust
pub fn recompose_for_body_width(...) {
    restyle_fallback_runs_by_char_shapes(composed, para);   // ← measurement 만 빠짐
    recompose_for_cell_width(composed, para, column_inner_width_px, styles);
}
```
측정 경로만 `restyle_fallback_runs_by_char_shapes` 를 건너뛰어, `compose_lines` NO_LS
fallback 이 만든 단일 스타일 run 을 실제 글자모양별로 재분할하지 않았다.

## 2. 영향

`PARA_LINE_SEG` 가 없고 글자모양이 섞인 본문 문단(예: 15pt 도입부 + 14pt 본문)에서
typeset/render 는 CharShapeRef 로 run 을 쪼개 정확히 측정하는데 `HeightMeasurer` 는 모든
run 을 단일 스타일로 측정 → 폭/줄수/줄간격 오차가 페이지네이션 입력(`document_core/queries/
rendering.rs:3065` `measure_section`)에 누적되어 실제 렌더 결과와 다른 쪽에서 분할됐다.

## 3. 변경

`src/renderer/height_measurer.rs:527` — `recompose_for_cell_width` → `recompose_for_body_width`
(한 단어). 주변 주석을 실제 코드·형제 경로와 일치하도록 갱신.

## 4. 검증

### 신규 테스트 (기전 증명)

`src/renderer/composer/tests.rs::body_recompose_splits_fallback_run_by_char_shapes_but_cell_recompose_does_not`

`compose_lines` fallback(단일 run, char_style_id=0)을 만든 뒤 두 래퍼에 각각 통과시켜:
- `recompose_for_cell_width` → run 이 `[0]` 그대로(재분할 없음)
- `recompose_for_body_width` → run 이 `[0, 1]` 로 분할(두 글자모양이 드러남)

임을 단언한다. `height_measurer.rs` 의 수정이 기대는 정확한 메커니즘(body 래퍼가 cell
래퍼의 superset)을 직접 증명하는 white-box 테스트다.

```
test renderer::composer::tests::body_recompose_splits_fallback_run_by_char_shapes_but_cell_recompose_does_not ... ok
```

### 회귀

```
cargo test --lib renderer::  →  861 passed / 0 failed / 4 ignored
```

### 미실행 항목 (투명 고지)

- **`measure_paragraph` 를 통한 end-to-end 회귀**: `HeightMeasurer::measure_paragraph` 가
  `fn` (비공개)이고 `ResolvedStyleSet` 을 실제 폰트 폭 테이블까지 채워 구성해야 픽셀 단위
  높이 차이를 직접 단언할 수 있어, 이번 검증에서는 그 대신 근본 메커니즘(composer 레벨)을
  직접 테스트했다. 호출부 자체는 한 단어 교체이고 형제 두 경로가 이미 같은 패턴으로 검증돼
  있어(각각 자체 테스트 보유, 861건에 포함), 추가 위험은 낮다고 판단했다.
- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약상
  작업지시자 별도 승인 사항이라 실행하지 않았다.

## 5. 잔여 (같은 스윕의 별건, 범위 분리)

이슈 본문에 기록. 별도 이슈로 등록해 순차 처리 예정:
- `height_measurer.rs:521` match guard 로 인해 `masked_stored_lines_stale` 재래핑 분기가
  구조적으로 도달 불가.
- `table_ops.rs:1337` `update_neighbor_borders` DocInfo passthrough 무효화 누락.
- `formatting.rs:944-965` `find_or_create_font_id_for_lang` raw/IR 글꼴 불일치.
