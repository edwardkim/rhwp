# Task #1042 Stage 2 완료 보고서 — variant_vpos_reset_break narrow guard v2 (heading 정합)

**Issue**: [#1042 HWP3→HWP5 multi-fixture paragraph alignment 정합](https://github.com/edwardkim/rhwp/issues/1042)
**Branch**: `local/task1042`
**Status**: Stage 2 완료 (sample16-hwp5-2022 65→64 정답 회복 + heading paragraph cascading 해소)

---

## 1. Stage 2 목표

Stage 1 의 진단 결과를 기반으로 본 task 의 **핵심 architectural fix** 적용:

1. **sample16-hwp5-2022 의 +1 over-split 해소** (65 → 64 정답)
2. **heading paragraph cascading 해소** — 한컴 정답지의 page 4 ("2. 제안참가 안내" 시작) 와 rhwp page 7 정합
3. **upstream/devel 동기화** + Copilot typeset 변경 통합
4. **회귀 없음** — 일반 fixture + lib test + integration test 모두 baseline 유지

---

## 2. Root cause 단언

### 2.1 sample16-hwp5-2022 의 +1 over-split

이전 진단 (Stage 1) 에서 fixture 의 paragraph vpos +1952 HU 누적 시프트가 root cause 라 단언했으나, Stage 2 진단에서 **variant_vpos_reset_break path 의 false positive** 가 진정한 root cause 임을 확인:

- Task #1007/#1035 의 `variant_vpos_reset_break` path 는 paragraph 의 `vpos < 1500 HU` + 이전 paragraph 의 `vpos_end > body_height × 95%` 인 경우 page break trigger
- sample16-hwp5-2022 의 일부 paragraph (예: pi=87 빈 문단, sb=284 HU) 가 false positive trigger → +1 over-split

### 2.2 Heading paragraph cascading

사용자 시각 검증 (한컴 오피스 2024 직접 출력 vs rhwp p7) 에서 발견된 cascading:
- 한컴 정답지 page_num=4 (실제 page 7) 시작 = "2. 제안참가 안내" (pi=162)
- rhwp page 7 시작 = "(1) 제안참가신청" (pi=164) — pi=162 가 page 6 끝에 fit

진단 결과:
- pi=162 의 vpos=852 (paragraph local, **vpos reset signal**), spacing_before=852 HU (heading)
- 한컴 encoder 의 page break signal — heading paragraph 의 paragraph local vpos
- rhwp 가 이 signal 처리 안 함 → heading orphan

### 2.3 양 root cause 의 충돌 해소

- **path 완전 제거** → sample16-2022 정합 ✓, cascading 잔존 ✗
- **path narrow guard (text_len > 0 만)** → sample16-2022 회귀 ✗
- **path narrow guard v2 (text + spacing_before ≥ 500 HU)** → **양쪽 모두 정합 ✓✓**

---

## 3. 변경 내용

### 3.1 src/renderer/typeset.rs + src/renderer/pagination/engine.rs

variant_vpos_reset_break narrow guard v2 적용 — heading paragraph 만 page break signal 인정:

```rust
let mut variant_vpos_reset_break = false;
if is_hwp3_variant && body_height_hu_for_variant > 0 && !para.text.is_empty() {
    let para_sb_hu = para_styles
        .get(para.para_shape_id as usize)
        .map(|ps| (ps.spacing_before * 7200.0 / 96.0) as i32)
        .unwrap_or(0);
    if para_sb_hu >= 500 {
        // ... vpos reset detection (high_threshold 95%, low_threshold 1500)
    }
}
```

조건:
- `is_hwp3_variant`: 변환본 한정
- `para.text.is_empty() == false`: 빈 문단 skip (sample16-2022 pi=87 false positive 차단)
- `spacing_before ≥ 500 HU`: heading paragraph 만 (content paragraph 의 false positive 차단)
- `prev_end_vpos > body_height × 95%`: 페이지 끝 부근만
- `curr_first_vpos < 1500`: paragraph local vpos reset

### 3.2 src/renderer/typeset.rs (Copilot 변경 통합)

```rust
let allowed_top_vpos = if st.is_hwp3_variant { 1500 } else { 0 };
matches!((next_first_vpos, curr_last_vpos), (Some(nv), Some(cl))
if (if multi_col { nv < cl } else { nv <= allowed_top_vpos })
    && cl > 5000)
```

variant 의 단일 단 partial-table split 결정 시 next paragraph vpos 허용 상한 0 → 1500 (variant 한정).

### 3.3 upstream/devel 동기화

Merge commit `6c160a14` — upstream/devel 의 새 변경 (Task #1062 endnote, #1067 shape, #1053 HWPML, #1066 PR 등) 통합. stash pop 자동 merge 성공 (conflict 없음).

---

## 4. 검증 결과

### 4.1 페이지 수 sweep

| fixture | 정답 | rhwp 결과 | 상태 |
|---------|------|-----------|------|
| sample16-hwp5 (변환기) | 64 | 64 | ✓ |
| sample16-hwp5-2010 | 64 | 64 | ✓ |
| sample16-hwp5-2018 | 64 | 64 | ✓ |
| **sample16-hwp5-2022** | **64** | **64** | **✓ (Stage 1 baseline 65 → 64 회복)** |
| sample16-hwp5-2024 | 64 | 64 | ✓ |
| k-water-rfp | 27 | 29 | 잔존 (+2, 별도 follow-up) |
| k-water-rfp-2024 | 27 | 29 | 잔존 (+2, 별도 follow-up) |

### 4.2 일반 fixture 회귀 sweep

| fixture | baseline | 현재 | 상태 |
|---------|---------|------|------|
| hwp3-sample-hwp5 | 16 | 16 | ✓ |
| hwp3-sample4-hwp5 | 36 | 36 | ✓ |
| hwp3-sample5-hwp5 | 64 | 64 | ✓ |
| hwp3-sample10-hwp5 | 763 | 763 | ✓ |
| hwp3-sample11-hwp5 | 151 | 151 | ✓ |
| hwp3-sample13-hwp5 | 3 | 3 | ✓ |
| hwp3-sample14-hwp5 | 11 | 11 | ✓ |
| hwp3-sample19-hwp5 | 2 | 2 | ✓ |
| exam_kor | 20 | 20 | ✓ |
| exam_math | 20 | 20 | ✓ |
| aift | 74 | 74 | ✓ |
| biz_plan | 6 | 6 | ✓ |

### 4.3 자동 검증

```
cargo test --release --lib       → test result: ok. 1335 passed; 0 failed
cargo test --release --tests     → FAILED 없음
cargo build --release            → warning 없음
```

### 4.4 cascading 해소 검증 (rhwp p6/p7 boundary)

이전 (Stage 1, path 제거 후):
- rhwp p6 (page_num=4) 마지막 paragraph = pi=162 "2. 제안참가 안내"
- rhwp p7 (page_num=5) 시작 paragraph = pi=163 (빈) + pi=164 "(1) 제안참가신청"

현재 (Stage 2, narrow guard v2):
- rhwp p6 (page_num=4) 마지막 paragraph = pi=160 "* 입찰가격 평점산식의 ..."
- **rhwp p7 (page_num=5) 시작 paragraph = pi=162 "2. 제안참가 안내"** ← 한컴 정답 정합 ✓

---

## 5. Stage 1 의 가설 정정

Stage 1 의 진단:
- **잘못된 가설**: fixture 의 vpos 누적 +1952 HU 시프트가 root cause
- **정확한 단언**: variant_vpos_reset_break path 의 false positive trigger 가 root cause

진단 path (Stage 1 → Stage 2):
1. paragraph_layout 의 last ls 제외 시도 → test_544 PDF y 기대값과 -9.4 px diff → **wrong direction** revert
2. variant_vpos_reset_break path 완전 제거 → sample16-2022 정합 ✓ but cascading 잔존
3. narrow guard text_len > 0 → sample16-2022 회귀
4. **narrow guard text + spacing_before ≥ 500 HU → 양쪽 정합 ✓✓**

---

## 6. 잔존 결함 (별도 follow-up)

### 6.1 k-water-rfp 양본 +2 over-split

- root cause: 표 row 측정 over-estimate (pi=180 표 32x4) + 표 page break 결정 (pi=52 표 4x4, cell content > cell.h)
- 시도: cell_units only / 3 path 동시 / paras>10 narrow guard / 표 fit 가능시 통째 이동 — 모두 회귀
- 결론: cell rendering 모델의 본질적 변경 필요 (별도 task)

### 6.2 sample16-hwp5 의 paragraph 드래그 선택 정확도

- root cause: composer fallback 의 ComposedLine.segment_width=0 → paragraph_layout hit-test 부정확
- Stage 2 scope 외 — 별도 task

---

## 7. 산출물

### 7.1 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/typeset.rs` | variant_vpos_reset_break narrow guard v2 + Copilot allowed_top_vpos |
| `src/renderer/pagination/engine.rs` | 동일 narrow guard v2 |

### 7.2 진단 자료

| 파일 | 용도 |
|------|------|
| `tests/diag_1042_2022.rs` | sample16-2022 vs 변환기 paragraph diff |
| `tests/diag_1042_height_calc.rs` | paragraph height calc trace |
| `tests/diag_1042_normal_vs_abnormal.rs` | 정상 (2010/2024) vs 비정상 (2018/2022) 비교 |
| `tests/diag_1042_table_row_height.rs` | k-water-rfp 표 row height over-estimate 정량 |
| `tests/diag_1042_trailing.rs` | p83 trailing line 검증 |
| `tests/diag_1042_variant_check.rs` | is_hwp3_variant flag 확인 |
| `tests/diag_1042_used_breakdown.rs` | p6 used breakdown — paragraph_layout y 누적 분석 |
| `tests/diag_1042_pi162_attr1.rs` | pi=162 ParaShape attr1 비트 (keep_with_next 등) |

### 7.3 본 보고서
- `mydocs/working/task_m100_1042_stage2.md` (본 문서)

---

## 8. 다음 단계

### Stage 3 (최종)
- 최종 결과보고서 (`mydocs/report/task_m100_1042_report.md`)
- 오늘 할일 갱신 (`mydocs/orders/`)
- k-water-rfp 의 +2 over-split follow-up issue 등록
- 시각 검증 (작업지시자 PC 의 rhwp-studio 또는 SVG export-svg + PDF 대조)
- Stage 2 + Stage 3 통합 commit
- PR 생성 (base devel, closes #1042)
