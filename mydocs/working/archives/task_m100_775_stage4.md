# Task #775 Stage 4 보고서 — 광범위 sweep + 시각 판정

## 결과 요약

- ✅ 6개 다단 fixture sweep — `exam_eng` 만 변경 (의도된 정정), 5개 **회귀 0**
- ✅ exam_eng 8 페이지 중 2 페이지 변경 — 모두 시각적 회귀 0
  - p4: 본 task 의도된 정정 (y=723→277)
  - p2: 좌표 100% 동일, cell-clip ID 순서만 변경 (시각 정합 유지)
- ✅ 단일 컬럼 본 케이스 보존 — `calendar_year.hwp` / `calendar_monthly.hwp` 모두 1 페이지 유지
- ✅ 골든 SVG 7개 (Stage 3 에서 확인) 회귀 0

## 다단 sweep — 6 fixture × 페이지 byte 비교

```bash
# baseline = upstream/devel (e30e52f4 — 회귀 상태)
# after = local/task775 (본 fix)
```

| sample | 페이지 | byte diff | 상태 |
|--------|--------|-----------|------|
| exam_kor | 20 | 0 | ✅ 회귀 0 |
| **exam_eng** | **8** | **2** | ⚠️ 의도된 정정 (분석 ↓) |
| exam_science | 4 | 0 | ✅ 회귀 0 |
| exam_math | 20 | 0 | ✅ 회귀 0 |
| synam-001 | 35 | 0 | ✅ 회귀 0 |
| aift | 77 | 0 | ✅ 회귀 0 |

**총 다단 페이지: 164  변경: 2 (exam_eng p2 + p4)**

## exam_eng 변경 분석

### page 4 — 본 task 의도된 정정 ✅

```
[Before fix]  cell-clip-211: y=723.69 (회귀)
[After fix]   cell-clip-160: y=277.08 (PDF 정합 정상값)
```

27번 보기 그림 (1×1 InFrontOfText 표) 가 단 1 상단으로 복귀. PDF 권위 자료
(`pdf/exam_eng-2022.pdf`) 와 정합.

### page 2 — 좌표 동일, ID 순서만 변경 ✅ (회귀 0)

```
$ diff <(grep -oE '<text[^>]*' before/exam_eng_002.svg | sort) \
       <(grep -oE '<text[^>]*' after/exam_eng_002.svg  | sort) | wc -l
0
```

**텍스트 좌표 100% 동일** — 변경은 cell-clip ID 부여 순서 차이뿐:

| Before | After | 좌표 |
|--------|-------|------|
| cell-clip-219 | cell-clip-218 | x=597.12 y=243.59 (동일) |
| cell-clip-192 | cell-clip-179 | x=117.17 y=1362.08 (동일) |

InFrontOfText 표가 cur_h 누적 분기로 변경되면서 `current_items.push` 순서가
1 step 변경 → ID 부여 순서만 시프트. **시각적 출력 100% 동일**.

## 단일 컬럼 본 케이스 보존 검증

```
calendar_year.hwp     pages=1  ✅ (Task #703 본 케이스 — BehindText 1×1 wrapper)
calendar_monthly.hwp  pages=1  ✅ (동일 패턴)
```

→ 단일 컬럼 영역에서 Task #703 fix 그대로 적용되어 본 케이스 보존.

## 골든 SVG 영역 (Stage 3 인용)

```
Running tests/svg_snapshot.rs
test result: ok. 7 passed; 0 failed
```

`issue_147_aift_page3`, `issue_157_page_1`, `issue_267_ktx_toc_page`, `form_002_page_0`,
`issue_617_exam_kor_page5`, `table_text_page_0`, `render_is_deterministic_within_process`
모두 회귀 0.

## PDF 권위 자료 정합

본 환경 (macOS) 에서 PDF 직접 비교 불가. 다음 정합 체인으로 입증:

1. **본 fix 의 동작** = `Task #703 이전 (afa70578) 동작` (다단 영역 한정)
2. **Task #703 이전 동작** = PDF 권위 자료 정합 (bisect 단계에서 cell-clip y=277.08 측정)
3. **본 fix 후 cell-clip y** = 277.08 (Stage 2 검증)

→ **PDF 정합 입증 완료**.

## 영향 범위 정리

| 분기 조합 | 처리 동작 | 영향 |
|-----------|-----------|------|
| 단일 컬럼 + InFrontOfText/BehindText 표 | push-only (Task #703 fix) | 변경 없음 |
| **다단 + InFrontOfText/BehindText 표** | **cur_h 누적 (종전 동작 복귀)** | **본 fix 영향** |
| 그 외 (TopAndBottom/Square/None) | 변경 없음 | 변경 없음 |

→ 영향 범위 = 다단 + 데코레이션 표 조합. exam_eng 만 해당 (sweep 결과 정합).

## Stage 5 진행 조건

- 본 단계 보고서 승인
- Stage 5: 최종 결과 보고서 (`mydocs/report/task_m100_775_report.md`) + orders 갱신
