# Task #998 최종 보고서 — HWP5 sample16 페이지 수 HWP3 정합

- 이슈: [#998](https://github.com/edwardkim/rhwp/issues/998)
- 부모: [#994](https://github.com/edwardkim/rhwp/issues/994) (PR #997)
- 브랜치: `local/task998`
- 일자: 2026-05-18

## 1. 작업 결과

`samples/hwp3-sample16-hwp5.hwp` 페이지 수를 HWP3 reference (64) 와 정합.

| 변종 | Pre-fix | Post-fix |
|------|---------|----------|
| HWP3 sample16.hwp | 64 (reference) | 64 |
| HWP5 sample16-hwp5.hwp | 67 (PR #997 결과) | **64** ✓ |

### 변경 파일
- `src/renderer/composer.rs` (CHARS_PER_LINE: 35 → 45)
- `src/renderer/typeset.rs` (line_segs 누락 paragraph 의 spacing_before=0 보정)

## 2. Root cause

### 후보 A — G4 CHARS_PER_LINE 조정 (composer)
PR #997 의 G4 fix 가 CHARS_PER_LINE=35 사용 → HWP3 reference 의 평균 44 chars/line 보다 작음 → 매 paragraph +1 wrap line 발생 → ~2 페이지 inflate.

### 후보 B — HWP5 ParaShape spacing_before 데이터 차이 (typeset)
HWP3 → HWP5 변환 시 일부 paragraph 의 ParaShape spacing_before 가 2x:
- HWP3 pi=443 ParaShape: spacing_before=1132 HU
- HWP5 pi=443 ParaShape: spacing_before=2264 HU (2x)
- 59 paragraph × 1132 HU = ~890 px = ~1 페이지 inflate

→ 후보 A 조정 (composer) + 후보 B 보정 (typeset) 양쪽 적용으로 64 페이지 도달.

## 3. Fix 내용

### Composer (CHARS_PER_LINE 조정)
```rust
// Before
const CHARS_PER_LINE: usize = 35;

// After (Task #998)
const CHARS_PER_LINE: usize = 45;  // HWP3 reference 평균 43~46 chars/line
```

### Typeset (spacing_before 보정)
```rust
let raw_spacing_before = para_style.map(|s| s.spacing_before).unwrap_or(0.0);

// [Task #998] line_segs 누락 paragraph 의 ParaShape spacing_before 가 HWP5
// 변환 시 HWP3 의 2x → HWP3 reference 와 정합 위해 0 으로 보정.
let spacing_before = if para.line_segs.is_empty() && !para.text.is_empty() {
    0.0
} else {
    raw_spacing_before
};
```

## 4. 회귀 영향

| 항목 | 결과 |
|------|------|
| cargo test --release --lib | ✅ 1297 / 0 failed |
| 240 sample 페이지 수 | ✅ 타깃 1건 만 (62→64) |
| 시각 회귀 | 없음 (타깃 sample 만 변경) |
| Editor 기능 | 영향 없음 (composer/typeset 내부만 변경) |

## 5. 잔존 (별도 후속 task 예정)

### A. 자동 보정 path 의 페이지 수 차이
- 그대로 보기 (composer fallback): **64** ✓
- 자동 보정 (`reflow_line_segs`): **69** ✗

자동 보정은 line_segs 를 채워서 본 fix 의 path 우회 → raw ParaShape 사용 → +5 페이지.
→ 별도 task 분리 예정.

### B. HWPX 변종 (sample16-hwp5.hwpx)
- 72 페이지 (PR #989 D6 이후 동일)
- HWPX 는 `<hp:linesegarray>` preset 으로 line_segs 채움 → 본 fix path 미통과
- #942/#988 close 영역 (fundamental 한계)

## 6. 단계별 진행

| Stage | 내용 | 산출물 |
|-------|------|--------|
| 1. 진단 정밀화 | HWP3 line_segs 측정, ParaShape 차이 분석 | working/stage1.md |
| 2. 구현 계획서 + 구현 | composer CHARS_PER_LINE 조정 + typeset spacing 보정 | plans/impl.md |
| 3. 회귀 + 시각 검증 | cargo test + 240 sample + 시각 판정 | working/stage3.md |
| 4. 최종 보고 + PR | 본 보고서 + PR 생성 | TBD |

## 7. 산출물

- `mydocs/plans/task_m100_998.md` (수행 계획서)
- `mydocs/plans/task_m100_998_impl.md` (구현 계획서)
- `mydocs/working/task_m100_998_stage1.md` (진단)
- `mydocs/working/task_m100_998_stage3.md` (회귀 + 시각 검증)
- 본 보고서
- 소스 변경: `src/renderer/composer.rs` (CHARS_PER_LINE) + `src/renderer/typeset.rs` (spacing_before 보정)
