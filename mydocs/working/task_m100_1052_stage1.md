# Task #1052 Stage 1 보고서 — 정밀 진단 (paginator 경로 확정)

- 이슈: [#1052](https://github.com/edwardkim/rhwp/issues/1052)
- 단계: Stage 1 (정밀 진단)
- 일시: 2026-05-21

## 1. 결과 요약

**본질 확정**: 본 sample (HWPX + HWP) 은 default 경로 = **TypesetEngine** 로 진입.
typeset.rs:1278 (Shape `current_items.push` 직후) 글상자 안 각주 수집 코드 누락이 결함의 본질.

→ 구현 계획서의 Stage 2 (typeset.rs 정정) 진행 확정.

## 2. paginator 경로 분기 확인

`src/document_core/queries/rendering.rs:1548-1580`:

```rust
// TypesetEngine을 main pagination으로 사용. RHWP_USE_PAGINATOR=1 로 fallback 가능.
let use_paginator = std::env::var("RHWP_USE_PAGINATOR")
    .map(|v| v == "1")
    .unwrap_or(false);
let mut result = if use_paginator {
    paginator.paginate_with_measured_opts(...)   // engine.rs (legacy)
} else {
    let typesetter = TypesetEngine::new(self.dpi);
    typesetter.typeset_section_with_variant(...)  // typeset.rs (default, main)
};
```

→ default = TypesetEngine. engine.rs 는 `RHWP_USE_PAGINATOR=1` env var fallback only.

**Task #993 명시** (`src/renderer/pagination/tests.rs:594` ignore reason):
> "레거시 Paginator(engine.rs)는 분할 표 컷을 생산하지 않음 — TypesetEngine 컷 모델로 대체"

## 3. 코드 경로 매트릭스 (재확인)

| 경로 | Body 각주 | TableCell 각주 | ShapeTextBox 각주 | 사용 |
|------|-----------|---------------|-------------------|------|
| `engine.rs` (legacy) | ✓ line 1430 | ✓ line 1774 | ✓ line 1376-1398 | env opt-in |
| **`typeset.rs` (main)** | ✓ line 1324 | ✓ line 2317 | ❌ **누락** | **default** |
| `get_footnote_paragraphs` | ✓ | ✓ | ✓ line 1084-1104 | 통일 |
| `layout_footnote_area` | ✓ | ✓ | ✓ (fn_paras 통일) | 통일 |

## 4. 결함 재현 정량

`samples/hwpx/footnote-tbox-01.hwpx` → `output/poc/issue_footnote_tbox/svg/footnote-tbox-01.svg`:

SVG 텍스트 element 분석 (42개):
- "글상자 내부에 각주가 있는 경우" (본문) ✓
- "와우" (본문) ✓
- "사람들은" + **"2)" + "일반 문단내 각주"** (본문 + 본문 각주) ✓
- "여기에 각주가 들어있는 경우" (글상자 안 본문) ✓
- **"1)"** (글상자 안 각주 번호 마크) ← 글상자 안에 표시 ✓
- **"1) 글상자 내부 각주" — 페이지 하단 각주 영역 부재 ❌**

한컴 PDF 정답지 (`pdf-large/hwpx/footnote-tbox-01.pdf`):
```
1) 글상자 내부 각주
2) 일반 문단내 각주
```

## 5. Stage 2 진행 결정

본 진단 결과로 구현 계획서의 Stage 2 (typeset.rs:1278 직후 글상자 안 각주 수집 코드 추가) 진행 확정.

위치: `typeset.rs:1276-1278`
```rust
match routed {
    Some((page_idx, col_idx)) => { ... }
    None => {
        st.current_items.push(item);
    }
}
// ← 여기 (1278 직후) 글상자 안 각주 수집 코드 추가
```

추가 코드 (engine.rs:1376-1398 동등 미러링):
```rust
// [Task #1052] 글상자 내 각주 수집 (engine.rs:1376-1398 동등)
if let Control::Shape(shape_obj) = ctrl {
    if let Some(text_box) = shape_obj.drawing().and_then(|d| d.text_box.as_ref()) {
        for (tp_idx, tp) in text_box.paragraphs.iter().enumerate() {
            for (tc_idx, tc) in tp.controls.iter().enumerate() {
                if let Control::Footnote(fn_ctrl) = tc {
                    if let Some(page) = st.pages.last_mut() {
                        page.footnotes.push(FootnoteRef {
                            number: fn_ctrl.number,
                            source: FootnoteSource::ShapeTextBox {
                                para_index: para_idx,
                                shape_control_index: ctrl_idx,
                                tb_para_index: tp_idx,
                                tb_control_index: tc_idx,
                            },
                        });
                        let fn_height = Self::estimate_footnote_height(fn_ctrl, self.dpi);
                        st.add_footnote_height(fn_height);
                    }
                }
            }
        }
    }
}
```

## 6. 잔여 확인 항목 (Stage 2 시 검증)

- `estimate_footnote_height` 가 typeset.rs 의 associated fn 으로 호출 가능한지 (기존 line 1335 동등 사용)
- `add_footnote_height` 가 typeset state 에 존재하는지 (기존 line 1336 동등 사용)
- `has_table` 가드 적용 여부 — 본문 각주 (line 1325 `if !has_table`) 와 동일 정합 필요

기존 코드 (line 1324-1338) 가 동일 패턴으로 동작 — Stage 2 에서 미러링.
