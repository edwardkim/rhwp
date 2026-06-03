# Task #1052 Stage 2 보고서 — typeset.rs 글상자 안 각주 수집 구현

- 이슈: [#1052](https://github.com/edwardkim/edward/rhwp/issues/1052)
- 단계: Stage 2 (구현)
- 일시: 2026-05-21

## 1. 결과 요약

`src/renderer/typeset.rs:1278` 직후 (Shape `current_items.push` 완료 후) `engine.rs:1376-1398` 동등 코드를 미러링 추가. 본 sample HWPX + HWP 양쪽 모두 페이지 하단 각주 영역에 "1) 글상자 내부 각주" 표시 정합 입증.

## 2. 코드 변경

### 2.1 `src/renderer/typeset.rs` (+27 라인)

위치: line 1278 (Shape `match routed { ... }` 블록 직후, `Task #409 v2` 주석 직전)

```rust
// [Task #1052] 글상자 내 각주 수집 (engine.rs:1376-1398 동등)
// footnote-tbox-01.hwpx 의 글상자 안 각주 본문이 페이지 하단 영역
// 에 누락되는 결함 정정. engine.rs (legacy) 는 이미 처리하나
// typeset.rs (main, default) 만 누락 — feedback_image_renderer_paths_separate.
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
                        let fn_height =
                            Self::estimate_footnote_height(fn_ctrl, self.dpi);
                        st.add_footnote_height(fn_height);
                    }
                }
            }
        }
    }
}
```

설계 정합:
- `FootnoteSource::ShapeTextBox` 는 기존 enum variant 이미 정의됨 (pagination.rs)
- `get_footnote_paragraphs` 의 `FootnoteSource::ShapeTextBox` 처리 이미 구현됨 (picture_footnote.rs:1084-1104)
- `layout_footnote_area` 는 source variant 무관 통일 처리 (fn_paras 추출 후 동일 layout)
- `estimate_footnote_height` + `add_footnote_height` 는 본문 각주 분기 (line 1335-1336) 와 동일 동작

→ 본 PR 의 변경 영역 한정 — 단순 push + height 누적만.

## 3. 정량 입증 — SVG 텍스트 sequence 검사

### 3.1 HWPX `samples/hwpx/footnote-tbox-01.hwpx`

| 항목 | BEFORE | AFTER |
|------|--------|-------|
| 텍스트 element 수 | 42 | **51** (+9) |
| 글상자 안 본문 + 마크 ("여기에 각주..경우 1)") | ✓ | ✓ (회귀 부재) |
| 본문 + 본문 각주 ("사람 2) 들은") | ✓ | ✓ |
| **각주 영역 "1) 글상자 내부 각주"** | ❌ 부재 | ✓ **추가됨** |
| 각주 영역 "2) 일반 문단내 각주" | ✓ | ✓ |

### 3.2 HWP `samples/footnote-tbox-01.hwp`

| 항목 | BEFORE | AFTER |
|------|--------|-------|
| 텍스트 element 수 | 42 | **51** (+9) |
| **각주 영역 "1) 글상자 내부 각주"** | ❌ 부재 | ✓ **추가됨** |
| 회귀 부재 (기존 동작) | ✓ | ✓ |

### 3.3 한컴 PDF 정답지 정합

`pdf-large/hwpx/footnote-tbox-01.pdf`:
```
1) 글상자 내부 각주
2) 일반 문단내 각주
```

rhwp AFTER 출력 — 한컴 정답지 완전 정합.

## 4. 검증

| 항목 | 결과 |
|------|------|
| cargo build --lib | OK |
| cargo build --release --bin rhwp | OK |
| **회귀 가드** `cargo test --release --test issue_1052_footnote_in_textbox` | **4/4 passed** |
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |

## 5. 결함 본질 정합

`feedback_image_renderer_paths_separate` 정합:
- engine.rs (legacy) line 1376-1398 — 이미 구현 (env opt-in 경로)
- typeset.rs (main, default) — 본 PR 로 추가 (default 경로)

`feedback_diagnosis_layer_attribution` 정합 — Stage 1 진단으로 본질 위치 (typeset.rs default 경로) 정확 식별 + 정정.

## 6. 다음 단계

Stage 3 (회귀 가드 + sweep + WASM + 작업지시자 시각 판정) 진행.
