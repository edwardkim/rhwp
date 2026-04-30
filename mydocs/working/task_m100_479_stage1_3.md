# Task M100 #479 Stage 1~3 통합 보고서

## Stage 1: 베이스라인 측정

| 샘플 | 페이지 수 (Before) | 비고 |
|------|------------------|------|
| 21_언어_기출_편집가능본.hwp | 15 | 페이지 12 단 0 used=1012px |
| 2010-01-06.hwp | 6 | |
| exam_kor.hwp | 20 | |
| exam_eng.hwp | 8 | |
| k-water-rfp.hwp | 28 | |
| hwpspec.hwp | 177 | |
| synam-001.hwp | 35 | |

골든 SVG 6 통과, 단위 테스트 1078 통과.

## Stage 2: 옵션 A 적용

### 변경 1: typeset.rs:802

```rust
// Before
let lines_total: f64 = line_heights.iter().zip(line_spacings.iter())
    .map(|(h, s)| h + s).sum();
let total_height = spacing_before + lines_total + spacing_after;

// After (#479)
let n = line_heights.len();
let lh_sum: f64 = line_heights.iter().sum();
let inter_ls_sum: f64 = line_spacings.iter().take(n.saturating_sub(1)).sum();
let total_height = spacing_before + lh_sum + inter_ls_sum + spacing_after;
let height_for_fit = total_height.max(0.0);
```

paragraph 영역 = `lh_sum + (n-1)*ls + sb + sa` — HWP vpos `(last_vpos + last_lh) - first_vpos` 와 정합.

### 변경 2: paragraph_layout.rs:2557

```rust
let is_full_paragraph_end = line_idx + 1 >= end && end >= composed.lines.len();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none() {
    // 셀 외부 paragraph 의 마지막 줄: trailing line_spacing 제외
    y += line_height;
} else {
    y += line_height + line_spacing_px;
}
```

**셀 외부 paragraph만** 적용 — 셀 내부 paragraph는 기존 동작 유지(셀 안 paragraph 사이 spacing 정상).

## Stage 3: 회귀 검증

### 페이지 수 변화

| 샘플 | Before | After | 차이 |
|------|--------|-------|------|
| 21_언어_기출 | 15 | 15 | 0 |
| 2010-01-06 | 6 | 6 | 0 |
| exam_kor | 20 | 20 | 0 |
| exam_eng | 8 | 8 | 0 |
| **k-water-rfp** | 28 | **26** | **-2** |
| **hwpspec** | 177 | **172** | **-5** |
| synam-001 | 35 | 35 | 0 |

페이지 수 감소는 paragraph 누적이 정확해져 더 많은 paragraph 가 한 페이지에 들어가는 결과.

### TYPESET_DRIFT 검증 (samples/aift.hwp 페이지 3)

| paragraph | diff (Before) | diff (After) |
|-----------|--------------|--------------|
| pi=0~9 | +9.5 (모두) | **+0.0 (모두)** ✓ |

**모든 paragraph에서 fmt_total = vpos_h** — HWP vpos 와 완전 정합.

### 핵심 회귀 케이스: samples/21_언어_기출_편집가능본.hwp 페이지 12

| 측정 | Before | After |
|------|--------|-------|
| 23번 박스 y | 1166.0 | **1040.2** (-125.8 ✓) |
| 답안 ①~⑤ 위치 | 단 끝 부근 | 정상 위치 |

페이지 12 시각: 22번 + 답안 + 23번 + 답안 모두 정상 배치 — 사용자 보고 핵심 증상 해결.

### 골든 SVG 갱신

옵션 A 결과가 HWP vpos 정합 — 4 골든 SVG (form-002, issue-147, issue-157, issue-267) 갱신:
- `UPDATE_GOLDEN=1 cargo test --release --test svg_snapshot`
- 갱신 후 6/6 통과

issue-157 표 셀 안 텍스트 회귀 (셀 내부 trailing_ls 제외 회귀)는 paragraph_layout.rs 의 `cell_ctx.is_none()` 조건으로 차단.

### 통합 테스트 회귀 1건 + 정정

`tests/issue_418.rs` `hwpspec_page20_no_duplicate_image_emit`:
- pi=83/86/89 가 페이지 20 → 페이지 18 로 이동 (paragraph 누적 정확화 결과)
- 테스트 페이지 인덱스 19 → 17 갱신 + 함수명 `hwpspec_no_duplicate_image_emit` 로 변경
- 테스트 의도 (이미지 중복 emit 검출) 유지

### 전체 회귀 결과

```
cargo test --release
test result: ok. 1078 passed; 0 failed; 1 ignored; (lib + 모든 통합)
```

## 다음 단계

Stage 4 — 광범위 샘플 시각 검증 (필요 시) + Stage 5 — 최종 보고서 + 커밋.
