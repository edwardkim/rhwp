# Task #1042 Stage 1 진단 보고서 — multi-fixture alignment 정합 본질 분석

**Issue**: [#1042 HWP3→HWP5 multi-fixture paragraph alignment 정합 — sample16-hwp5/k-water-rfp 한컴 오피스 버전별 정확 정렬 (PR #1036 후속)](https://github.com/edwardkim/rhwp/issues/1042)
**Branch**: `local/task1042`
**Stage**: 1 (진단 + 자료 정리, fix code 없음)

---

## 1. 본 task scope 재정의 배경

이전 scope (p23 외곽선 overflow) 의 모든 fix 시도 회귀. 사용자 새 대조군 (k-water-rfp.hwp + k-water-rfp-2024.hwp) 등장으로 본 task 의 본질이 **multi-fixture paragraph alignment 정합** 임을 입증.

새 title: "HWP3→HWP5 multi-fixture paragraph alignment 정합 — sample16-hwp5/k-water-rfp 한컴 오피스 버전별 정확 정렬 (PR #1036 후속)"

---

## 2. 5 fixture baseline 페이지 수 진단 (rhwp fix 없음)

| 파일 | 페이지 수 | line_segs empty | 한컴 정답 | diff |
|------|---------|----------------|---------|------|
| hwp3-sample16-hwp5.hwp (변환기) | 64 | 59 누락 | 64 (PDF) | ✓ |
| hwp3-sample16-hwp5-2010.hwp | 64 | 59 누락 | 64 | ✓ |
| hwp3-sample16-hwp5-2018.hwp | 64 | 0 (완전) | 64 | ✓ |
| hwp3-sample16-hwp5-2022.hwp | **65** | 59 누락 | 64 | **+1 ✗** |
| hwp3-sample16-hwp5-2024.hwp | 64 | 0 (완전) | 64 | ✓ |
| **k-water-rfp.hwp** | **29** | 0 (완전) | **27** (PDF) | **+2 ✗** |
| **k-water-rfp-2024.hwp** | **29** | 0 (완전) | **27** | **+2 ✗** |

---

## 3. Raw binary 리버싱 (sample16-hwp5 5 fixture)

[tests/diag_1042_raw_binary.rs](tests/diag_1042_raw_binary.rs) 진단 결과:

| 파일 | BodyText size | PARA_HEADER | PARA_TEXT | PARA_CHAR_SHAPE | PARA_LINE_SEG |
|------|---------------|-------------|-----------|-----------------|---------------|
| 변환기 | 647,520 | 4282 | 3093 | 4282 | **4223 (59 누락)** |
| 2010 | 647,520 | 4282 | 3093 | 4282 | **4223 (59 누락)** |
| 2018 | **652,426 (+4906)** | 4282 | 3093 | 4282 | **4282 (완전)** |
| 2022 | 647,330 | 4282 | 3093 | 4282 | **4223 (59 누락)** |
| 2024 | **652,652 (+5132)** | 4282 | 3093 | 4282 | **4282 (완전)** |

→ **두 그룹**:
- Group A (변환기 + 2010 + 2022): 59 PARA_LINE_SEG record 누락 (변환기 quirk 보존)
- Group B (2018 + 2024): PARA_LINE_SEG 완전 보정 (한컴 자체 개선)

파일 크기 차이 +5KB = 누락된 59 PARA_LINE_SEG record 의 크기.

---

## 4. 한컴 viewer 의 layout 동작 추론

한컴 viewer 는 PARA_LINE_SEG 없어도 layout 가능:
- paragraph header (paraShapeId, charShapeId) + text + font metric → line break + layout
- PARA_LINE_SEG 는 layout cache (빠른 load 용)
- 2010/2022 는 cache 미저장 (옛 변환기 quirk 그대로)
- 2018/2024 는 cache 자동 저장 (한컴 개선)

**rhwp baseline (composer fallback path) 가 viewer 동작 모방** → sample16 64 정합 (대부분의 fixture).

---

## 5. 합성 공식 도출 (2018/2024 reverse engineering)

[tests/diag_1042_2024_reverse.rs](tests/diag_1042_2024_reverse.rs):

```
lh = th = max base_size in paragraph
ls = th × (line_spacing - 100) / 100   (Percent type)
bl = th × 0.85
cs = ParaShape.margin_left
sw = body_width - margin_left - margin_right
tag: 첫 line=0x00060000, 이어지는 lines=0x00160000
```

**검증**:
- variant 와 2018/2024 둘 다 채워진 paragraph 1040 개의 `lh+ls` 매칭: **999/999 (100%)**
- p460 예: base_size=1300, line_spacing=160 → lh=1300, ls=780 ✓ 2024 actual 정합

---

## 6. k-water-rfp +2 over-split 정량 진단

페이지 별 diff (rhwp - hwp_used) — paragraph 만 있는 페이지:

| 페이지 | items | diff |
|--------|-------|------|
| 2 | 25 | +11.2px |
| 3 | 32 | +4.0px |
| 7 | 18 | +4.1px |
| 8 | 17 | +19.6px |
| 11 | 12 | +6.0px |
| 12 | 15 | +9.4px |
| 13 | 17 | +9.6px |
| 20 | 10 | +3.8px |
| 21 | 20 | +4.0px |
| 23 | 17 | +10.4px |

**일관 +4~+19 px diff**. paragraph 별 약 +0.5~+1 px 누적 → 27 페이지 누적 ~80~200 px = **+2 페이지 over-split**.

원인 추정:
- corrected_line_height 미세 차이 (font metric)
- baseline_distance 측정 차이
- spacing_before/after 적용 미세 차이
- font 의 line gap 계산 차이

---

## 7. paragraph_layout 의 multi-path 차이 단언

본 task 의 모든 fix 시도 회귀 원인:

paragraph_layout 이 line_segs.empty vs filled 에서 paragraph height 측정 알고리즘 다름:
- **empty path** (composer fallback): comp_line.line_height=400 sentinel → corrected_line_height(5.33, 17.33, Percent, 160) → 27.7 px/line. + comp_line.line_spacing 0 = 27.7 ✓
- **filled path**: comp_line.line_height=ls.line_height (1300 → raw_lh=17.33) → corrected_line_height(17.33, 17.33, ...) → raw_lh < max_fs? NO → else → 17.33. + comp_line.line_spacing 10.4 = 27.7 ✓

수식적으로 같은 결과. 그러나 paragraph_layout.rs:407-428 의 `line_break_char_indices` 가 line_segs.text_start 기반 → 합성 line_segs.text_start 가 부정확하면 line 분할 부정확 → paragraph 부풀림.

paragraph_layout.rs:1166 의 double count 버그 발견 (line_spacing 합산 시 line 3633 과 중복) — 본 task 의 fix 시도 중 발견 + revert.

---

## 8. 본 task 의 모든 fix 시도 결과 (모두 회귀)

| 시도 | sample16-hwp5 | 비고 |
|------|---------------|------|
| baseline (fix 없음) | **64 ✓** | paragraph 선택/p23 잔존 |
| sentinel 1300 (composer fallback) | 64 ✓ | 글자 compact + 선택 ✗ |
| corrected_line_height 분기 보강 (raw_lh <= max_fs) | — | aift +1, lib 9 fail |
| line_segs 합성 (CHARS_PER_LINE 45) | 100~110 | ✗ |
| line_segs 합성 + CHARS_PER_LINE 50 | 100 | ✗ |
| font_family lookup + estimate_text_width | 105 | ✗ |
| paragraph_layout double count fix | 105 | double count 발견 |
| word wrap + estimate_text_width | 106 | ✗ |
| **composer fallback 알고리즘 모방 (CHARS_PER_LINE 45 + 공백 backtrack)** | **103** | 약간 개선, 본질적 한계 입증 |

모든 시도 회귀 → 본 task 의 본질이 **paragraph_layout 의 multi-path 통합 + paragraph height 정밀화** 영역 (epic-level scope).

---

## 9. Stage 2 ~ 5 의 sub-task 분리

본 task 의 본질이 multi-fixture alignment 정합. 4 sub-task 로 분리하여 stage 별 진행:

### Stage 2: sample16-hwp5 paragraph 드래그 선택 정합
- composer fallback 의 ComposedLine.segment_width=0 보정
- 우선순위: 높음 (minimal 변경)

### Stage 3: k-water-rfp +2 over-split 해소
- paragraph 별 +0.5~+1 px 누적 차이 정합 보정
- PR #1036 style narrow guard 적용 가능성

### Stage 4: sample16-hwp5-2022 의 +1 baseline 회귀 해소
- paragraph data 동일이지만 section_def/header/version flag 차이 추정

### Stage 5: p23 외곽선 overflow 해소
- paginator + paragraph_layout 의 PartialParagraph 처리 정합화

---

## 10. Stage 1 산출물 + 다음 단계

### 보존 자료
- `tests/diag_1042_raw_binary.rs` — 5 fixture PARA_LINE_SEG 분포 진단
- `tests/diag_1042_2024_reverse.rs` — 2018/2024 reverse engineering + 합성 공식 검증

### 본 보고서
- 본 task 의 누적 진단 결과 종합
- multi-fixture alignment 본질 단언
- Stage 2~5 의 sub-task 분리 권고

### Stage 1 commit
- 진단 자료 + 보고서 + 수행/구현 계획서 정정 + 이슈 #1042 정정

각 후속 stage 는 작업지시자 결정으로 진행.
