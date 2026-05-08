# Task #604 Stage 2 — typeset 출력 메타데이터 (wrap_anchors) 도입

## 본 단계 목표

R3 본질 — typeset.rs 의 wrap_around state machine 매칭 결과를 layout 시점까지 전달하는
**메타데이터 채널** 도입. wrap_precomputed 플래그 (IR 누설) 의 정합한 대체 메커니즘.

## Stage 2a 시도 결과 (revert)

옵션 C (`LineSeg::is_in_wrap_zone(col_w_hu)` 단독 판정) 본질 부적합 — test_547 회귀.
HWP5 native passage box 본문 LineSeg `cs=852, sw=30184` 가 false-positive 판정. 전체
변경 revert. R3 (Stage 2 신규 본질) 채택.

## 변경 영역

### A. `src/renderer/pagination.rs` — `WrapAnchorRef` struct + `ColumnContent.wrap_anchors`

```rust
pub struct WrapAnchorRef {
    pub anchor_para_index: usize,  // anchor 문단 (그림/표 보유)
    pub anchor_cs: i32,            // wrap zone cs (HWPUNIT)
    pub anchor_sw: i32,            // wrap zone sw (HWPUNIT)
}

pub struct ColumnContent {
    // 기존 +
    pub wrap_anchors: HashMap<usize, WrapAnchorRef>,  // para_index → wrap context
}
```

### B. `src/renderer/typeset.rs` + `pagination/state.rs` — TypesetState/PaginationState 필드

`current_column_wrap_anchors: HashMap<usize, WrapAnchorRef>` 추가. flush_column 에서
ColumnContent 로 take.

### C. typeset.rs:495~ wrap_around 매칭 시 등록

매칭 성공 + `para.wrap_precomputed=true` (HWP3 파서가 set) 인 경우:
- `current_column_wrap_anchors.insert(para_idx, WrapAnchorRef { ... })`
- 흡수 skip → FullParagraph 흐름 통과

### D. `src/renderer/layout.rs` ColumnItemCtx + layout_column_item

- `ColumnItemCtx.wrap_anchors: &HashMap` 필드 추가
- `layout_column_item` 시그니처 `wrap_anchors` 인자 추가 + 호출처 정합화
- ColumnItemCtx destructure 3곳에 `wrap_anchors` 추가

### E. `src/renderer/layout/paragraph_layout.rs` 시그니처 정합화

- `layout_paragraph` / `layout_partial_paragraph` / `layout_composed_paragraph` 모두
  `wrap_anchor: Option<&WrapAnchorRef>` 인자 추가
- `wrap_precomputed` 검사 3곳 (line 862, 883, 1208) → `wrap_anchor.is_some()` 검사로 교체

### F. 모든 호출처 (21곳) wrap_anchor 인자 전달

- **PageItem::FullParagraph/PartialParagraph 처리 (layout.rs)**: `ctx.wrap_anchors.get(para_index)` 또는 `wrap_anchors.get(&para_index)`
- **머리말/꼬리말 / 바탕쪽 / 셀 / 도형 / 각주 / 캡션** (모든 비-wrap 컨텍스트): `None` 전달

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` + `cargo build --tests` | ✅ 통과 |
| `cargo test --lib` | ✅ **1130 passed** / 0 failed / 2 ignored |
| `cargo test --test issue_546` (Task #546) | ✅ 1 passed (exam_science 4페이지) |
| `cargo test --test issue_554` (HWP3 변환본) | ✅ 12 passed |
| `cargo test` (통합 31) | ✅ 모두 통과 |
| `test_547_passage_text_inset_match_pdf_p4` | ✅ PASS (Stage 2a 회귀 해결) |

## LOC 합계

| 파일 | 변경 |
|------|-----|
| `src/renderer/pagination.rs` | +21 LOC (WrapAnchorRef + ColumnContent.wrap_anchors + clone) |
| `src/renderer/pagination/state.rs` | +6 LOC (필드 + init + flush + clear) |
| `src/renderer/typeset.rs` | +14 LOC (필드 + init + flush 2곳 + 매칭 분기) |
| `src/renderer/layout.rs` | +20 LOC (ColumnItemCtx 필드 + destructure 3곳 + 7 호출처) |
| `src/renderer/layout/paragraph_layout.rs` | +9 LOC (3 시그니처 + 3 검사 교체) |
| `src/renderer/layout/{table_layout,table_partial,table_cell_content,shape_layout,picture_footnote}.rs` | +7 LOC (None 인자 추가) |
| `src/renderer/page_number.rs` + `layout/tests.rs` | +6 LOC (테스트 ColumnContent 정합) |
| **소스 합계** | **+83 LOC** |

## 본질 정정 효과

| 영역 | 본질 |
|------|------|
| **typeset 출력 메타데이터** | wrap_around state machine 매칭 결과가 layout 시점까지 보존 |
| **layout wrap zone 판정** | `wrap_anchor.is_some()` — Paragraph IR 의존성 제거 |
| **포맷 일관성** | typeset 의 매칭은 LineSeg cs/sw 비교 — HWP3/HWP5/HWPX 동일 |
| **IR 부채 일부 청산** | layout 가 `Paragraph.wrap_precomputed` 미참조 — IR 정합화 진전 |

## 잔존 부채 (Stage 2b 영역)

- `Paragraph.wrap_precomputed` 필드 자체는 IR 에 남음 (typeset.rs:502 에서 매칭 분기 조건)
- HWP3 파서 후처리 30 LOC 잔존 (`mod.rs:1556~`)
- Stage 2b 에서 본질 정합 — anchor 종류 (Picture vs Table) 기반 분기로 교체

## 다음 단계 (Stage 2b) 영역 미리보기

- typeset.rs:502 `if para.wrap_precomputed` → anchor 가 Picture 인지 검사 (anchor_para.controls)
- `Paragraph.wrap_precomputed` 필드 제거
- `src/parser/hwp3/mod.rs:1556~` 후처리 30 LOC 제거
- 회귀 검증 (HWP5/HWPX 광범위 fixture 영향)

## 작업지시자 승인 요청

본 Stage 2 (R3 본격, typeset → layout wrap_anchors 메타데이터 채널 도입) 완료 보고. 다음
단계 진입 옵션:
- **Stage 2b**: `wrap_precomputed` 필드 제거 + HWP3 후처리 청산 (IR 부채 마무리)
- **Stage 3**: HWP3 파서 cs/sw 인코딩 정정 (Issue #604 결함 본질 정정)

작업지시자 결정 영역. Stage 2b 먼저 진행 권장 (IR 청산 후 안정 base 에서 Stage 3 진행).

## 참조

- 수행계획서: `mydocs/plans/task_m100_604.md`
- 구현계획서: `mydocs/plans/task_m100_604_impl.md`
- LineSeg 표준: `mydocs/tech/document_ir_lineseg_standard.md`
- Stage 1 보고서: `mydocs/working/task_m100_604_stage1.md`
- Issue: #604
