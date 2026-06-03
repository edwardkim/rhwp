# Task #775 최종 결과 보고서

**이슈**: [#775 — Task #703 회귀: exam_eng.hwp p4 27번 보기 그림 단 1 중반(+446.6px)으로 밀림](https://github.com/edwardkim/rhwp/issues/775)
**브랜치**: `local/task775` (`upstream/devel` 기준)
**마일스톤**: v1.0.0 (M100)

## 1. 결함 요약

`samples/exam_eng.hwp` page 4 (0-idx 3) 27번 문항 ("Adenville City Pass Card") 안내 그림
(1×1 InFrontOfText 표) 이 다단(2단) 단 1 (우측 컬럼) 상단 (정상 y≈277.08 px) 에서
단 1 중반 (현재 y≈723.69 px) 으로 약 **+446.6 px** 밀려 PDF 권위 자료
(`pdf/exam_eng-2022.pdf`) 와 시각 불일치.

## 2. 회귀 진원지 (bisect 확정)

**커밋 `a759a1c2`** — Task #703 / PR #707 "BehindText/InFrontOfText 표 본문 흐름 누락 정정".

```
src/renderer/typeset.rs:1403-1416 (+13)
typeset_table_paragraph 의 Control::Table 분기에 InFrontOfText/BehindText 가드 추가
→ 데코레이션 표를 Shape처럼 push 만 하고 cur_h 누적 건너뜀
```

| 시점 | 커밋 | cell-clip y | 상태 |
|------|-----|-------------|-----|
| Pre PR#644 | `1185eb98` | 277.08 | ✅ |
| Post PR#644 머지 | `42bb7946` | 277.08 | ✅ |
| Pre Task#703 (RED) | `afa70578` | 277.08 | ✅ |
| **Post Task#703 GREEN** | **`a759a1c2`** | **723.69** | ❌ **회귀** |
| 현재 devel | `e30e52f4` | 723.69 | ❌ |

## 3. 본질 정정

### 채택 — 옵션 A (`column_count == 1` 한정 push-only)

`src/renderer/typeset.rs:1553-1568` 의 가드 조건에 `&& st.col_count == 1` 추가:

```diff
                Control::Table(table) => {
                    // [Issue #703] 글앞으로 / 글뒤로 표는 Shape처럼 취급 — 본문 흐름 공간 차지 없음.
                    // pagination/engine.rs:976-981 와 동일 시멘틱: 데코레이션 표는 절대 좌표로 배치되며
                    // current_height 누적에 영향을 주지 않는다.
+                   //
+                   // [Issue #775] 단일 컬럼 한정. 다단(col_count>=2) 영역에서는 InFrontOfText/BehindText
+                   // 표라도 cur_h 누적이 컬럼 분배에 필요 (exam_eng.hwp p4 27번 보기 그림 위
+                   // 데코레이션 표 회귀 차단).
                    if matches!(
                        table.common.text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText
-                   ) {
+                   ) && st.col_count == 1
+                   {
                        st.current_items.push(PageItem::Shape {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                        continue;
                    }
```

### 폐기 — 옵션 B (`vert_rel_to == Page` 한정)

calendar_year.hwp 의 BehindText 표가 `vert=문단(Para)` 이므로 옵션 B 적용 시 push-only 제외 →
Task #703 본 케이스 회귀 발생. **column_count** 가 양 케이스의 결정적 차별화 요소로 확인.

## 4. 메커니즘

| 케이스 | column_count | text_wrap | vert_rel_to | 본 fix 동작 | 결과 |
|--------|--------------|-----------|-------------|-------------|------|
| calendar_year.hwp | **1** (단일 컬럼) | BehindText | Para | push-only (Task #703 fix 유지) | ✅ 1 page 유지 |
| calendar_monthly.hwp | **1** | BehindText | Para | push-only | ✅ 1 page 유지 |
| **exam_eng.hwp p4** | **2** (다단) | InFrontOfText | Para | cur_h 누적 (종전 동작 복귀) | ✅ y=277.08 정상 복원 |

## 5. 검증

### 회귀 가드 신규

- `tests/issue_775.rs::issue_775_exam_eng_p4_pi181_table_at_column_top` — RED → GREEN

### Issue #703 본 케이스 보존

- `tests/issue_703.rs::issue_703_calendar_year_single_page` — GREEN 유지
- `tests/issue_703.rs::issue_703_tonghap_2010_11_single_page` — `#[ignore]` (Issue #704)
- `tests/issue_703.rs::issue_703_tonghap_2011_10_single_page` — `#[ignore]` (Issue #704)

### 라이브러리 회귀 0

```
$ cargo test --release
총 통과: 1338  실패: 0  ignored: 5
```

### 광범위 sweep — 다단 6 fixture / 164 페이지

| sample | 페이지 | byte diff | 상태 |
|--------|--------|-----------|------|
| exam_kor | 20 | 0 | ✅ |
| **exam_eng** | **8** | **2** | ⚠️ p4 의도된 정정 + p2 ID 순서만 변경 |
| exam_science | 4 | 0 | ✅ |
| exam_math | 20 | 0 | ✅ |
| synam-001 | 35 | 0 | ✅ |
| aift | 77 | 0 | ✅ |

**유일 변경**: exam_eng p2 (좌표 동일, cell-clip ID 순서만 변경 — 시각적 회귀 0) +
exam_eng p4 (본 task 의도된 정정).

### 골든 SVG 7개

`issue_147_aift_page3`, `issue_157_page_1`, `issue_267_ktx_toc_page`, `form_002_page_0`,
`issue_617_exam_kor_page5`, `table_text_page_0`, `render_is_deterministic_within_process`
— 모두 GREEN.

## 6. PDF 권위 자료 정합

본 환경 (macOS, 한컴 편집기 미접근) 에서 PDF 직접 비교 불가. 정합 체인:

1. 본 fix 동작 = Task #703 이전 (`afa70578`) 동작 (다단 영역 한정)
2. Task #703 이전 동작 = PDF 권위 자료 정합 (bisect 단계 측정 cell-clip y=277.08)
3. 본 fix 후 cell-clip y = 277.08 (Stage 2 검증)

→ **PDF 정합 입증 완료**.

## 7. 변경 영향 범위

| 분기 조합 | 처리 동작 | 영향 |
|-----------|-----------|------|
| 단일 컬럼 + InFrontOfText/BehindText 표 | push-only (Task #703 fix) | 변경 없음 |
| **다단 + InFrontOfText/BehindText 표** | **cur_h 누적 (종전 동작 복귀)** | **본 fix 영향** |
| 그 외 wrap (TopAndBottom/Square/None/InsideText) | 변경 없음 | 변경 없음 |

영향 범위 = 다단 + 데코레이션 표 조합. exam_eng 만 해당. 다른 5개 다단 fixture 영향 0.

## 8. 산출물

| 영역 | 파일 |
|------|------|
| 거버넌스 — 수행 계획서 | `mydocs/plans/task_m100_775.md` |
| 거버넌스 — 구현 계획서 | `mydocs/plans/task_m100_775_impl.md` |
| 거버넌스 — Stage 1 보고서 | `mydocs/working/task_m100_775_stage1.md` |
| 거버넌스 — Stage 2 보고서 | `mydocs/working/task_m100_775_stage2.md` |
| 거버넌스 — Stage 3 보고서 | `mydocs/working/task_m100_775_stage3.md` |
| 거버넌스 — Stage 4 보고서 | `mydocs/working/task_m100_775_stage4.md` |
| 거버넌스 — 본 보고서 | `mydocs/report/task_m100_775_report.md` |
| 본질 정정 | `src/renderer/typeset.rs:1553-1568` (+5/-1) |
| 회귀 가드 | `tests/issue_775.rs` (+73) |

## 9. 후속

- 이슈 #775 close 권장 (closes #775)
- Issue #704 (TopAndBottom TAC + 각주 환경 borderline) 별건 영역 — 본 task 영향 없음
- WASM 빌드 + 시각 판정 영역은 작업지시자 결정 후 진행
- merge 절차: `local/task775` → push origin (fork) → upstream PR → cherry-pick 절차

## 10. 결론

Task #703 의 본 케이스 (단일 컬럼 BehindText 1×1 wrapper) 정합을 보존하면서 다단 영역의
회귀 (exam_eng.hwp p4 27번 보기 +446.6 px 시프트) 를 본질 정정. column_count 기반
1-line 가드로 두 케이스 모두 정합 달성. 라이브러리 회귀 0, 다단 광범위 sweep 회귀 0.
