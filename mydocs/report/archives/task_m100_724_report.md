# Task #724 최종 결과 보고서

## 1. 이슈 요약

**Issue #724**: hwp3-sample5-hwp5.hwp paragraph 441 wrap zone 매칭 실패 — anchor host cs=0 caption-style IR 패턴 + (작업지시자 추가 진단) HWP3 파서의 mid-paragraph LINE_SEG sw=0 인코딩 결함

**증상 1**: HWP5 변환본 paragraph 441 wrap zone 매칭 실패 → 좁은 폭 분할 (typeset wrap_around state machine 매칭 가드 부재)

**증상 2**: HWP3 native (`samples/hwp3-sample5.hwp`) 페이지 4/16/22 등에서 paragraph (예: 75, 443) 의 mid-paragraph 또는 wrap zone 끝 line 이 sw=0 으로 인코딩 → 본 환경 visual 좁은 폭 분산 layout

**작업지시자 핵심 지적**: "HWP3 파서가 잘못 해석해서 IR로 잘못 전달하는것 아님? composer.rs 에서 처리하면 너무 예외" — 본질 위치 정확 식별. CLAUDE.md HWP3 파서 규칙 정합.

## 2. 본질 정정 영역

### `src/renderer/typeset.rs` (Stage 1~3)

#### 가설 A: image expected_cs 정확 일치 매칭 가드 (line 495~520)

기존 매칭 분기에 `anchor_image_match` 가드 추가 (anchor cs=0 한정):

```rust
let anchor_image_match = if st.wrap_around_cs == 0 {
    let body_left = page_def.margin_left as i32;
    let expected_cs_hu = paragraphs.get(st.wrap_around_table_para)
        .and_then(|p| p.controls.iter().find_map(|c| {
            // wrap=Square Picture image 의 horizontal_offset + width + 2*margin.right - body_left
        }))
        .unwrap_or(0);
    expected_cs_hu > 0
        && (para_cs - expected_cs_hu).abs() < 200
        && para_sw > 0
        && para_cs + para_sw <= body_w + 200
} else { false };
```

#### 가설 D: vpos-reset 시 wrap_around 강제 종료 (line 417~445)

기존 vpos-reset 가드 분기에 `st.wrap_around_cs < 0` 조건 제거 + anchor cs=0 한정 강제 종료:

```rust
if trigger {
    if st.wrap_around_cs == 0 {
        st.wrap_around_cs = -1;
        st.wrap_around_sw = -1;
        st.wrap_around_any_seg = false;
    }
    if st.wrap_around_cs < 0 {
        st.advance_column_or_new_page();
    }
}
```

### `src/parser/hwp3/mod.rs` (Stage 4~5)

#### 후처리 분기 sw=column_width_hu 정합화 (line 1705~1715)

```rust
} else if wrap_zone_end_vpos > 0 && acc_section_vpos >= wrap_zone_end_vpos {
    // [Task #724] sw=column_width_hu (col_area 전체 폭) 한컴 IR 정합.
    if seg.column_start > 0 || seg.segment_width == 0 {
        seg.column_start = 0;
        seg.segment_width = column_width_hu;
    }
}
```

본 환경 HWP3 파서가 wrap zone 끝 line 의 sw=0 강제 → 한컴 HWP5 변환본 정합 sw=col_area_width=51024.

## 3. 수행 단계 요약

