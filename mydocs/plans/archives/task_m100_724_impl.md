# Task #724 구현 계획서

## 개요

Issue #724 "hwp3-sample5-hwp5.hwp paragraph 441 wrap zone 매칭 실패" 정정. 가설 A (image expected_cs 정확 일치 가드) + 가설 D (wrap_around vpos-reset 강제 종료) 결합.

## Stage 1: 본질 진단 (완료)

### 진단 결과 (`mydocs/working/task_m100_724_stage1.md` 작성 예정)

- **가설 A 검증**: paragraph 440 image (x_offset=3992, width=21356, margin=852) → expected_cs=22800 = paragraph 441/442/443 cs 정확 일치
- **가설 D 발견**: typeset.rs:419~ Task #321 vpos-reset 가드 존재. 그러나 `st.wrap_around_cs < 0` 조건 (Task #362) 으로 wrap_around active 시 무시
- **페이지 분할 회귀 본질**: Stage 9 broad 가드 → paragraph 442/443 wrap_around 매칭 → paragraph 599 (vpos=0) 시점에도 wrap_around active → vpos-reset 가드 무시 → 페이지 분할 왜곡

## Stage 2: 본질 정정 — 가설 A + D 결합

### 정정 영역

#### `src/renderer/typeset.rs:484~545` (가설 A: image expected_cs 정확 일치 매칭 가드)

기존 매칭 분기 (`exact match || any_seg_matches || sw0_match`) 에 image expected_cs 가드 추가:

```rust
// [Task #724] HWP5 변환본 case: anchor host 의 wrap=Square image 위치/폭/margin 으로
// expected_cs 정확 계산 후 para_cs 일치 확인.
// expected_cs = (image_x_offset + image_width + 2 * image_margin) - col_area.x
let anchor_image_match = if st.wrap_around_cs == 0 {
    let expected_cs_hu = paragraphs.get(st.wrap_around_table_para)
        .and_then(|p| p.controls.iter().find_map(|c| {
            let cm = match c {
                Control::Picture(pic) => Some(&pic.common),
                Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(pic) = s.as_ref() {
                    Some(&pic.common)
                } else { None },
                _ => None,
            };
            cm.filter(|cm| !cm.treat_as_char
                && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square))
                .map(|cm| {
                    let body_left = page_def.margin_left as i32;
                    cm.horizontal_offset as i32 + cm.width as i32
                        + 2 * cm.margin.right as i32 - body_left
                })
        }))
        .unwrap_or(0);
    expected_cs_hu > 0
        && (para_cs - expected_cs_hu).abs() < 200
        && para_sw > 0
        && para_cs + para_sw <= body_w + 200
} else { false };
if (para_cs == st.wrap_around_cs && para_sw == st.wrap_around_sw)
    || (any_seg_matches && (is_empty_para || st.wrap_around_any_seg))
    || sw0_match
    || anchor_image_match {
```

#### `src/renderer/typeset.rs:419~440` (가설 D: vpos-reset wrap_around 강제 종료)

Task #321 vpos-reset 가드 분기에 wrap_around 활성 케이스도 처리. wrap_around active + vpos-reset trigger 발동 시 wrap_around 강제 종료 + page break.

```rust
// [Task #724] vpos-reset 가드 발동 시 wrap_around 강제 종료.
// HWP5 변환본 case: paragraph 442/443 wrap_around 매칭 후 paragraph 444 (cs=0) 에서
// wrap_around 종료되어야 하지만 일부 case 에서 종료 늦어짐 → 후속 paragraph (예: 599)
// vpos=0 hint 시점에도 wrap_around active → 페이지 분할 위반.
// 정정: vpos-reset trigger 시 wrap_around 강제 종료 (cs/sw reset = -1).
if para_idx > 0 && !st.current_items.is_empty() {
    let prev_para = &paragraphs[para_idx - 1];
    let curr_first_vpos = para.line_segs.first().map(|s| s.vertical_pos);
    let prev_last_vpos = prev_para.line_segs.last().map(|s| s.vertical_pos);
    if let (Some(cv), Some(pv)) = (curr_first_vpos, prev_last_vpos) {
        let trigger = if st.col_count > 1 {
            cv < pv && pv > 5000
        } else {
            cv == 0 && pv > 5000
        };
        if trigger {
            // [Task #724] wrap_around active 시 강제 종료
            if st.wrap_around_cs >= 0 {
                st.wrap_around_cs = -1;
                st.wrap_around_sw = -1;
                st.wrap_around_any_seg = false;
            }
            st.advance_column_or_new_page();
        }
    }
}
```

