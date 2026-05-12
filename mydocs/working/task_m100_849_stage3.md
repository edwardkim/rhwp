# Stage 3 보고 — Task #849 (M100) — 광역 회귀 검증

상태: **회귀 0 확인**.

## 1. 페이지 수 비교 (baseline `upstream/devel` ↔ 현재 `#846+#849`)

| 샘플 | baseline | 현재 | 비고 |
|------|---------:|-----:|------|
| `basic/shortcut.hwp` | 8 | **7** | 의도 변경 (pi=94/95 페이지 3 이동, 한컴 PDF 7 정합) |
| `exam_math.hwp` | 20 | 20 | 무변화 (#846 단독 시 11 → #849 게이트로 baseline 복원) |
| `exam_math_8.hwp` | 1 | 1 | 무변화 |
| `exam_eng.hwp` | 8 | 8 | 무변화 |
| `exam_kor.hwp` | 20 | 20 | 무변화 |
| `exam_science.hwp` | 4 | 4 | 무변화 |
| `exam_social.hwp` | 4 | 4 | 무변화 |
| `k-water-rfp.hwp` | 27 | 27 | 무변화 |
| `biz_plan.hwp` | 6 | 6 | 무변화 |
| `hwpspec.hwp` | 175 | 175 | 무변화 |
| `issue-505-equations.hwp` | 4 | 4 | 무변화 |
| `21_언어_기출_편집가능본.hwp` | 15 | 15 | 무변화 |
| `basic/interview.hwp` | 2 | 2 | 무변화 |
| `basic/treatise sample.hwp` | 7 | 7 | 무변화 |
| `hwp-3.0-HWPML.hwp` | 122 | 122 | 무변화 |
| `aift.hwp` | 77 | 77 | 무변화 |

## 2. SVG 바이트 비교 (`export-svg`, baseline ↔ 현재)

대상: exam_eng, exam_kor, biz_plan, treatise sample, k-water-rfp, shortcut, 21_언어, exam_math.

```
shortcut_003.svg ~ shortcut_007.svg  → 차이 (pi=94/95 reflow + 후속 cascade)
shortcut_008.svg                     → baseline 에만 존재 (8→7페이지)
그 외 전 파일 (exam_eng / exam_kor / biz_plan / treatise / k-water-rfp / 21_언어 / exam_math 전 페이지) → 바이트 동일
```

즉 본 변경의 SVG 영향은 **`shortcut.hwp` 한 문서에 한정**, 나머지 다단/표분할/목차 샘플은 baseline 과 픽셀 단위 동일.

## 3. `cargo test` / clippy

- `cargo test --lib`: **1232 passed; 0 failed**. (`test_exam_math_page_count` ≥18, `test_539_partial_paragraph_after_overlay_shape`, `test_548_cell_inline_shape_first_line_indent_p8` 포함 — #846 단독 시 실패하던 3건 모두 통과.)
- `cargo clippy --lib`: `src/renderer/typeset.rs` 신규 경고 0 (#846 의 `find_map(all-Some)` 경고는 본 단계에서 `last().map(..)` 으로 정리). 기존 `table_ops.rs`/`object_ops.rs` unwrap 패닉 경고는 본 타스크 무관(CLAUDE.md 기재).

## 4. 분석

본 변경(`start_new_column_band` 호출을 `ColumnType::Distribute` zone 으로 한정)은 **"배분(Distribute) 단 zone 에서 마지막 단에 `[단나누기]` 가 오는 경우"** 에만 동작한다. 이 조건을 만족하는 샘플은 `shortcut.hwp` 의 "보기"/"입력" 등 zone 뿐 — 그래서 다른 모든 샘플이 무변화. `exam_math.hwp` 등 신문형(`Normal`) 다단 문서는 #846 이전과 정확히 동일하게 동작.

다음: Stage 4 종합 검증 및 최종 보고서.
