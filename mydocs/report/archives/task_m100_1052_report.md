# Task #1052 최종 보고서 — 글상자 안 각주 본문 누락 정정

- 이슈: [#1052](https://github.com/edwardkim/rhwp/issues/1052)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1052`
- assignee: @edwardkim
- 일시: 2026-05-21
- 수행 계획서: [task_m100_1052.md](../plans/archives/task_m100_1052.md)
- 구현 계획서: [task_m100_1052_impl.md](../plans/archives/task_m100_1052_impl.md)
- 단계별: [stage1](../working/task_m100_1052_stage1.md) / [stage2](../working/task_m100_1052_stage2.md) / [stage3](../working/task_m100_1052_stage3.md)

## 1. 결함 본질

샘플: `samples/hwpx/footnote-tbox-01.hwpx` + `samples/footnote-tbox-01.hwp` + 한컴 PDF 정답지 [`pdf-large/hwpx/footnote-tbox-01.pdf`](../../pdf-large/hwpx/footnote-tbox-01.pdf)

글상자 안 paragraph 에 정의된 각주의 마크 ("1)") 는 글상자 본문에 정상 표시되나, 각주 본문 ("글상자 내부 각주") 이 페이지 하단 각주 영역에서 누락.

**근본 원인** (`feedback_image_renderer_paths_separate` 정합):

| 경로 | Body 각주 | TableCell 각주 | ShapeTextBox 각주 | 사용 |
|------|-----------|---------------|-------------------|------|
| `engine.rs` (legacy) | ✓ line 1430 | ✓ line 1774 | ✓ line 1376-1398 | env opt-in (`RHWP_USE_PAGINATOR=1`) |
| **`typeset.rs` (main)** | ✓ line 1324 | ✓ line 2317 | ❌ **누락** | **default** |

Task #993 이후 TypesetEngine 이 main paginator (default). engine.rs 는 fallback only. 본 sample 은 typeset.rs 경로로 진입하면서 ShapeTextBox 각주 수집이 누락 → `page.footnotes` 에 추가되지 못함.

`get_footnote_paragraphs` (picture_footnote.rs:1084-1104) + `layout_footnote_area` 는 이미 `FootnoteSource::ShapeTextBox` 통일 처리 완비 — **push 만 typeset.rs 에 추가하면 해결**.

## 2. 변경 사항

### 2.1 `src/renderer/typeset.rs` (+27 라인)

위치: line 1278 (Shape `current_items.push` 직후, `Task #409 v2` 주석 직전)

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

### 2.2 `tests/issue_1052_footnote_in_textbox.rs` (88 라인, 신규)

회귀 가드 4 tests:
- `issue_1052_textbox_footnote_appears_in_footer_area_hwpx` — HWPX 본질
- `issue_1052_textbox_footnote_appears_in_footer_area_hwp` — HWP variant
- `issue_1052_body_footnote_no_regression_hwpx` — 본문 각주 회귀 부재
- `issue_1052_textbox_footnote_marker_present` — 각주 마크 본문 위치 유지

검증 헬퍼: `svg_text_sequence()` — SVG 의 모든 `<text>...</text>` 내용을 순서대로 이어붙인 문자열에서 sub-string 등장 단언 (SVG 가 글자 단위 분리되어 있으므로).

## 3. 검증 결과

### 3.1 자동 검증

| 항목 | 결과 |
|------|------|
| cargo build --lib | OK |
| cargo build --release --bin rhwp | OK |
| cargo test --release --lib | **1319 passed** |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| cargo test --release --test issue_1052_footnote_in_textbox | **4/4 passed** |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 | OK (4.90 MB) |
| rhwp-studio 동기화 | OK (`public/rhwp_bg.wasm` + `rhwp.js`) |

### 3.2 광범위 sweep (9 fixtures, 143 SVG each)

| Fixture | 페이지 수 | BEFORE/AFTER diff |
|---------|----------|-------------------|
| **samples/hwpx/footnote-tbox-01.hwpx** | 1 | **1** (의도된 본질 정정) |
| **samples/footnote-tbox-01.hwp** | 1 | **1** (의도된 본질 정정) |
| samples/footnote-01.hwp | 6 | 0 |
| samples/2010-01-06.hwp | 6 | 0 |
| samples/table-in-tbox.hwp | 2 | 0 |
| samples/aift.hwp | 74 | 0 |
| samples/KTX.hwp | 27 | 0 |
| samples/biz_plan.hwp | 6 | 0 |
| samples/exam_kor.hwp | 20 | 0 |

```
diff -rq output/poc/issue_1052/before/ output/poc/issue_1052/after/ = 2
```

→ **footnote-tbox-01 (HWPX + HWP) 만 변동** (본질 정정), **나머지 7 fixture 회귀 부재** 정량 입증.

### 3.3 정량 입증 — SVG 텍스트 element 수

| Fixture | BEFORE | AFTER | 추가 항목 |
|---------|--------|-------|----------|
| HWPX | 42 | **51** (+9) | "1)" + "글" "상" "자" " " "내" "부" " " "각" "주" |
| HWP | 42 | **51** (+9) | 동일 |

한컴 PDF 정답지 (`pdf-large/hwpx/footnote-tbox-01.pdf`):
```
1) 글상자 내부 각주
2) 일반 문단내 각주
```

→ rhwp AFTER 출력 = 한컴 정답지 완전 정합.

### 3.4 작업지시자 시각 판정

- HWPX + HWP 양쪽 fixture 페이지 하단 각주 영역 정합 ✓
- 일반 fixture (table-in-tbox / aift / KTX / biz_plan) 회귀 부재 ✓

## 4. 성공 기준 충족

| 기준 | 내용 | 결과 |
|------|------|------|
| C1 | 글상자 안 각주 본문 페이지 하단 표시 (한컴 정합) | ✓ HWPX + HWP 양쪽 정합 |
| C2 | 본문 직속 각주 (기존) 회귀 부재 | ✓ "일반 문단내 각주" 유지 |
| C3 | 글상자 안 각주 마크 (기존) 위치 정합 유지 | ✓ "1)" + "2)" 위치 유지 |
| C4 | 회귀 가드 영구화 | ✓ tests/issue_1052_footnote_in_textbox.rs (4 tests) |
| C5 | 일반 fixture 회귀 부재 | ✓ 7 fixture diff=0 |
| C6 | 자동 검증 통과 | ✓ 1319 lib + 통합 + clippy + fmt |
| C7 | 작업지시자 시각 판정 통과 | ✓ HWPX + HWP 양쪽 정합 |

## 5. 메모리 룰 정합

- ✅ `feedback_image_renderer_paths_separate` — engine.rs / typeset.rs 두 paginator 경로 동기화 누락 사례 정확 식별 + 정정
- ✅ `feedback_diagnosis_layer_attribution` — Stage 1 진단으로 본질 위치 (typeset.rs default 경로) 정확 식별
- ✅ `feedback_self_verification_not_hancom` — 한컴 PDF 정답지 (pdf-large/hwpx) 시각 정합 게이트
- ✅ `feedback_search_troubleshootings_first` — 트러블슈팅 사전 검색 (footnote_line_spacing.md 무관 확인)
- ✅ `feedback_assign_issue_before_work` — Issue #1052 등록 직후 assignee=@edwardkim 지정
- ✅ `feedback_process_must_follow` — 이슈 → 브랜치 → 수행 계획서 → 구현 계획서 → 단계별 → 보고서 절차 준수
- ✅ `feedback_visual_judgment_authority` — 작업지시자 시각 판정 게이트 통과
- ✅ `feedback_push_full_test_required` — cargo test --tests + clippy + fmt 모두 통과

## 6. 잔여 / 후속

- 머리말/꼬리말 안 각주 (본 sample 범위 외)
- 미주 (Endnote) 의 Shape 내부 처리 (별도 검토 필요)
- table-in-tbox 안 표 셀 안 각주 (별도 가드 필요 시 후속)

## 7. 관련 컨텍스트

- Task #483 (closed): 각주 multi-paragraph line_spacing 누락
- Task #696 (closed): 글상자 컨테이너 자식 표/문단 미렌더
- Task #919 (closed): `resolve_table_by_path` Shape 5 variants traverse 일반화
- Task #974 (closed): 글상자 안 그림 출력 누락
- Task #993 (closed): TypesetEngine main paginator 전환 (legacy engine.rs fallback)

본 task 는 Shape (글상자) 컨테이너 안 컨트롤 traverse 누락 시리즈 (#696/#919/#974) 의 footNote 영역 연장.
