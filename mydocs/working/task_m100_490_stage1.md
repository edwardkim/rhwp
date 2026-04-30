# Task #490 Stage 1+2 통합 완료 보고서

## 1. 변경

### `src/renderer/layout/paragraph_layout.rs` (한 곳)

`comp_line.runs.is_empty() && !tac_offsets_px.is_empty()` 분기 (Task #287 도입) 에 paragraph alignment 적용.

```rust
// [Task #490] paragraph alignment 적용. 셀에 텍스트 없이 수식만 있을 때
// (text_len=0 + ctrls=1+) 정렬이 무시되어 좌측 고정되던 결함 수정.
let line_tac_width: f64 = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= line_start_char && *pos < line_end_char)
    .map(|(_, w, _)| *w)
    .sum();
let align_offset = match alignment {
    Alignment::Center | Alignment::Distribute => {
        (available_width - line_tac_width).max(0.0) / 2.0
    }
    Alignment::Right => {
        (available_width - line_tac_width).max(0.0)
    }
    _ => 0.0,
};
let mut inline_x = effective_col_x + effective_margin_left + align_offset;
```

### `src/renderer/layout/integration_tests.rs`

`test_490_empty_para_with_tac_equation_respects_alignment` 추가:
- exam_science p1 SVG 에서 3번 표 영역(y∈[1040,1090]) 의 28 수식 group transform x 좌표 검증
- x≥380 일 때 통과 (수정 전 358.7 → 수정 후 415.5)

## 2. 검증

### 2.1 단위 테스트

- `cargo test --lib --release`: **1094 passed; 0 failed** (1093 + 신규 1)
- `cargo test --release --test svg_snapshot`: **6/6 passed**

### 2.2 광범위 byte 비교 (9 종 샘플 263 페이지)

| 샘플 | 페이지 | 차이 |
|------|------|------|
| exam_science p1 | 1 페이지 | 28/36 두 수식 x: 358.7 → 415.5 (+56.8 px) |
| exam_science p2 | 1 페이지 | 14 수식 우측 이동 (+15.8~+25.8) |
| exam_science p3 | 1 페이지 | 18 수식 우측 이동 (+8.0~+28.2) |
| exam_science p4 | 1 페이지 | 14 수식 우측 이동 (+26.4~+137.2) |
| exam_math p13 | 1 페이지 | 10 수식 우측 이동 (+2.5~+28.9) |
| kps-ai p46 | 1 페이지 | 1 수식 우측 이동 (+80.5) |
| **합계** | **6 페이지** | **59 수식 위치 정정** |

**전수 분석 결과**: 59 변경 모두 **양의 방향 (우측 이동)** — alignment 적용 방향과 일치. 음의 방향 (좌측 이동) 0 건.

```
exam_science_001:  positive 2 / negative 0   (Δ +56.8 ~ +56.8)
exam_science_002:  positive 14 / negative 0  (Δ +15.8 ~ +25.8)
exam_science_003:  positive 18 / negative 0  (Δ  +8.0 ~ +28.2)
exam_science_004:  positive 14 / negative 0  (Δ +26.4 ~ +137.2)
exam_math_012:     positive 10 / negative 0  (Δ  +2.5 ~ +28.9)
kps-ai_045:        positive 1  / negative 0  (Δ +80.5)
```

### 2.3 회귀 영역 점검

| 영역 | 결과 | 근거 |
|------|------|------|
| Justify/Left 셀 paragraph | 회귀 0 | match 의 `_ => 0.0` |
| 텍스트+수식 혼재 줄 | 회귀 0 | comp_line.runs 비어있지 않음 → 본 분기 미진입 (line 1200 케이스로) |
| 일반 paragraph | 회귀 0 | tac_offsets_px 비어있을 때 미진입 |
| Task #287 (수식 y 위치) | 회귀 0 | x 만 변경, y/height 미변경 |
| Task #489 Picture+Square wrap | 회귀 0 | 별개 분기 |

### 2.4 시각 검증 (exam_science p1)

3번 표 (이온 결합 화합물) 셀 7/11 의 28/36 수식:
- 셀 7 영역: x=336.8..478.0 (w=141)
- 셀 중앙: x=407.4
- 수정 전: x=358.7 (좌측, 중앙에서 -49)
- 수정 후: x=415.5 (중앙에서 +8 — 시각적으로 중앙 정렬)

## 3. 다음 단계

Stage 3: 최종 보고서 + orders 갱신 + merge.