기존 `st.wrap_around_cs < 0` 가드 (Task #362) 조건 제거.

### 정정 원칙

- **케이스 가드 명시**: 가설 A 는 `st.wrap_around_cs == 0` (HWP5 변환본 caption-style anchor) 한정 + image expected_cs 정확 일치
- **회귀 위험 좁힘**: 가설 D 는 vpos-reset 발동 시 wrap_around 강제 종료 — 일반 케이스 영향 없음 (vpos-reset 자체가 paragraph vpos=0 + prev>5000 한정)
- **본질 룰**: 한컴 PDF 권위 자료 정합

### 검증 절차

1. **결정적 검증** (release 모드)
   - cargo test --lib --release 회귀 0 (1166 passed)
   - cargo clippy --release 신규 경고 0
2. **시각 검증** (rsvg-convert PNG)
   - HWP5 변환본 페이지 16 paragraph 441/442/443 wrap zone 정합
   - HWP5 변환본 페이지 21/22 paragraph 분할 정합 (paragraph 599 페이지 22 시작)
   - HWP3 native 페이지 8/27/48 (PR #723) 정합 보존
3. **광범위 페이지네이션 sweep**
   - 209 fixture 페이지 수 차이 0 검증

### 정정 산출물

- 영향 코드 변경 (`src/renderer/typeset.rs` 2 곳)
- `mydocs/working/task_m100_724_stage2.md` 단계별 보고서

### 승인 요청

Stage 2 정정 결과 + 시각 판정 통과 후 Stage 3 진행 승인.

## Stage 4: HWP3 native 페이지 16 paragraph 443 ls[2~] 본질 진단

### 목표

HWP3 native (`samples/hwp3-sample5.hwp`) 페이지 16 paragraph 443 의 mid-paragraph LINE_SEG cs/sw=0 처리 결함 본질 식별.

### 진단 절차

1. **paragraph 443 IR 확인**: ls[0/1] cs=21096/sw=29928 (wrap zone), ls[2~6] cs=0/sw=0 (col_area 전체)
2. **본 환경 layout 결과**: ls[2~6] 좁은 폭 분산 (한컴 위반)
3. **composer 의 ComposedLine 생성 추적**: LINE_SEG.sw=0 시 ComposedLine.segment_width 처리
4. **paragraph_layout 의 wrap_anchor 처리**: line별 cs/sw 적용 시 sw=0 fallback 분기

### 진단 산출물

`mydocs/working/task_m100_724_stage4.md` — 본질 진단 결과 + Stage 5 정정 방향 결정.

### 승인 요청

Stage 4 진단 결과 + Stage 5 정정 방향 승인 요청.

## Stage 5: HWP3 native 페이지 16 paragraph 443 본질 정정

### 목표

Stage 4 진단에 따라 정정 위치/방법 결정.

### 검증 절차

1. **결정적 검증** (release 모드): cargo test/clippy
2. **시각 검증** (rsvg-convert PNG)
   - HWP3 native 페이지 16 paragraph 443 ls[2~] col_area 전체 폭 정합
   - HWP3 native 페이지 8/27/48 (PR #723) 정합 보존
   - HWP5 변환본 페이지 16/22 (Task #724 Stage 1~3) 정합 보존
3. **광범위 페이지네이션 sweep** 회귀 0

### 정정 산출물

- 영향 코드 변경
- `mydocs/working/task_m100_724_stage5.md` 단계별 보고서

### 승인 요청

Stage 5 정정 결과 + 시각 판정 통과 후 Stage 6 진행 승인.

## Stage 6: 광범위 회귀 sweep + 최종 검증

### 검증 절차

1. **광범위 페이지네이션 sweep** (209 fixture)
2. **결정적 검증** (release 모드)
3. **시각 판정 게이트** (작업지시자)

### 산출물

- `mydocs/report/task_m100_724_report.md` 최종 보고서
- 오늘할일 (`orders/`) 갱신
- PR 준비

## 회귀 위험 영역 좁힘 원칙

- 가설 A 가드: anchor cs=0 case 한정 + image expected_cs 정확 일치 (tolerance 200 HU = 0.7mm)
- 가설 D 가드: vpos-reset trigger 발동 시 wrap_around 강제 종료 (이미 보수적 가드)
- IR 무수정
- Task #722 영역 (HWP3 native) 보존

## 의존성

- 선행 의존: PR #723 (Task #722) — 본 task 가 PR #723 위 분기
- 후행 의존: 없음

## 작업지시자 결정 영역

Stage 2 정정 코드 작성 진행 승인 요청.
