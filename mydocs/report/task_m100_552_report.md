# Task #552 최종 결과 보고서

**제목**: Task #479 회귀 정정 — paragraph border 시작 직전 trailing line spacing 보존
**브랜치**: `local/task552`
**이슈**: https://github.com/edwardkim/rhwp/issues/552
**관련 회귀 이슈 (선행)**: Task #479 (페이지 12 200 px drift fix)

---

## 1. 요약

Task #479 가 본문 paragraph (cell_ctx.is_none()) 마지막 줄에서 trailing line
spacing 을 제외하도록 변경한 결과, **다음 paragraph 가 visible border 시작인
경우** 박스 top y 가 `line_spacing` 만큼 위로 이동하여 PDF (한컴 2010) 와 정합이
어긋나는 회귀 발생. (예: 21_언어_기출 페이지 2 [4~6] 박스 → header 와 gap=0
밀착)

본 task 는 "transition 시점" 에서만 trailing ls 를 복원하여 PDF 정합을 회복
하면서, Task #479 의 본 효과 (다중 paragraph 누적 결과인 페이지 12 drift fix)
는 보존한다.

## 2. 변경 본질

`is_full_paragraph_end && cell_ctx.is_none()` 분기를 다음 paragraph 의 visible
border 여부로 두 갈래로 분리:

| 다음 paragraph | trailing ls 처리 | 의미 |
|----------------|------------------|------|
| no visible border (1652 cases) | 제외 (Task #479 그대로) | 본문 sequential, drift 누적 방지 |
| visible border 시작 (48 cases) | **보존** (Task #552 신규) | 박스 top y PDF 정합 |
| in_border (290 cases) | 변경 없음 | border 그룹 내부 |
| border→no (48 cases) | 변경 없음 | border 끝 |

## 3. 핵심 측정값

페이지 2 우측 단 [4~6] 박스 (test_552_passage_box_top_gap_p2_4_6):

| 항목 | pre-#479 | post-#479 (회귀) | post-#552 (fix) | PDF 한컴 2010 |
|------|----------|------------------|-----------------|----------------|
| 박스 top y | 233.97 | 224.43 | **233.97** | 175.36 pt |
| gap (header→box) | 9.54 px | 0.00 px | **9.54 px** | 8.73 px |

→ **pre-#479 baseline 정확 회복**. PDF ±2 px tolerance 통과.

## 4. 광범위 회귀 검증 (Stage 3)

### 4.1 단위 테스트

```
test result: ok. 1119 passed; 0 failed; 2 ignored
test test_552_passage_box_top_gap_p2_4_6 ... ok    (RED → GREEN)
```

### 4.2 페이지 카운트 무회귀 (6 샘플)

| 샘플 | pages |
|------|-------|
| 21_언어_기출_편집가능본 | 15 |
| exam_kor | 20 |
| exam_math | 20 |
| exam_eng | 8 |
| exam_science | 6 |
| synam-001 | 35 |

→ Stage 1 baseline 동일. 다중 paragraph 누적 본질 보존.

### 4.3 paragraph border 전환 분포 (Stage 1 도구 재실행)

| 샘플 | total | no→border | in_border | border→no | no→no |
|------|-------|-----------|-----------|-----------|-------|
| 21_언어_기출 | 325 | 10 | 59 | 10 | 245 |
| exam_kor | 749 | 14 | 225 | 14 | 493 |
| exam_math | 275 | 8 | 4 | 8 | 253 |
| exam_eng | 318 | 16 | 2 | 16 | 283 |
| exam_science | 130 | 0 | 0 | 0 | 129 |
| synam-001 | 250 | 0 | 0 | 0 | 249 |
| **합계** | **2047** | **48** | 290 | 48 | 1652 |

영향 케이스 = no→border 48 cases. 그 외 1999 cases 변경 없음.

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | Cell + helper + 3 caller 보강 (+44 LOC) |
| `src/renderer/layout/paragraph_layout.rs` | `is_full_paragraph_end` 분기 보강 (+5 / -1 LOC) |
| `src/renderer/layout/integration_tests.rs` | RED→GREEN 테스트 1건 (+97 LOC) |
| `examples/scan_border_starts.rs` | 광범위 평가 도구 (신규) |
| `mydocs/plans/task_m100_552.md` | 수행계획서 |
| `mydocs/working/task_m100_552_stage1.md` | Stage 1 보고서 |
| `mydocs/working/task_m100_552_stage2.md` | Stage 2 보고서 |
| `mydocs/report/task_m100_552_report.md` | 최종 보고서 (본 문서) |

## 6. 커밋 이력

```
e2b9a711 Task #552 Stage 1: TDD RED 테스트 + 광범위 사전 평가
1934161f Task #552 Stage 2: paragraph border 시작 직전 trailing ls 보존
[Stage 3 commit] Task #552 Stage 3: 광범위 회귀 검증 + 최종 보고서
```

## 7. Scope 외 사항 (별도 이슈 후보)

본 task 진행 중 발견되었으나 scope 외인 사항:

### 7.1 paragraph border 좌측 850 HU 시프트 회귀

- 현상: [1~3], [4~6] 등 박스가 우측으로 약 11.34 px (850 HU) 시프트
- 원인: Task #544 (commit `7ba2ecbe`) 에서 정정한 `box_x` 산식이 merge `a7e43f99`
  (Task #517~#528 통합) 에서 **revert** 됨
- 동시에 `test_544_passage_box_coords_match_pdf_p4`, `test_547_passage_text_inset_match_pdf_p4`,
  `test_548_cell_inline_shape_first_line_indent_p8` 통합 테스트도 누락
- 본 task 와 코드 경로·테스트가 다르므로 **별도 신규 이슈** 로 분리 처리

## 8. 승인 요청

Task #552 모든 단계 완료. RED→GREEN, 1119 baseline 무회귀, 페이지 카운트 보존,
영향 분포 baseline 동일. local/devel merge 진행 OK?
