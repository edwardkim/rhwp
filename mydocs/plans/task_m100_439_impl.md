# Task #439 구현 계획서

> Square wrap 표 직후 col 0 over-fill (exam_kor 페이지 14 단 0 1225>1211)

- **이슈**: [#439](https://github.com/edwardkim/rhwp/issues/439)
- **브랜치**: `local/task439`
- **선행 단계**: Stage 1 (베이스라인 + 진단) — 완료
- **본 단계**: Stage 2 (원인 확정 + 구현 계획) — 코드 변경 없음

---

## 1. 원인 정밀 분석

### 1.1 정확한 cur_h 누적 추적 (페이지 14 col 0)

`typeset.rs::place_table_with_text` 에 임시 디버그를 삽입하여 실측 (수정 후 모두 revert 완료):

```
[T439-PTW] pi=33 cur_h_before=196.05 v_off=396 total_lines=4 pre_end=4 table_total=84.93 is_square=true
   pre_text push pi=33 pre_h=98.03 cur_h: 196.05 -> 294.08
   table push   pi=33 table_total=84.93 cur_h: 294.08 -> 379.01

[T439-PTW] pi=37 cur_h_before=474.03 v_off=396 total_lines=3 pre_end=3 table_total=60.75 is_square=true
   pre_text push pi=37 pre_h=73.52 cur_h: 474.03 -> 547.55
   table push   pi=37 table_total=60.75 cur_h: 547.55 -> 608.29

[T439-PTW] pi=40 cur_h_before=736.99 v_off=396 total_lines=2 pre_end=2 table_total=38.45 is_square=true
   pre_text push pi=40 pre_h=49.01 cur_h: 736.99 -> 786.00
   table push   pi=40 table_total=38.45 cur_h: 786.00 -> 824.45

[T439-PTW] pi=47 cur_h_before=1091.56 v_off=396 total_lines=3 pre_end=3 table_total=60.75 is_square=true
   pre_text push pi=47 pre_h=73.52 cur_h: 1091.56 -> 1165.08
   table push   pi=47 table_total=60.75 cur_h: 1165.08 -> 1225.83
```

| pi | pre_h (host text) | table_total | sum (현재) | max(pre, v_off+table) | 차이 |
|---:|------------------:|------------:|----------:|---------------------:|-----:|
| 33 |             98.03 |       84.93 |    182.96 |        max(98.0,90.2)=98.03 | -84.93 |
| 37 |             73.52 |       60.75 |    134.27 |        max(73.5,66.0)=73.52 | -60.75 |
| 40 |             49.01 |       38.45 |     87.46 |        max(49.0,43.7)=49.01 | -38.45 |
| 47 |             73.52 |       60.75 |    134.27 |        max(73.5,66.0)=73.52 | -60.75 |
| **합** |    294.08 |     244.88 |    538.96 |              294.08 | **-244.88** |

(v_off=5.3 px = 396 HU 환산값, 4 표 모두 동일)

### 1.2 HWP 의도 동작

Square wrap 표는 호스트 문단의 텍스트와 **같은 수직 영역** 에 배치된다 (어울림 = 본문이 표 옆으로 흐름).
HWP layout 에서 다음 문단은 `max(호스트 텍스트 하단, 표 하단)` 직후에 시작한다.

검증:
- pi=33 호스트 마지막 line_seg vpos=23430, lh=1150 → 호스트 텍스트 하단 = 24580 HU
- pi=33 표: vert_offset=396, height=5667 → 표 하단 = 17916+396+5667 = 23979 HU
- pi=34 첫 line_seg vpos = 25268
- 25268 - 24580 = 688 HU = pi=33 trailing line_spacing ✓

따라서 column 누적량은 `pre_height` (4 줄 line_advance 합 = 호스트 텍스트 + trailing) **하나만 반영하면 충분**하다 (이 케이스에서는 호스트가 표보다 큼).

### 1.3 일반화된 정답

`current_height += max(pre_height, vertical_offset_px + table_total_height)`

- 호스트 텍스트가 표보다 길면 → pre_height
- 표가 호스트 텍스트보다 길면 → vertical_offset_px + table_total_height

(현재 typeset.rs 는 둘을 **합산** 하여 양쪽 모두 누적)

### 1.4 ~14.5 px 의 정체 (Stage 1 미해결 의문 해소)

Stage 1 에서 "이론상 ~290 px 과다인데 실제 +14.5 px" 이라고 표현했다. 정확한 분석:

- 4 Square wrap 표 합산 누적: **538.96 px**
- 정답 (max) 합산: **294.08 px**
- 과다 누적 총량: **244.88 px**

페이지 14 col 0 used = 1225.83 → fix 후 col 0 used ≈ **980.95 px**.

> Stage 1 의 "+14.5 px" 는 1211.3 - 1225.83 = -14.5 의 단순 차이였음. 실제 Square wrap 의 과다 누적 (~245 px) 이 압도적으로 큰 원인.

Stage 1 보고서의 가설 B ("어딘가 보정 로직 존재") 는 **사실 아님**. 단순히 244.88 px 의 과다 누적이 1225.83 px 으로 결과적으로 14.5 px 만 본문 한계를 초과하는 우연. 페이지 14 col 0 에는 다른 문단들 (~700 px) 이 함께 있어 표면적 over-fill 이 작아 보였을 뿐이다.

### 1.5 col 1 underflow 의 원인

페이지 14 col 0 over-fill (1225.8 > 1211.3) → pi=48 fit 실패 → col 1 advance.
col 1 fit 검사로 pi=48, pi=49 만 흡수 (64.3 px 사용).
**col 1 이 빈 이유**: pi=50 이 [단나누기] (`ColumnBreakType::Column`) 또는 vpos-reset 가드에 의해 새 페이지로 강제 이동.

dump 확인:
```
=== 페이지 15 (global_idx=14, section=1, page_num=15) ===
  단 0 (items=18, used=1077.6px, hwp_used≈1155.7px, diff=-78.1px)
    PartialParagraph  pi=50  lines=0..3
    Table            pi=50 ci=0  ... wrap=Square ...
```

pi=50 도 Square wrap 표 보유. 같은 over-fill 버그로 페이지 15 col 0 도 영향받음.

fix 후 페이지 14 의 가용 공간이 늘어나 (col 0 -244.88, col 1 0 → 가용)→ pi=48..pi=49 + 추가 콘텐츠가 페이지 14 에 흡수 가능. pi=50 의 흡수 여부는 구현 후 측정 필요.

## 2. 회귀 영향 평가

### 2.1 동일 패턴 영향 케이스

| 샘플 | 페이지수 | Square wrap 표 | 영향 |
|------|--------:|--------------|------|
| `exam_kor.hwp` (대상) | 22 | pi=33,37,40,47,50,... | **본 fix 대상** |
| `exam_math.hwp` | 20 | pi=27 (146.9×114.4px) v_off=168 | fix 영향 받음, 회귀 검증 필요 |
| `21_언어_기출_편집가능본.hwp` | 15 | pi=299 (22.7×220.7px) v_off=475 | fix 영향 받음 (표가 매우 김) |
| `exam_eng.hwp` | 8 | 없음 | 영향 없음 |

### 2.2 표가 호스트 텍스트보다 큰 경우 (21언어 pi=299)

표 220.7 px (vert_offset 6.3 px 합 = 227 px) vs 호스트 텍스트 (3 줄 ≈ 75 px).

- 현재 (sum): 75 + 220.7 = 295.7 px 누적
- 제안 fix (max): max(75, 227) = 227 px 누적
- 차이: -68.7 px

이 케이스는 fix 후 column 누적이 **줄어들어** 페이지 수가 같거나 줄어들 수 있다. 회귀라기보다는 정확도 향상.

### 2.3 hwpspec.hwp 등 표+텍스트 혼합 문서

- `treat_as_char` (TAC) 표는 본 fix 의 영향 받지 않음 (`is_wrap_around_table` 조건이 `!treat_as_char` 포함)
- `wrap=TopAndBottom` (자리차지) 표도 영향 없음 (Square 만 영향)
- 따라서 회귀 위험은 **Square wrap 표를 가진 문서** 에 한정됨

## 3. 수정안 (Stage 3 구현 대상)

### 3.1 변경 파일

`src/renderer/typeset.rs::place_table_with_text` (라인 1400-1476)

### 3.2 변경 내용

```rust
fn place_table_with_text(
    &self,
    st: &mut TypesetState,
    para_idx: usize,
    ctrl_idx: usize,
    para: &Paragraph,
    table: &crate::model::table::Table,
    fmt: &FormattedParagraph,
    table_total_height: f64,
) {
    let vertical_offset = Self::get_table_vertical_offset(table);
    let total_lines = fmt.line_heights.len();
    let pre_table_end_line = if vertical_offset > 0 && !para.text.is_empty() {
        total_lines
    } else {
        0
    };

    // ★ 신규: Square wrap 식별 (engine.rs:1328 동일 시멘틱)
    let is_wrap_around_table = !table.common.treat_as_char
        && matches!(table.common.text_wrap, crate::model::shape::TextWrap::Square);

    // pre-table 텍스트 (첫 번째 표에서만)
    let is_first_table = !para.controls.iter().take(ctrl_idx)
        .any(|c| matches!(c, Control::Table(_)));

    let pre_height: f64 = if pre_table_end_line > 0 && is_first_table {
        let h = fmt.line_advances_sum(0..pre_table_end_line);
        st.current_items.push(PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: 0,
            end_line: pre_table_end_line,
        });
        h
    } else {
        0.0
    };

    // 표 배치
    st.current_items.push(PageItem::Table {
        para_index: para_idx,
        control_index: ctrl_idx,
    });

    // ★ 변경: Square wrap 표는 호스트 텍스트와 같은 y 영역 차지
    //         → max(pre_height, v_off + table_total) 만 누적
    if is_wrap_around_table && pre_height > 0.0 {
        let v_off_px = crate::renderer::hwpunit_to_px(vertical_offset, self.dpi);
        let table_bottom = v_off_px + table_total_height;
        st.current_height += pre_height.max(table_bottom);
    } else {
        // 기존 동작: pre + table 합산
        st.current_height += pre_height + table_total_height;
    }

    // post-table 텍스트 (기존 로직 유지)
    let is_last_table = !para.controls.iter().skip(ctrl_idx + 1)
        .any(|c| matches!(c, Control::Table(_)));
    let tac_table_count = para.controls.iter()
        .filter(|c| matches!(c, Control::Table(t) if t.attr & 0x01 != 0))
        .count();
    let post_table_start = if table.attr & 0x01 != 0 {
        pre_table_end_line.max(1)
    } else if is_last_table && !is_first_table {
        0
    } else {
        pre_table_end_line
    };
    let pre_text_exists = post_table_start == 0 && st.current_items.iter().any(|item| {
        matches!(item, PageItem::PartialParagraph { para_index, start_line, .. }
            if *para_index == para_idx && *start_line == 0)
    });
    let should_add_post_text = is_last_table && tac_table_count <= 1
        && !para.text.is_empty() && total_lines > post_table_start && !pre_text_exists;
    if should_add_post_text {
        let post_height: f64 = fmt.line_advances_sum(post_table_start..total_lines);
        st.current_items.push(PageItem::PartialParagraph {
            para_index: para_idx,
            start_line: post_table_start,
            end_line: total_lines,
        });
        // ★ 변경: Square wrap 의 post-text 도 동일 처리 (호스트 텍스트와 표가 같은 y)
        //   다만 post-text 는 표 뒤의 텍스트이므로 일반적으로 추가 누적이 맞다.
        //   현재 케이스에서는 vert_offset>0 이면 pre_table_end_line=total_lines 이므로
        //   total_lines > post_table_start 가 false → post-text 진입 없음.
        st.current_height += post_height;
    }

    // TAC 표: trailing line_spacing 복원 (기존 로직 유지)
    let is_tac = table.attr & 0x01 != 0;
    let has_post_text = !para.text.is_empty() && total_lines > post_table_start;
    if is_tac && fmt.total_height > fmt.height_for_fit && !has_post_text {
        st.current_height += fmt.total_height - fmt.height_for_fit;
    }
}
```

핵심 변경:
1. **신규**: `is_wrap_around_table` 식별 (engine.rs:1328 와 동일 시멘틱)
2. **변경**: Square wrap 시 `current_height += max(pre_height, v_off+table_total)`
3. **유지**: PartialParagraph + Table 의 PageItem push 는 그대로 (layout 측 렌더링용)
4. **유지**: post-text, TAC 보정은 그대로

### 3.3 예상 효과

페이지 14 col 0 used:
- 현재: 1225.83 px
- fix 후: 1225.83 - 244.88 = **980.95 px** (호스트 텍스트만 누적)

페이지 수:
- 현재: 22 페이지
- fix 후: 21 또는 그 이하 (col 0 가용 공간 ~245 px 확장 + col 1 1211.3 px 가용)

### 3.4 회귀 우려 사항

| 시나리오 | 현재 동작 | fix 후 | 영향 |
|----------|----------|--------|------|
| Square wrap, 호스트>표 (exam_kor) | 합산 (over-count) | max=호스트 | 정확 |
| Square wrap, 표>호스트 (21언어 pi=299) | 합산 (over-count) | max=표+v_off | 정확, 페이지 수 감소 |
| Square wrap, v_off=0 | 합산 (post-text 도 누적) | max | post-text 분기 분석 필요 |
| TAC 표 (treat_as_char=true) | 변경 없음 | 변경 없음 | 영향 없음 |
| TopAndBottom 표 (자리차지) | 변경 없음 | 변경 없음 | 영향 없음 |

`v_off=0` Square wrap 케이스 (현재 데이터셋에서 미발견) 는 Stage 4 회귀 테스트에서 검증.

## 4. 단계 (Stage 3 → Stage 4)

### Stage 3 — 수정 구현 + 단위 검증 (~1 시간)

1. `typeset.rs::place_table_with_text` 수정 (위 3.2 코드)
2. `cargo build --release` + `cargo test`
3. **단위 측정**:
   - `dump-pages -p 13` 으로 col 0 used 확인 (≤ 1211.3)
   - `dump-pages` 로 exam_kor 페이지 수 (22 → ?)
4. **산출물**: `mydocs/working/task_m100_439_stage3.md` (구현 diff 요약 + 측정값)

### Stage 4 — 회귀 검증 + 최종 보고 (~30 분)

1. **회귀 테스트** (페이지 수 비교):
   - `exam_math.hwp` (현재 20)
   - `21_언어_기출_편집가능본.hwp` (현재 15)
   - `exam_eng.hwp` (현재 8)
   - `basic.hwp` 등
2. `cargo test` + `cargo clippy -- -D warnings`
3. **SVG 시각 검증** (Square wrap 표 렌더링 정상 확인)
4. **산출물**: `mydocs/report/task_m100_439_report.md` + `mydocs/orders/{yyyymmdd}.md` 갱신

## 5. 검증 기준 (DoD 갱신)

- [ ] exam_kor.hwp page 14 col 0 used ≤ 1211.3 px
- [ ] exam_kor.hwp 페이지 수 ≤ 21 (목표: 21)
- [ ] `cargo test` 전부 통과
- [ ] `cargo clippy -- -D warnings` 통과
- [ ] 회귀 샘플 페이지 수: exam_math, 21언어, exam_eng, basic — 동일 또는 감소 (증가 없음)
- [ ] Square wrap 표 SVG 렌더링 시각 정상 (pi=33, 37, 40, 47 의 [A], [B] 박스 정상 표시)

## 6. 코드 변경 내역 (Stage 2)

**없음**. 임시 디버그 println 삽입했다가 모두 revert.

```
$ git diff --stat
(empty)
```

---

## 승인 요청

위 구현 계획대로 Stage 3 (코드 수정) 진행해도 될지 검토 부탁드립니다. 특히:

1. **수정 위치 (`typeset.rs::place_table_with_text`) 와 방식 (max 사용)** 동의 여부
2. **post-text 분기**: 현재 데이터셋 (v_off>0) 에서는 post-text 진입 안 함 — 추후 별도 이슈로 분리해도 무방. 본 fix 에서는 post-text 로직 변경 없음.
3. **engine.rs (legacy fallback) 도 함께 수정 여부** — 현재는 변경 없음으로 계획 (fallback 이므로). 필요 시 별도 task.
