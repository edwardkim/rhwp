# Task #1007 Stage 2 — Fix 후보 평가 + 선정

이슈: [#1007](https://github.com/edwardkim/rhwp/issues/1007)
Stage 1: [`task_m100_1007_stage1.md`](task_m100_1007_stage1.md)

## 1. Fix 후보

### 후보 X — Cross-paragraph vpos reset 감지 (가장 유망)

**핵심**: pagination engine 에서 paragraph 추가 시 prev/curr vpos 비교.

```rust
// Pseudo:
if let Some(prev_pi) = prev_para_idx {
    if let (Some(prev_para), Some(curr_para)) = (paragraphs.get(prev_pi), paragraphs.get(curr_pi)) {
        let prev_last_vpos = prev_para.line_segs.last().map(|ls| ls.vertical_pos);
        let curr_first_vpos = curr_para.line_segs.first().map(|ls| ls.vertical_pos);
        if let (Some(prev), Some(curr)) = (prev_last_vpos, curr_first_vpos) {
            // page break 감지: prev > THRESHOLD AND curr < THRESHOLD
            let page_height_hu = layout.body_area_height_in_hu();
            if prev > page_height_hu / 2 && curr < page_height_hu / 4 {
                trigger_page_break();
            }
        }
    }
}
```

**Pros**:
- HWP3 변환본 의 page break 시그널 (vpos reset) 정확 catch
- 다른 자동 page-fill 알고리즘과 결합 가능

**Cons**:
- THRESHOLD 결정 필요 (false positive 가능)
- pagination engine 의 main loop 수정 필요

### 후보 Y — variant 시 `respect_vpos_reset` 자동 활성화

**핵심**: `is_hwp3_variant=true` 시 `respect_vpos_reset` option auto-enable.

**Pros**: 단순 (옵션 default 변경만)

**Cons**:
- 기존 `respect_vpos_reset` 로직은 paragraph 내 (line_segs[i].vpos==0 for i>0) 만 catch
- cross-paragraph case (본 issue) 는 미해결
- **X 와 결합 필요**

### 후보 Z — Empty paragraph 후 vpos reset 만 감지

**핵심**: pi=87 이 (빈) paragraph 라는 특수 패턴 활용. variant + 빈 paragraph 후 vpos reset 시만 page break.

**Pros**: 가장 specific, false positive risk 최소

**Cons**: 다른 패턴 (예: 비어있지 않은 paragraph 후 vpos reset) 미해결

## 2. 선정 — X + Y 결합

### 선정 이유

1. **본 issue 의 직접 해결**: pi=87 → pi=88 cross-paragraph vpos reset 을 catch (X)
2. **variant 한정 안전**: variant 일 때만 적용하여 일반 HWP5 회귀 차단 (Y)
3. **단계적 적용 가능**: X 의 THRESHOLD 를 보수적으로 시작 (False positive 차단)

### 구현 위치

`src/renderer/pagination/engine.rs` 의 paragraph 추가 loop (line ~280-340 부근):
- prev_pi → curr_pi 변경 시점에 vpos reset 감지
- variant 일 때만 활성화 (Document::is_hwp3_variant 전달 필요)
- 감지 시 `st.start_new_page()` 또는 동등 메커니즘 트리거

### THRESHOLD

- `prev_vpos > body_area_height_in_hu * 0.5` (페이지 절반 이상)
- AND `curr_vpos < body_area_height_in_hu * 0.25` (페이지 1/4 이내)

이 조건은 page break 의 명확한 시그널 — 대부분의 일반 paragraph 흐름은 이 패턴을 보이지 않음.

### 추가 가드

- prev paragraph 가 빈 paragraph 일 때만 적용 (Z 의 specificity 흡수) — 더 안전
- 또는 prev paragraph 의 column_break/page_break flag 와 결합

## 3. 회귀 risk 분석

| Sample 종류 | 영향 | 회귀 risk |
|------------|------|----------|
| sample16-hwp5 (변환본) | ✓ 의도 효과 | 정합 개선 |
| 다른 HWP3 변환본 | ✓ 의도 효과 | 정합 개선 |
| 일반 HWP5 (variant=false) | 미적용 | 0 |
| HWPX | variant 식별 시 적용 | 휴리스틱 검증 필요 |

## 4. 단위 검증 시나리오 (Stage 3)

1. `samples/hwp3-sample16-hwp5.hwp` page 3 → pi=88 이 page 4 로 이동
2. 페이지 수 변동 (62 → 64 페이지 ?)
3. 다른 변환본 sample (있다면) 페이지 분할 변동

## 5. Stage 3 진입 계획

1. `src/renderer/pagination/engine.rs` 에 cross-paragraph vpos reset 감지 추가
2. Document::is_hwp3_variant 를 pagination engine 까지 전달
3. THRESHOLD + 추가 가드 (prev empty para) 구현
4. 단위 검증: sample16-hwp5 page 3-4 한컴 정합
5. Stage 3 보고서

## 6. 잔존 의문

- 한컴이 정확히 같은 detection 로직을 사용하는지 미상 (reverse-engineering)
- 다른 변환본에서 다른 false positive 발생 가능 — Stage 4 sweep 필수
- THRESHOLD 조정 필요 가능 (50% / 25% 가 적절한지 검증)
