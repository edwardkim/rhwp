# Task #724 Stage 7~9 단계별 보고서 — HWP3 파서 페이지 break / sw 정합화

## 개요

작업지시자 핵심 지적 ("HWP3 파서가 잘못해석해서 IR 로 잘못 전달, document IR 표준 확대 적용 후 HWP3 파서와 Document IR 관계 중심 점검") 에 따라 HWP3 파서 본질 정정.

## 1. 결함 본질

페이지 7/8 경계 결함 진단 (사용자 시각 판정):
- **HWP3 native paragraph 171 column_type=Page** ([쪽나누기]) — HWP3 파서가 paragraph 170 의 flags & 0x02 를 paragraph 171 column_type=Page 으로 변환
- **HWP5 변환본 paragraph 171 column_type=Normal** — 한컴 IR 정합 (페이지 분할은 자연 흐름)

HWP3 파서가 HWP5 변환본 IR 표준과 다른 변환 결과 → typeset 페이지 분할 결함.

## 2. Stage 7 정정 — `src/parser/hwp3/mod.rs` 빈 paragraph 가드 (2 곳)

```rust
// prev_para_had_flags_break 분기 (line 1574)
if prev_para_had_flags_break {
    let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
    if !is_empty_no_ctrl {
        para.column_type = ColumnBreakType::Page;
    } else {
        force_vpos_reset = true;
    }
}

// line_info.break_flag 분기 (line 1590)
if let Some(first_line) = line_infos.first() {
    if first_line.break_flag & 0x8001 == 0x8001 {
        let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
        if !is_empty_no_ctrl {
            para.column_type = ColumnBreakType::Page;
        } else {
            force_vpos_reset = true;
        }
    }
}
```

빈 paragraph (text_len=0 + controls=0) + page break flag → column_type=Normal + force_vpos_reset=true.

## 3. Stage 8 정정 — vpos reset 보존

```rust
// vpos reset 분기 (line 1614)
if matches!(para.column_type, ColumnBreakType::Page) || force_vpos_reset {
    acc_section_vpos = 0;
    wrap_zone_end_vpos = 0;
}
```

paragraph 171 column_type=Normal 유지 + vpos reset 강제 (paragraph 435 vpos=1440 정합 회복).

## 4. Stage 8 진단 — 페이지 9 SVG 출력 누락 발견

paragraph 191 ("루트 파일시스템") + paragraph 192 SVG 출력 누락. 다른 paragraph 들 y=1064 페이지 영역 밖.

dump-pages 결과는 정상 (paragraph 191 vpos=7200) 이지만 SVG 출력 결함.

본질: HWP3 파서가 페이지 break 후 paragraph (paragraph 189 ls[3~6], paragraph 190/191) 의 sw=0 으로 인코딩. 한컴 HWP5 변환본 IR 정합 sw=51024 (col_area 전체 폭).

## 5. Stage 9 정정 — wrap_zone 비활성 sw=0 정합화

```rust
} else if wrap_zone_end_vpos == 0 {
    // [Task #724 Stage 9] wrap zone 비활성 + cs=0/sw=0 인 case
    // sw=column_width_hu 정합화 — 한컴 HWP5 변환본 IR 정합 (sw=51024).
    if seg.column_start == 0 && seg.segment_width == 0 {
        seg.segment_width = column_width_hu;
    }
}
```

후처리 분기에 wrap_zone 비활성 case 추가. paragraph 189 ls[3~6], paragraph 190/191 등 페이지 break 후 paragraph 의 sw 정합화.

## 6. IR 정합 검증

| paragraph | Stage 7~8 적용 후 | Stage 9 적용 후 | HWP5 변환본 정합 |
|-----------|-------------------|-----------------|------------------|
| 189 ls[3~6] | cs=0/sw=0 | cs=0/**sw=51024** | cs=0/sw=51024 ✓ |
| 191 ls[0] | cs=0/sw=0 | cs=0/**sw=51024** | cs=0/sw=51024 ✓ |
| 435 ls[0~2] | vpos=221760 (절대) | vpos=1440 (페이지 안) | vpos=1440 ✓ |

## 7. 시각 판정 (rsvg-convert PNG)

- **stage9_p9_native.png**: paragraph 189~ + 191 "루트 파일시스템" + 192 ✓
- **stage9_p16_native.png**: paragraph 435 시작 + paragraph 441 image 우측 wrap zone ✓
- **stage9_p8_native.png**: paragraph 172 시작 + paragraph 175 wrap zone ✓

페이지 8/9/16 모두 한컴 정합 회복.

## 8. 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo test --lib --release` | **1166 passed; 0 failed** (회귀 0) |
| `cargo clippy --release` | 신규 경고 0 |
| 광범위 sweep (209 fixture) | **DIFF 0** (회귀 0) |

## 9. 회귀 위험 영역

- `src/parser/hwp3/mod.rs` 3 곳 변경 (page break flag 가드 2개 + sw 정합화 1개)
- HWP3 파서 한정 (composer/layout 미수정)
- 빈 paragraph 가드 좁힘 (text_len=0 + controls=0 한정)
- sw=0 정합화 좁힘 (cs=0 + sw=0 한정, wrap_zone 비활성)
- 광범위 sweep 회귀 0

## 10. Stage 10 진행 승인 요청

광범위 sweep + 시각 판정 통과. 최종 결과 보고서 갱신 + commit + PR 진행 승인 요청.