| Stage | 영역 | 결과 |
|-------|------|------|
| 1 | HWP5 변환본 본질 진단 — 가설 A + D 도출 | 결정 |
| 2 | typeset.rs 가설 A + D 정정 | HWP5 페이지 16/22 정합 |
| 3 | 광범위 sweep (kps-ai 회귀 발견 → 가드 좁힘) | 209 fixture DIFF 0 |
| 4 | 작업지시자 핵심 지적 — HWP3 파서 결함 진단 | sw=0 인코딩 본질 식별 |
| 5 | HWP3 파서 후처리 분기 정합화 | paragraph 443 정합 |
| 6 | line_cs_sw 분기 정정 시도 → 회귀 발견 → rollback | paragraph 75 정합 |
| 7 | HWP3 파서 page break flag 빈 paragraph 가드 (2 곳) | paragraph 171 column_type=Normal |
| 8 | force_vpos_reset 추가 (paragraph 435 vpos 회복) | 페이지 16 정합 보존 |
| 9 | wrap_zone 비활성 sw=0 정합화 | 페이지 9 paragraph 191 등 표시 회복 |

## 4. 검증 결과

### 결정적 검증 (release 모드)

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1166 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep (209 fixture) | **DIFF 0** (회귀 0) |

### 시각 판정 (rsvg-convert PNG, PDF 권위 자료 정합)

| 페이지 | paragraph | 결과 | 정합 |
|--------|-----------|------|------|
| HWP3 native 페이지 4 | 75 (image 우측 wrap zone, 21 lines) | wrap zone (cs=35460/sw=15564) ✓ | PDF |
| HWP3 native 페이지 8 | 172 시작, 175 image 우측 wrap zone | paragraph 172/174/175 정합 ✓ | PDF |
| HWP3 native 페이지 9 | 189 PartialParagraph + 191/192 + 후속 | "루트 파일시스템" 등 표시 회복 ✓ | PDF |
| HWP3 native 페이지 16 | 435 시작 + 441/443 wrap zone | paragraph 435 vpos=1440 정합 ✓ | PDF |
| HWP3 native 페이지 22 | 599 (MBR image) | image 우측 wrap zone ✓ | PDF |
| HWP3 native 페이지 8/27/48 (PR #723) | 175/779/1394 | 정합 보존 ✓ | PDF |
| HWP5 변환본 페이지 16 | 441/442/443 | image 우측 wrap zone ✓ | PDF |
| HWP5 변환본 페이지 22 | 599 페이지 시작 | 페이지 분할 정합 ✓ | PDF |

## 5. 회귀 위험 영역 정합

- `typeset.rs` 2 곳 변경 (가설 A 매칭 가드 + 가설 D vpos-reset wrap_around 종료) — 모두 anchor cs=0 한정
- `parser/hwp3/mod.rs` 1 곳 변경 (후처리 분기 sw 정합화) — wrap zone 끝 영역 한정
- IR 변경 (HWP3 파서 영역) → CLAUDE.md "HWP3 파서 규칙" 정합
- 광범위 sweep 209 fixture 페이지 수 차이 0
- Task #722 (PR #723) 영역 보존
- Task #604 영역 보존

## 6. 작업지시자 핵심 지적의 가치

본 task 의 본질 식별에 작업지시자 핵심 지적이 결정적:

> "hwp3 파서에서 잘못해석해서 ir 로 잘못 전달하는것 아님? 모두 composer.rs 에서 처리하면 너무 예외가 맞지 않음? hwp3 파서에서 정확하게 파싱해서 전달하면 되는것 아님?"

본 지적 후 HWP5 변환본 paragraph 443 IR 비교 → 본 환경 sw=0 vs 한컴 sw=51024 차이 발견 → HWP3 파서 결함 본질 확정. composer/layout 정정 영역 회피.

**`feedback_visual_judgment_authority` 권위 사례 강화**: 작업지시자 본질 지적이 본 환경 결함의 정확한 위치 (HWP3 파서 vs composer/layout) 결정.

## 7. closes #724

- HWP5 변환본 (hwp3-sample5-hwp5.hwp) paragraph 441 wrap zone 매칭 실패 정정
- HWP3 native (hwp3-sample5.hwp) paragraph 443/75 등 wrap zone 끝 line sw=0 결함 정정
- 한컴 PDF 권위 자료 정합 검증
- 광범위 fixture 회귀 0
