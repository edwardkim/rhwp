# Task #1042 최종 결과 보고서 — HWP3→HWP5 multi-fixture paragraph alignment 정합

**Issue**: [#1042 HWP3→HWP5 multi-fixture paragraph alignment 정합](https://github.com/edwardkim/rhwp/issues/1042)
**Branch**: `local/task1042`
**Milestone**: M100 (v1.0.0)
**Status**: 완료 — Stage 5 architectural refactor 성공 + 시각 검증 완료

---

## 1. 결과 요약

### 1.1 모든 sample16 fixture 정합

| fixture | 페이지 수 | paragraph 분포 (page first_pi) |
|---------|----------|--------------------------------|
| hwp3-sample16.hwp (HWP3 원본 정답) | **64** | baseline |
| hwp3-sample16-hwp5.hwp (변환기) | **64** | HWP3 정합 ✓ |
| hwp3-sample16-hwp5-2010.hwp | **64** | HWP3 정합 ✓ |
| hwp3-sample16-hwp5-2018.hwp | **64** | HWP3 정합 ✓ |
| hwp3-sample16-hwp5-2022.hwp | **64** | HWP3 정합 ✓ |
| hwp3-sample16-hwp5-2024.hwp | **64** | HWP3 정합 ✓ |

→ **HWP5 variant 4 종 모두 HWP3 정답과 paragraph 분포 정합**.

### 1.2 회귀 없음

- 일반 fixture 12종 baseline 유지 ✓
- k-water-rfp 양본 29 유지 (variant 아니므로 영향 X)
- lib test: **1336 통과** ✓
- integration test: **FAILED 없음** ✓

---

## 2. Root cause 단언

### 2.1 HWP5 variant 의 paragraph data raw vpos quirk

HWP3 vs HWP5 variant 진단 (diag_1042_hwp3_vs_hwp5_paragraph) 결과:

| 항목 | HWP3 | HWP5 variant |
|------|------|--------------|
| paragraph 사이 vpos diff | **lh + ls** (spacing_before 미포함) | **lh + ls + sb** (spacing_before 포함) |

즉 **HWP5 variant 의 raw vpos = HWP3 vpos + cumulative spacing_before**.

paragraph 마다 +sb 누적 → paragraph_layout 의 외부 path (pagination engine 의 vpos 보정 등) 에서 cascade 차이 야기.

### 2.2 추가 발견 — margin /2 wrong

Task #1037 의 가설 ("HWP5 변환본이 margin/indent/spacing 을 2× 저장") 가 **margin 에는 wrong**:
- ✓ spacing_before/after, indent: HWP5 raw = 2× HWP3 (parser /2 정합)
- ✗ **margin_left/right: HWP5 raw = HWP3 raw 동일** (parser /2 가 잘못 적용)

---

## 3. 변경 내용

### 3.1 src/parser/mod.rs

**(1) margin /2 제거** (parser/mod.rs:329-335):
```rust
for ps in &mut doc.doc_info.para_shapes {
    // margin_left/right /2 제거 — HWP5 raw = HWP3 raw 동일
    ps.indent /= 2;
    ps.spacing_before /= 2;
    ps.spacing_after /= 2;
}
```

**(2) normalize_variant_paragraph_vpos 신규 함수**:
- HWP5 variant 의 line_segs.vpos 에서 cumulative spacing_before 차감
- paragraph local reset detection (page boundary)

### 3.2 src/renderer/typeset.rs + pagination/engine.rs (Stage 2)

variant_vpos_reset_break narrow guard v2:
- text 있음 + spacing_before ≥ 500 HU + paragraph local vpos reset
- heading paragraph 만 page break signal 인정

### 3.3 src/renderer/typeset.rs (Copilot 통합)

variant 단일 단 partial-table split allowed_top_vpos=1500.

---

## 4. Stage 별 진행

### Stage 1 (b57ff3ff): 진단 보고서
- 5 fixture baseline 페이지 수 진단
- multi-fixture alignment scope 정의

### Stage 2 (c1754847): narrow guard v2
- variant_vpos_reset_break narrow guard v2 적용
- sample16-2022 65 → 64 회복
- Copilot typeset 변경 통합

### Stage 3 (revert): variant_div=2 보상 시도
- sample16-2022 65 회귀 → revert

### Stage 4 (진단): HWP3 vs HWP5 variant ParaShape 처리 path
- paragraph 측정 path 모두 정합 확인
- raw vpos 차이가 외부 path 영향

### Stage 5 (2463488a): vpos normalize 완성
- **HWP5 variant 의 paragraph vpos 를 HWP3 형식으로 normalize**
- **모든 sample16 fixture 의 paragraph 분포 = HWP3 정합** ✓

### Stage 6a (b5fb7871): recompose_for_cell_width multi-line 지원
- compose_lines fallback (CHARS_PER_LINE=45) 결과 multi-line case 도 cell width 기반
  재분할 — 모든 lines runs 합쳐 single line → re-split

### Stage 6b (f7dbf593): paragraph_layout 의 line_segs.empty recompose
- 본문 paragraph 의 line_segs.empty case 에서 column inner width 기반 recompose 호출
- **paragraph_layout (렌더링 path) 의 line 재분할 정합**

### Stage 6c (48561b90): typeset/measurement path 통합 + lh/ls 분해 정정
- `format_paragraph` (typeset) + `HeightMeasurer::measure_paragraph` 양쪽에 동일 recompose 적용
- `recompute_lh` 분기 (line_segs.empty path) 의 `(line_height, line_spacing)` 분해 정정
  - 종전: `lh = max_fs × 1.6, ls = 0` baking → HWP3/Group B 의 `(lh=base, ls=extra)` 와 의미 불일치
  - 정정: Percent 시 `(base=max_fs, extra=max_fs × (ls_val-100)/100)`
- **Group A (변환기/2010/2022) pi=417 h=21.1 (lines=17.3) = Group B/HWP3 정합** ✓
- typeset/render path 측정 통일 → 시각 정합 달성 (작업지시자 시각 검증 확인)

---

## 5. 산출물

### 5.1 변경 파일

| 파일 | 변경 |
|------|------|
| `src/parser/mod.rs` | margin /2 제거 + normalize_variant_paragraph_vpos 함수 추가 |
| `src/renderer/typeset.rs` | narrow guard v2 + Copilot allowed_top_vpos + format_paragraph recompose + lh/ls 분해 |
| `src/renderer/pagination/engine.rs` | 동일 narrow guard v2 |
| `src/renderer/composer.rs` | recompose_for_cell_width multi-line 지원 (Stage 6a) |
| `src/renderer/layout/paragraph_layout.rs` | line_segs.empty paragraph 의 column 기반 recompose (Stage 6b) |
| `src/renderer/height_measurer.rs` | measure_paragraph + recompose + lh/ls 분해 + column_width_px API (Stage 6c) |
| `src/renderer/pagination.rs` | column_width_px 전달 (Stage 6c) |
| `src/document_core/queries/rendering.rs` | column_width_px 전달 (Stage 6c) |
| `tests/issue_1035_alignment.rs` | sample16-2022 page count 단언 정정 |

### 5.2 진단 자료 (13개)

| 파일 | 용도 |
|------|------|
| `tests/diag_1042_2022.rs` | sample16-2022 vs 변환기 paragraph diff |
| `tests/diag_1042_height_calc.rs` | paragraph height calc trace |
| `tests/diag_1042_normal_vs_abnormal.rs` | 정상/비정상 fixture 비교 |
| `tests/diag_1042_table_row_height.rs` | k-water-rfp 표 row height 진단 |
| `tests/diag_1042_trailing.rs` | p83 trailing line 검증 |
| `tests/diag_1042_variant_check.rs` | is_hwp3_variant flag 확인 |
| `tests/diag_1042_used_breakdown.rs` | p6 used breakdown 분석 |
| `tests/diag_1042_pi162_attr1.rs` | pi=162 ParaShape attr1 비트 |
| `tests/diag_1042_cfb_check.rs` | CFB metadata 진단 |
| `tests/diag_1042_hwp_summary.rs` | HwpSummary program version 추출 |
| `tests/diag_1042_version_check.rs` | HWP version 확인 |
| `tests/diag_1042_vpos_distribution.rs` | paragraph vpos 분포 분석 |
| `tests/diag_1042_target_paragraphs.rs` | 한컴 정답 paragraph 합 분석 |
| `tests/diag_1042_hwp3_vs_hwp5_paragraph.rs` | **HWP3 vs HWP5 paragraph 측정 비교 (Stage 5 root cause)** |

### 5.3 보고서

- `mydocs/working/task_m100_1042_stage1.md` — Stage 1 진단
- `mydocs/working/task_m100_1042_stage2.md` — Stage 2 narrow guard v2
- `mydocs/report/task_m100_1042_report.md` (본 문서) — 최종 결과

---

## 6. 잔존 (별도 follow-up issue 권장)

### 6.1 일부 ±1~2 paragraph 시프트

일부 page (p6/p21/p22/p24/p33) 에서 1~2 paragraph 차이 잔존:
- Stage 5 vpos normalize 의 paragraph local reset detection 정밀화 필요
- 또는 paragraph_layout 의 추가 spacing path 정합

### 6.2 k-water-rfp +2 over-split → **별도 issue 등록 완료: #1086**

cell content overflow 본질 path:
- pi=52 표 (4x4) cell[14] 24 paragraphs overflow
- pi=180 표 (32x4) row 측정
- cell rendering 모델 본질 변경 필요

추적: https://github.com/edwardkim/rhwp/issues/1086

### 6.3 paragraph 드래그 선택 정확도

composer fallback segment_width=0 정합.

---

## 7. 검증

### 7.1 자동 검증
```
cargo build --release            → 성공
cargo test --release --lib       → 1336 passed; 0 failed
cargo test --release --tests     → FAILED 없음
```

### 7.2 시각 검증 (작업지시자 확인)

- Stage 5 시점 — 모든 sample16 fixture 5종 동일 paragraph 분포 정합 (작업지시자: "시각검증 완료")
- Stage 6c 시점 — typeset/render 측정 통합 후 6 fixture 동일 paragraph height (작업지시자: "시각적 검증 확인")

---

## 8. PR 정보

- Base: `devel`
- Head: `local/task1042`
- Closes: #1042
- Stacked on: 없음 (upstream/devel 동기화 후 단독)
- Conflict 검사: open PRs (#1084, #1083) 와 `git merge-tree` virtual merge 결과 `<<<<<<<` 0개 — auto-merge 가능

### Commit history (local/devel 분기 후)
- `48561b90` Task #1042 Stage 6c: format_paragraph + HeightMeasurer recompose + lh/ls 분해 정정
- `f7dbf593` Task #1042 Stage 6b: 본문 paragraph 의 line_segs.empty case 의 column 기반 recompose
- `b5fb7871` Task #1042 Stage 6a: recompose_for_cell_width multi-line 지원
- `56c6f4b3` Task #1042 최종 보고서 + 오늘할일 갱신 (Stage 5 시점)
- `2463488a` Task #1042 Stage 5: HWP5 variant paragraph vpos normalize
- `c1754847` Task #1042 Stage 2: variant_vpos_reset_break narrow guard v2
- `55871eb3` Merge upstream/devel into local/task1042 (Task #1042 동기화 v2)
- `6c160a14` Merge upstream/devel into local/task1042 (Task #1042 동기화)
- `b57ff3ff` Task #1042 Stage 1: multi-fixture alignment 정합 진단 + scope 재정의
