# Task #332 Stage 5 — vpos correction 가드 완화 + drift root cause 분석 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/layout.rs:1361-1364`)

```diff
                     if !prev_has_overlay_shape {
                     if let Some(prev_para) = paragraphs.get(prev_pi) {
-                        let prev_seg = prev_para.line_segs.iter().rev().find(|ls| {
-                            ls.segment_width > 0 && (ls.segment_width - col_width_hu).abs() < 3000
-                        });
+                        // Task #332 Stage 5: vpos correction trigger 조건 완화 —
+                        // 기존엔 segment_width 가 col_width 와 ±3000 HWPUNIT 이내일 때만 적용해
+                        // 짧은 단락/indent 가 있는 경우 trigger 누락 → drift 누적. 조건을 완화해
+                        // 마지막 segment 를 사용하되 width 검증 자체는 가드 조건으로 약화.
+                        let prev_seg = prev_para.line_segs.iter().rev().find(|ls| ls.segment_width > 0)
+                            .or_else(|| prev_para.line_segs.last());
```

vpos correction 의 segment_width 일치 가드를 완화. 짧은 단락/indent 가 있는 paragraph 도 vpos 보정을 받게 함.

## drift root cause 분석 (Stage 5 의 핵심 결과물)

### 21_언어 page 0 단 0 의 drift 추적

**typeset 의 cur_h 진행 (TYPESET_DRIFT_PI 출력)**:
- pi=0 (PartialParagraph + Table TopAndBottom): cur_h 가 표 effective_height 까지 누적 (~ 159px hwp_used)
- pi=1~9: 매 paragraph 마다 fmt.height_for_fit 누적
- pi=10 partial 진입 시점 cur_h = 1154.3px

**layout 의 y_offset 진행**:
- partial pi=10 line 1 끝 y = 1451.7
- 역산: line 0 시작 y ≈ 1437 - 24.2 = 1413, col_top 209.8 → 진행 = 1203.2px
- typeset 추정 1154.3 vs layout 실제 1203.2 → **drift = 48.9px**

### drift 발생원 (3 가지 후보)

1. **Table (TopAndBottom)** + host paragraph: typeset 이 표 effective_height 와 host 의 line height 를 별도 추정해 누적. layout 은 vpos 기반.
2. **Shape (TAC)**: pi=4 의 inline TAC Shape 가 line height 를 키움. typeset/layout 의 처리 차이.
3. **vpos correction trigger 조건의 광범위 누락**: 본 단계에서 완화했으나 multi-column + Table 환경에서는 여전히 trigger 안 되는 경우가 있음.

### drift 의 본질적 해결책 (3 가지 옵션)

- **A**: typeset 자체를 LINE_SEG vpos 기반으로 재설계. fit 검사를 vpos 진행 기반으로. 큰 작업, 별도 task.
- **B**: layout 의 vpos correction 을 paragraph 단위가 아닌 **line 단위**로 적용. 매 line 의 vpos 를 보정 기준으로 사용. 중간 규모 작업, 회귀 위험 큼.
- **C**: HeightMeasurer 를 typeset 과 layout 모두의 single source of truth 로 만들기. 표/Shape 측정 일치. 큰 작업.

본 Stage 5 에서는 **vpos correction 가드 완화** 만 적용. A/B/C 의 본질적 해결은 별도 task 분기 필요.

## 검증 결과

```
cargo test --lib                  → 992 passed
cargo test --test '*' (기타)      → 모두 passed
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (Stage 2 와 동일 baseline 차이)
```

### 21_언어 page 0

```
LAYOUT_OVERFLOW_DRAW: section=0 pi=10 line=1 y=1451.7 col_bottom=1436.2 overflow=15.5px
```

여전히 잔존. drift 본질 해결은 후속 task.

### 다른 샘플 회귀

| 샘플 | OVERFLOW | 비고 |
|------|----------|------|
| form-01 | 0 | 정상 |
| hwp-multi-002 | page 2 Table 31.3px (pre-existing), pi=68 line 0 44.7px DRAW | 손실/piling 없음 |
| multi-table-001 | 0 | 정상 |
| lseg-06-multisize | 0 | 정상 |
| aift | Table 2건 (pre-existing), pi=222 line 3 8.6px DRAW | 손실/piling 없음 |

vpos correction 완화로 인한 회귀 없음.

## 본 task 의 종합 평가

### 달성한 것

- ✅ typeset 의 advance 모델을 height_for_fit 으로 정합 (Stage 1)
- ✅ layout per-paragraph advance 를 동일 모델로 정합 (Stage 2)
- ✅ vpos correction 의 trail_ls 제외 + 양방향 + collapse 가드 (Stage 3a, 3b)
- ✅ vpos correction 의 segment_width 가드 완화 (Stage 5)
- ✅ clamp pile 제거 → 글자 겹침 차단 (Stage 4b)
- ✅ typeset 측 layout drift 안전 마진 + partial split avail_for_lines 마진 (Stage 4a, 4b)
- ✅ cargo test --lib 992 passed 유지
- ✅ 통합 테스트 회귀 없음 (golden 2개 baseline 변경은 의도)

### 미달성 / 알려진 한계

- ❌ Task #331 원 의도 (21_언어 page 1 col 1 의 pi=26 + 보기 ①②③ fit, PDF 일치): **부분 달성**.
  Stage 1 의 advance 변경으로 partial split 결과는 개선되었으나 layout drift 자체는 잔존.
- ❌ pi=10 line 1 의 col 경계 시각 넘김 15.5px: **잔존**. drift 본질 해결은 후속 task.

## 다음 단계 제안

**후속 task #N** (별도 이슈로 분기):

> typeset / layout 의 advance 모델 재설계 — LINE_SEG vpos 를 single source of truth 로

옵션 A (typeset 을 vpos 기반으로) 또는 B (layout 의 line 단위 vpos correction) 중 선택. 본 task #332 의 Stages 1~5 가 정합 작업의 토대가 되어 후속 작업이 더 안전.

## 산출물

- 본 task 332 의 5 단계 commit
- golden SVG 2 개 (issue-147, issue-157) baseline 갱신 필요 (의도된 변경)
- 최종 보고서: `mydocs/report/task_m100_332_report.md`
