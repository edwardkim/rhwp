# Task #1052 구현 계획서 — 글상자 안 각주 본문 누락 정정

- 이슈: [#1052](https://github.com/edwardkim/rhwp/issues/1052)
- 수행 계획서: [task_m100_1052.md](task_m100_1052.md)
- 브랜치: `local/task1052`
- 일시: 2026-05-21

## 1. 진단 정밀화 결과

### 1.1 코드 경로 매트릭스

| 경로 | Body 각주 | TableCell 각주 | ShapeTextBox 각주 |
|------|-----------|---------------|-------------------|
| **`engine.rs` (paginator)** | ✓ line 1430 | ✓ line 1774 | ✓ **line 1376-1398 (이미 구현)** |
| **`typeset.rs` (typeset)** | ✓ line 1324 | ✓ line 2317 | ❌ **누락** |
| **`get_footnote_paragraphs`** | ✓ | ✓ | ✓ **(이미 구현 line 1084-1104)** |
| **`layout_footnote_area`** | ✓ | ✓ | ✓ (`fn_paras` 받아 통일 처리) |

### 1.2 본 sample 의 진입 경로 확인

- `samples/hwpx/footnote-tbox-01.hwpx` (HWPX) → typeset 경로 진입 추정 (Stage 1 에서 정량 확인)
- engine 경로 진입 시 글상자 안 각주 본문 표시되어야 정상 (코드 이미 있음)

### 1.3 결함 본질 분리

- **typeset.rs 에 engine.rs:1376-1398 동등 코드 누락** = 본질
- `get_footnote_paragraphs` + `layout_footnote_area` = 이미 ShapeTextBox 처리 완비

→ **수정 영역 한정**: typeset.rs 의 Shape 분기에 글상자 안 각주 수집 코드 추가 (engine.rs 패턴 미러링).

## 2. Stage 별 구현 계획 (3 단계)

### Stage 1 — 정밀 진단 (paginator 경로 + push 위치 확정)

목표:
- 본 sample (HWPX + HWP) 의 실제 paginator 경로 식별 (engine vs typeset)
- engine 경로일 경우 — 본질 위치 재정정 (layout 단계 회귀)
- typeset 경로일 경우 — Stage 2 진행

수행:
- 정량 진단 테스트 작성 (FootnoteRef 수집 결과 단언)
- 본 환경에서 SVG export 후 SVG 안 "글상자 내부 각주" 텍스트 부재 확인 (이미 입증)
- engine.rs vs typeset.rs 경로 분기 조건 dump (paginator entry 지점 디버그 출력)

산출물:
- `mydocs/working/task_m100_1052_stage1.md` — paginator 경로 + push 위치 확정 보고서

### Stage 2 — typeset.rs 글상자 안 각주 수집 (본질 정정)

목표:
- typeset.rs:1240 `Control::Shape(_)` 분기 안에 engine.rs:1376-1398 동등 코드 추가
- `FootnoteSource::ShapeTextBox` push + `add_footnote_height`

수행:
- typeset.rs:1278 (st.current_items.push(item) 직후) 글상자 안 각주 수집:

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

위치: typeset.rs:1278 `match routed { ... None => st.current_items.push(item) }` 블록 직후 (push 완료 후).

산출물:
- 코드 변경 (typeset.rs +20 라인 추정)
- 빌드 + lib 테스트 + 본 sample SVG export "글상자 내부 각주" 텍스트 정합 입증
- `mydocs/working/task_m100_1052_stage2.md` — 정량 입증 보고서

### Stage 3 — 회귀 가드 + 광범위 sweep + 보고서

목표:
- 회귀 가드 `tests/issue_1052_footnote_in_textbox.rs` 영구화
- 광범위 sweep (footnote 관련 fixture + 일반 fixture)
- WASM 빌드 + studio 동기화 + 작업지시자 시각 판정

수행:
- 회귀 가드 작성:
  ```rust
  #[test]
  fn issue_1052_footnote_in_textbox_appears_in_footer_area() {
      let svg = export_svg("samples/hwpx/footnote-tbox-01.hwpx");
      assert!(svg.contains("글상자 내부 각주"),
              "글상자 안 각주 본문이 페이지 하단 각주 영역에 표시되어야 함");
      assert!(svg.contains("일반 문단내 각주"),
              "본문 직속 각주는 회귀 부재 (기존 동작)");
  }
  ```
- sweep fixture 후보:
  - `samples/hwpx/footnote-tbox-01.hwpx` (본 sample, HWPX)
  - `samples/footnote-tbox-01.hwp` (본 sample, HWP)
  - `samples/footnote-01.hwp` (본문 직속 각주, 회귀 부재 확인)
  - `samples/2010-01-06.hwp` (각주 multi-paragraph, 회귀 부재)
  - `samples/table-in-tbox.hwp` (글상자 컨테이너, 회귀 부재)
  - 일반 fixture (aift / KTX / biz_plan / exam_kor)
- 자동 검증: cargo test --lib + --tests + clippy + fmt
- WASM Docker 빌드 + rhwp-studio public 동기화
- 작업지시자 시각 판정 게이트:
  - HWPX 로드 → SVG 페이지 하단 각주 영역 "1) 글상자 내부 각주" + "2) 일반 문단내 각주" 정합
  - 한컴 PDF (`pdf-large/hwpx/footnote-tbox-01.pdf`) 정합

산출물:
- 코드 변경 (tests/issue_1052_footnote_in_textbox.rs 신규)
- sweep 결과 (`output/poc/issue_1052/{before,after}/`)
- WASM 빌드 + studio 동기화
- `mydocs/working/task_m100_1052_stage3.md` — sweep + 시각 판정 보고서
- `mydocs/report/task_m100_1052_report.md` — 최종 보고서

### Stage 4 — merge + close + orders + archives

- no-ff merge `local/task1052` → `local/devel` → push devel
- Issue #1052 close
- 임시 산출물 정리, archives 이동
- orders 갱신

## 3. 위험 / 완화

| 위험 | 완화 |
|------|------|
| typeset.rs 정정 후 engine.rs 회귀 가능성 | Stage 3 sweep 으로 양쪽 경로 fixture 회귀 부재 입증 |
| `FootnoteSource::ShapeTextBox` 의 fn_paras 조회 누락 | 이미 `get_footnote_paragraphs` 구현 완료 (line 1084-1104) — 신규 추가 불필요 |
| 본 sample 이 engine 경로로 진입 (typeset 무관) | Stage 1 진단 결과에 따라 본질 영역 재정정 (layout 단계 회귀 가능성) |
| HWP variant 결함 본질 동일 여부 | Stage 3 sweep 으로 HWPX + HWP 양쪽 정합 입증 |

## 4. 비범위 / 후속

- 머리말/꼬리말 안 각주 (본 sample 본문 + 글상자만)
- 미주 (Endnote) 의 Shape 내부 처리 (별도 검토)
- table-in-tbox 의 안 표 셀 안 각주 (별도 가드 필요 시 후속)

## 5. 작업지시자 결정 요청

| 결정 | 옵션 |
|------|------|
| 진행 여부 | A. 본 계획 승인 (3 단계 구현 + Stage 4 merge) / B. 단계 추가/제외 / C. 보류 |
| Stage 2 본질 코드 위치 | typeset.rs:1278 (st.current_items.push 직후) / 별도 location |
| Stage 3 sweep 범위 | 본 계획 기본 (5+4 fixture) / 광범위 (변환본 9종 포함) |
