# Task #525 구현 계획서 — A안 (layout.rs:3534 layout_column_shapes_pass 중복 호출 제거)

## 1. 개요

Stage 1 진단 (`mydocs/working/task_m100_525_stage1.md`) 결과에 따라 비-TAC Picture wrap=Square host paragraph 의 wrap-around 텍스트 중복 emit 결함 정정. `layout_wrap_around_paras` 가 두 곳에서 호출되는 중복 (layout.rs:3106 `layout_shape_item` + layout.rs:3534 `layout_column_shapes_pass`) 중 후자 (3534, typeset 경로 fallback 으로 도입된 것) 를 제거.

영향 범위는 7 샘플 170 페이지 중 **37 페이지 / 205 dup-instances** (Stage 1 §4). 사용자 보고 단일 케이스 (exam_science pi=37) 외 4 샘플 잠복.

## 2. 코드 변경 위치 한정

| 파일 | 라인 | 변경 |
|------|------|------|
| `src/renderer/layout.rs` | 3499-3546 | `// 비-TAC Square wrap 그림/도형: 어울림 문단 렌더링.` if 블록 전체 제거. 대체 주석으로 layout_shape_item:3106 가 동일 처리 수행함을 명시. |

다른 파일 무수정 (`paragraph_layout.rs`, `shape_layout.rs`, `composer.rs`, `typeset.rs`, `pagination/engine.rs`).

## 3. 단계 (3단계)

### Stage 2 — 정정 구현

**핵심 변경**: `layout.rs:3499-3546` 의 `// 비-TAC Square wrap 그림/도형: 어울림 문단 렌더링.` 으로 시작하는 if 블록 전체 제거.

```rust
// REMOVE (layout.rs:3499-3546):
// 비-TAC Square wrap 그림/도형: 어울림 문단 렌더링.
// typeset.rs 경로에서 PaginationResult.wrap_around_paras는 항상 비어있으므로
// col_content.wrap_around_paras를 직접 사용해야 함.
// 용지 기준(page-relative) 그림도 어울림 텍스트는 body 기준 좌표로 렌더링.
{
    let (opt_cm, opt_pic_h) = if let Some(ctrl) = paragraphs.get(para_index)
        .and_then(|p| p.controls.get(control_index))
    {
        match ctrl {
            Control::Shape(shape) => { ... }
            Control::Picture(pic) => { ... }
            _ => (None, 0.0),
        }
    } else { (None, 0.0) };
    if let Some(cm) = opt_cm {
        if !cm.treat_as_char && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square) {
            ...
            self.layout_wrap_around_paras(
                tree, col_node, paragraphs, composed, styles, col_area,
                page_content.section_index,
                para_index, &col_content.wrap_around_paras,
                para_y, para_y + opt_pic_h,
                wrap_text_x, wrap_text_width, 0.0,
                bin_data_content,
                None,
            );
        }
    }
}
```

**대체 주석** (간결한 한 줄):
```rust
// 비-TAC Picture/Shape Square wrap 의 어울림 문단 렌더링은
// layout_shape_item:3106 (PageItem::Shape 처리 시) 에서 수행. 여기 별도 fallback
// 호출은 중복 (Task #525) 이라 제거.
```

**완료 기준**:
- `cargo build --release` 성공
- `cargo test --lib --release` 1122+ pass (기존 무회귀)
- pi=37 의 `layout_composed_paragraph` 호출이 3회 → 2회 감소 (정상 PageItem 1회 + layout_shape_item:3106 wrap-around 1회)
- pi=37 ls[6]~7 의 dup-instances 0개 (재측정)

### Stage 3 — 회귀 검증

- `cargo test --lib --release` (전체 통과)
- `cargo clippy --release --lib -- -D warnings` (warning 0)
- `scripts/svg_regression_diff.sh build HEAD~1 HEAD` (7 샘플 byte 비교)
- 변경 페이지 분류:
  - **의도된 정정**: Stage 1 §4 표 의 37 페이지 (exam_kor 16 + exam_eng 6 + exam_science 4 + exam_math 5 + 2010-01-06 4 + synam-001 1 + aift 1)
  - **회귀**: 위 외 페이지 — 0 건 목표
- dup-instances 재측정: 변경 페이지에서 dup 인스턴스 0 (또는 양의 감소) 확인

**완료 기준**: 회귀 0 건. 의도된 정정 페이지의 dup-instances 0 또는 거의 0.

### Stage 4 — 최종 보고서 + close

- `mydocs/report/task_m100_525_report.md`
- `mydocs/orders/20260503.md` 갱신 (#525 행 → 완료)
- merge: `local/task525` → `local/devel` → `devel` (push)
- `gh issue close 525`

## 4. 위험·대응

| 위험 | 영향 | 대응 |
|------|------|------|
| typeset 경로 (PaginationResult.wrap_around_paras 비어있음) 에서 wrap-around 텍스트 미렌더 | 그림 옆 텍스트 사라짐 | Stage 1 진단 결과 layout_shape_item:3106 가 동일 처리 수행 — 의도된 동작 유지. 만약 회귀 발생 시 Stage 1 정정 (다른 안 채택) 으로 fallback. |
| 본 결함 영향 외 페이지 회귀 | 광범위 회귀 | svg_regression_diff 7 샘플 byte 비교 — Stage 1 §4 영향 페이지 외 변경은 회귀로 분류, 0 건 목표. |
| 사용자 시각 검증 부담 (37 페이지) | 시각 정합 판정 지연 | 본 보고서에 변경 페이지 목록 + dup-instances 감소 비교 (BEFORE/AFTER) 명시. 작업지시자가 핵심 케이스 (exam_science p2 8번 + exam_kor 14p) 만 우선 검증해도 됨. |
| 호출 1 (PageItem::FullParagraph) 만으로 그림 옆 텍스트 미렌더 | 그림 옆 텍스트 사라짐 | 호출 1 자체가 `has_picture_shape_square_wrap` 분기로 ls[0..5] 좁은 영역 + ls[6..7] 넓은 영역 모두 처리 (paragraph_layout.rs:822, 973-982). 정상 동작. |

## 5. 검증 게이트

| 게이트 | 도구 | 기준 |
|--------|------|------|
| 빌드 | `cargo build --release` | 성공 |
| 단위 테스트 | `cargo test --lib --release` | 1122+ pass |
| Clippy | `cargo clippy --release --lib -- -D warnings` | warning 0 |
| 회귀 검증 | `scripts/svg_regression_diff.sh` | 7 샘플 영향 외 byte-identical, 영향 37 페이지는 의도 정정 |
| dup-instances 감소 | python 추출 (Stage 1 §4 방법) | 합계 205 → 거의 0 |
| 시각 정합 | export-svg + 작업지시자 PDF 비교 | exam_science p2 8번 문제 ls[6]~7 중복 0 |

## 6. 참고

- 본 변경은 Task #524 (Square wrap 그림 anchor 정정 — 비-TAC Picture vert_align=Top 위치 산출) 와 영역 인접하나 본질 독립.
- A안 채택 후에도 호출 1 (정상 PageItem) + 호출 2 (layout_shape_item:3106) 가 남음. 추가 중복 의심 시 Stage 3 결과 분석 후 follow-up task.
- layout.rs:3499-3502 주석에서 명시하는 "typeset.rs 경로에서 PaginationResult.wrap_around_paras는 항상 비어있다" 가정은 본 정정으로 의미 없어짐. typeset 경로도 layout_shape_item 을 거치므로 fallback 불필요.

---

승인 요청: 본 구현 계획서대로 Stage 2 정정 진행 가능 여부.
