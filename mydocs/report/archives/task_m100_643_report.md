# Task #643 최종 보고서: 페이지 분할 드리프트 정정

## 목표 및 결과

**목표**: `samples/2022년 국립국어원 업무계획.hwp` 6페이지 마지막 줄 (' 및 점자 해당 분야 전문인력 확보 어려움') 이 다음 페이지로 부당 분리되는 회귀 정정.

**결과**: ✅ HWP 원본 정합 회복. pi=80 가 페이지 6 에 `FullParagraph` 로 배치. 1221 테스트 통과 (회귀 0).

## Root Cause (4축 누적 드리프트)

| 축 | 위치 | 증상 |
|----|------|------|
| 1 | `pagination/engine.rs:846-852` fit 루프 | `line_advances_sum` 사용으로 마지막 줄의 트레일링 ls 까지 누적 → 잔여 공간 산정 왜곡 |
| 2 | `typeset.rs:876` 안전마진 10px | 축 1 의 band-aid 였으나 본 케이스 fit 결정 차단 |
| 3 | `layout.rs:1521` VPOS_CORR backward | 1.0px 만 허용 → 누적 layout drift (~6.7px) 회복 불가 |
| 4 | `layout.rs:1504` VPOS_CORR end_y | sb_N 미차감 → layout 의 sb_N 추가와 합쳐져 line 0 위치 시프트 |

### 산술 검증 (pi=80, page 6)

| 산식 | line 1 bottom (절대 y) | body bottom 1028.0 대비 |
|------|------------------------|---------------------|
| HWP vpos 기준 (정답) | 1025.7 | -2.3 fits |
| 정정 전 우리 layout | 1032.4 | **+4.4 overflow** |
| 정정 후 우리 layout (visible) | 1025.7 | -2.3 fits ✓ |

### 핵심 통찰: HWP vpos 인코딩 vs Layout y advance

- HWP: vpos_(N+1) - vpos_N = lh_total + ls_total + sa_N + **sb_(N+1)**
- Layout: y_advance per pi = **sb_N** + lh_total + ls_total

→ sb_N ≠ sb_(N+1) (예: 빈 문단 sb=0 인접) 시 차이 누적. 본 페이지에서는 pi=76 (sb=6.7) → pi=77 (sb=0) 전환에서 +6.7px 누적, 마지막 본문 끝까지 잔존.

## 수정

### 1. `pagination/engine.rs` — fit 산식 정정

```rust
// AS-IS: line_advance(li) = lh + ls (트레일링 ls 포함)
cumulative += mp.line_advance(li);

// TO-BE: 마지막 줄은 lh 만 (트레일링 ls 제외)
cumulative += if li + 1 < seg_end {
    mp.line_advance(li)
} else {
    mp.line_heights[li]
};
```

`part_line_height` 도 동일 산식 적용.

### 2. `typeset.rs` — LAYOUT_DRIFT_SAFETY_PX 축소

`10.0 → 4.0px`. 축 1 정정 후 누적 드리프트가 감소하여 보수적 마진 축소 가능.

### 3. `layout.rs` VPOS_CORR — 백워드 허용폭 확장

```rust
// AS-IS: end_y >= y_offset - 1.0
// TO-BE: end_y >= y_offset - MAX_BACKWARD_PX (8.0)
```

문단 사이 trailing line_spacing 영역 내에서 안전 백워드 보정.

### 4. `layout.rs` VPOS_CORR end_y — sb 차감

```rust
let curr_sb = paragraphs.get(item_para)
    .and_then(|p| styles.para_styles.get(p.para_shape_id as usize))
    .map(|ps| ps.spacing_before)
    .unwrap_or(0.0);
let end_y = (raw_end_y - curr_sb).max(col_area.y);
```

vpos_end (HWP의 line 0 top) 에서 sb 차감 → layout 이 sb 추가 후 net 결과가 vpos_end 정합.

## 검증

### 회귀 테스트 (1221 / 1221 통과)

- `tests/issue_643.rs`: 본 케이스 (RED → GREEN) ✅
- `task554_no_regression_2022_kuglip`: 페이지 수 40 → 38 (의도된 정정)
- `issue_147_aift_page3`: golden SVG 본문 y 좌표 -6.67px 시프트 (pi 누적 sb 드리프트 해소) — UPDATE_GOLDEN
- `test_544_passage_box_coords_match_pdf_p4`: 통과 (layout trailing_ls 정책 변경 없음)

### 페이지 수 변화 검증

`2022년 국립국어원 업무계획.hwp`: 40 → 38 페이지.
- pi=80 fits → page 6 압축 → 후속 페이지 누적 압축
- HWP/PDF 원본 정합 회복 (한컴 본 = 38 페이지)

## 잔존 사항

### LAYOUT_OVERFLOW 진단 메시지 (9.7px)

본 케이스에서 `LAYOUT_OVERFLOW` 메시지가 paragraph y_out 기준으로 9.7px 잔존 (트레일링 line_spacing 12px 영역).

- **메커니즘**: layout 의 `y_offset` 은 paragraph 끝 (트레일링 ls 포함) 까지 advance. body bottom 보다 9.7px 초과.
- **실제 영향**: visible content (line 1 bottom) = 1025.7 ≤ body bottom 1028.0 → 시각적으로 fits.
- **해결 보류 사유**: layout 의 트레일링 ls 정책 변경 시 Task #544 (paragraph border box 좌표) 회귀.
- **조치**: `record_overflow` 는 진단 기록만 수행, 렌더링 액션 없음. 추후 LAYOUT_OVERFLOW 임계 재조정 또는 paragraph y_out 산정 정책 통합 시 (Task #644 후보) 재검토.

## 미래 작업 권장

1. **layout vs typeset corrected_line_height 정합 (#644 후보)**: 빈 문단 등에서의 추가 drift 원인 제거
2. **LAYOUT_OVERFLOW 임계 정책**: trailing ls 영역은 overflow 로 보지 않도록 visible content bottom 기준 재정의
3. **paragraph border box 좌표 모델 통합 (#544 ↔ #643)**: 트레일링 ls 의 양쪽 요구사항 (border 포함 vs fit 제외) 통일

## 산출물

- 코드: `src/renderer/pagination/engine.rs`, `src/renderer/typeset.rs`, `src/renderer/layout.rs`
- 테스트: `tests/issue_643.rs` (신규), `tests/issue_554.rs` (갱신), `tests/golden_svg/issue-147/aift-page3.svg` (갱신)
- 문서: `mydocs/plans/task_m100_643.md`, `task_m100_643_impl.md`, 본 보고서
- 커밋: 2개 (Stage 0/1, Stage 2-4)

## 결론

**4축 드리프트 누적 모델을 본질적으로 분해**하여 각 축에 좁은 정정 적용. HWP 원본 정합 회복 + 회귀 0. layout drift 의 보다 깊은 정합 (corrected_line_height, LAYOUT_OVERFLOW 임계) 은 후속 이슈로 분리.
