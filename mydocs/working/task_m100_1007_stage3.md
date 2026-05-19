# Task #1007 Stage 3 — Fix 적용 보고서

이슈: [#1007](https://github.com/edwardkim/rhwp/issues/1007)
Stage 1/2: [`stage1`](task_m100_1007_stage1.md), [`stage2`](task_m100_1007_stage2.md)

## 1. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/pagination.rs` | `PaginationOpts::is_hwp3_variant` 필드 추가 |
| `src/renderer/pagination/engine.rs` | `paginate_with_measured_opts` 에 variant cross-paragraph vpos reset 감지 |
| `src/renderer/typeset.rs` | `typeset_section_with_variant` 신규 메서드 + cross-paragraph vpos reset 감지 |
| `src/document_core/queries/rendering.rs` | `is_hwp3_variant` 전달 |

## 2. 핵심 fix 로직

```rust
// 조건:
//   1. variant document (is_hwp3_variant=true)
//   2. prev paragraph 가 빈 paragraph (text empty + controls empty)
//   3. prev_last_vpos > body_height_hu × 0.5
//   4. curr_first_vpos < body_height_hu × 0.25
let mut variant_vpos_reset_break = false;
if is_hwp3_variant && body_height_hu > 0 {
    if let Some(prev_pi) = variant_prev_para_idx {
        if let Some(prev_para) = paragraphs.get(prev_pi) {
            let prev_is_empty = prev_para.text.chars().all(|c| c.is_whitespace())
                && prev_para.controls.is_empty();
            if prev_is_empty {
                let prev_last = prev_para.line_segs.last().map(|ls| ls.vertical_pos).unwrap_or(0);
                let curr_first = para.line_segs.first().map(|ls| ls.vertical_pos).unwrap_or(0);
                if prev_last > body_height_hu / 2 && curr_first < body_height_hu / 4 {
                    variant_vpos_reset_break = true;
                }
            }
        }
    }
}
```

## 3. 단위 검증

### 3-1. sample16-hwp5 page 3 확인

Fix 전:
```
페이지 3: pi=69 ~ pi=89 (section 1 + section 2 시작 packed)
```

Fix 후:
```
페이지 3: pi=69 ~ pi=87 (section 1 만)
페이지 4: pi=88 "(2) 주전산센터..." 부터 시작
```

### 3-2. 페이지 수 변동

| 파일 | Fix 전 | **Fix 후** | 비고 |
|------|--------|-----------|------|
| hwp3-sample16-hwp5.hwp | 62 | **67** (+5) | variant, fix 적용 |
| hwp3-sample16.hwp (HWP3) | 64 | 64 | variant=false, 영향 없음 |
| exam_kor.hwp | 20 | 20 | 일반 HWP5, 영향 없음 |
| aift.hwp | 74 | 74 | 일반 HWP5, 영향 없음 |
| biz_plan.hwp | 6 | 6 | 일반 HWP5, 영향 없음 |

### 3-3. variant break trigger 통계

sample16-hwp5 에서 38 개 위치에서 cross-paragraph vpos reset 감지:
- 대부분 명백한 새 섹션 시작 ("Ⅲ. 제안요구 사항", "Ⅳ. 프로젝트 과업범위", "Ⅴ. 도입장비 내역서", "Ⅵ. 공사 정보화 현황" 등)
- 38 trigger 중 5 만 새 페이지 추가 (나머지 33 은 기존 page-fill 이 이미 break 한 위치 — 중복 무해)

**Net +5 페이지** (62 → 67):
- pi=88 "(2) 주전산센터..." 등 5 곳에서 사용자 보고 issue 해결

## 4. 검증

- `cargo build --release` ✓
- `cargo test --release --lib`: **1303 passed**, 0 failed ✓
- `cargo clippy --release -- -D warnings`: 0 warnings ✓
- WASM 빌드: ✓ (4.83 MB, 14:53)

## 5. 잔존

- 한컴 viewer 의 정확한 페이지 수 (sample16-hwp5) 사용자 시각 판정 필요
  - Task #998 baseline 64 vs 우리 fix 후 67 — 한컴 정합 확인
- Other variant samples 검증 (없을 수 있음 — sample16-hwp5 가 유일한 known variant)

## 6. Stage 4 진입 계획

- 모든 HWP3 sample (sample10/12/14/16/18/19) 페이지 수 sweep
- 모든 HWP5 sample (exam_*, aift, biz_plan, 통합재정통계, 복학원서) 페이지 수 sweep
- 시각 회귀 확인 (golden SVG 비교)
- 작업지시자 시각 판정
