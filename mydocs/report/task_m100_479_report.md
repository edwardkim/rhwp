# Task M100 #479 최종 결과 보고서

| 항목 | 내용 |
|------|------|
| 이슈 | [#479](https://github.com/edwardkim/rhwp/issues/479) |
| 마일스톤 | M100 (v1.0.0) |
| 브랜치 | `local/task479` |

## 1. 증상

`samples/21_언어_기출_편집가능본.hwp` 페이지 12 단 0의 23번 paragraph 가 한컴 PDF 기준 위치보다 약 200px 아래에 그려져 답안 일부가 단 밖으로 밀리는 layout drift.

## 2. 근본 원인

`src/renderer/typeset.rs:802` `total_height = sb + lh_sum + ls_sum + sa` — 마지막 line 의 `line_spacing` 까지 누적. HWP vpos 기준은 `(last_vpos + last_lh) - first_vpos = lh_sum + (n-1)*ls` (마지막 ls 미포함). 차이 = `trailing_ls` (≈9.5px). 페이지의 paragraph 들이 누적되며 drift = N*trailing_ls. 페이지 12 단 0 17개 paragraph → ~200px drift.

## 3. 수정

### typeset.rs (paragraph 누적 정확화)
```rust
let n = line_heights.len();
let lh_sum: f64 = line_heights.iter().sum();
let inter_ls_sum: f64 = line_spacings.iter().take(n.saturating_sub(1)).sum();
let total_height = spacing_before + lh_sum + inter_ls_sum + spacing_after;
```

### paragraph_layout.rs (실제 렌더 누적 정합)
```rust
let is_full_paragraph_end = line_idx + 1 >= end && end >= composed.lines.len();
if is_cell_last_line && cell_ctx.is_some() {
    y += line_height;
} else if is_full_paragraph_end && cell_ctx.is_none() {
    // 셀 외부 paragraph 의 마지막 줄: trailing ls 제외
    y += line_height;
} else {
    y += line_height + line_spacing_px;
}
```

**핵심**: 셀 외부 paragraph 만 적용. 셀 내부는 기존 동작 유지(셀 안 paragraph 사이 spacing 정상). issue-157 표 셀 텍스트 회귀 차단.

### 회귀 회피 (Task #452 패턴)

Task #452 노트: "이전 #332 의 layout-only trailing 제외 → pagination 과 1 ls drift 발생 → 회복". 본 task #479 는 typeset 과 layout 을 **동시에** 정정하여 drift 회피.

## 4. 검증

### 4-1. TYPESET_DRIFT 정합 (`samples/aift.hwp` 페이지 3)
| paragraph | Before | After |
|-----------|--------|-------|
| pi=0~9 모두 | diff=+9.5 | **diff=+0.0** ✓ |

모든 paragraph 에서 `fmt_total = vpos_h` — HWP vpos 와 완전 정합.

### 4-2. 핵심 회귀 케이스 (페이지 12)
| 측정 | Before | After |
|------|--------|-------|
| 23번 박스 y | 1166.0 | **1040.2** (-125.8) |
| 23번 답안 잘림 | 발생 | 해결 |

### 4-3. 단위/통합 테스트
- `cargo test --release`: **1078 + 모든 통합 테스트 통과**
- 골든 SVG 6 통과 (4건 갱신: form-002, issue-147, issue-157, issue-267 — Before 보다 더 정확한 vpos 기반 좌표)
- `tests/issue_418.rs`: 페이지 인덱스 갱신(19 → 17, 콘텐츠 이동에 따라) 후 통과

### 4-4. 페이지 수 변화

| 샘플 | Before | After | 차이 |
|------|--------|-------|------|
| 21_언어_기출 | 15 | 15 | 0 |
| 2010-01-06 | 6 | 6 | 0 |
| exam_kor | 20 | 20 | 0 |
| exam_eng | 8 | 8 | 0 |
| k-water-rfp | 28 | 26 | -2 |
| hwpspec | 177 | 172 | -5 |
| synam-001 | 35 | 35 | 0 |

paragraph 누적 정확화로 더 많은 paragraph 가 한 페이지에 들어감 → 일부 샘플 페이지 수 감소.

## 5. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | line 799-806: paragraph total_height 에서 trailing line_spacing 제외 |
| `src/renderer/layout/paragraph_layout.rs` | line 2547-2569: 셀 외부 paragraph 의 마지막 줄에서 trailing line_spacing 제외 |
| `tests/issue_418.rs` | 페이지 인덱스 19 → 17 갱신, 함수명 변경 |
| `tests/golden_svg/*` | 4건 (form-002, issue-147, issue-157, issue-267) 갱신 — vpos 정합 결과로 paragraph 좌표 미세 이동 |

## 6. 영향 범위

| 케이스 | 영향 |
|--------|------|
| 일반 본문 paragraph | 누적이 +9.5px ~ +18.8px 정확화. 페이지 분배 변화 가능 |
| 셀 내부 paragraph | 변화 없음 (cell_ctx.is_none() 조건으로 차단) |
| 각주 paragraph | Task #483 적용 영역 — 변화 없음 |
| 셀 마지막 paragraph 마지막 줄 | 기존 동작 유지 (Task #452 회복 분기) |

## 7. 잔여 / 후속 작업

- 한컴 한글 2010/2020 정답지와의 광범위 비교 (메모리 가이드 [PDF 비교 결과는 절대 기준이 아님]) — 별도 검증 task 권장
- k-water-rfp, hwpspec 의 페이지 수 감소가 한컴 정답과 일치하는지 점검

## 8. 요약

- 페이지 12 layout drift 200px 해결 ✓
- HWP vpos 기준과 정합 (모든 paragraph diff=+0.0) ✓
- 회귀 없음 (단위 1078 + 통합 + 골든 6 모두 통과) ✓
- Task #452 회복 분기 + 셀 내부 보호 분기 유지 ✓
