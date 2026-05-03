# Task #544 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증 + 박스 자동 검출 한계 분석
**브랜치**: `local/task544`
**이슈**: https://github.com/edwardkim/rhwp/issues/544

---

## 1. 단위 테스트 전체 통과

```
test result: ok. 1120 passed; 0 failed; 2 ignored
test_544_passage_box_coords_match_pdf_p4 ... ok
```

기존 1120 단위 테스트 모두 통과. Task #537/#539/#540 무회귀.

## 2. 21_언어_기출 9개 박스 PDF 정합 상세 분석

### 2.1 직접 일치 페이지 (자동 검출 신뢰)

| 페이지 | 박스 | top diff | x diff | width diff |
|--------|------|---------|--------|-----------|
| 2 | [4~6] | -1.84 | +0.79 | +0.04 |
| 4 | [7~9] | -1.84 | +0.34 | +0.04 |
| 10 | [16~18] | -1.84 | +0.34 | +0.04 |

핵심 케이스 모두 PDF 와 일치 (-1.84 px 오차). PDF 측정 오차 범위 ±2 px 안.

### 2.2 자동 검출 한계 페이지 (수동 분석 필요)

| 페이지 | 상황 | 결론 |
|--------|------|------|
| 6 | PDF y=603.58 vs SVG y=574.96 (col 1) | col 1 첫 paragraph = pi=135 (question 12), passage 본문은 페이지 5 col 1 ~ 페이지 6 col 0. PDF/SVG 가 다른 박스 비교 |
| 8 | col 1 [13~15] -1.84 ✓ / col 0 -25.50 (다른 박스) | col 1 박스는 일치, col 0 차이는 다른 박스 자동 검출 |
| 13 | PDF col 1 [22~24] vs SVG col 0 (다른 컬럼) | PDF/SVG 페이지 layout 차이 (paginator 분배 다름) |
| 14 | 동일 | 동일 |
| 15 | 동일 | 동일 |

페이지 13/14/15 의 layout 차이는 본 task (paragraph border 좌표) 와 무관한
**기존 paginator 차이**. 별도 분석 task 필요 시 등록.

### 2.3 결론

본 task 의 fix 가 검증된 케이스 (페이지 2 [4~6], 페이지 4 [7~9], 페이지 10
[16~18]) 모두 PDF 일치. 작업지시자 보고 받은 9개 passage 박스 중 검증 가능한
케이스 모두 정정 완료.

## 3. 광범위 회귀 검증

### 3.1 6 핵심 샘플 vs Stage 540 baseline

| 샘플 | 텍스트 +시프트 | 텍스트 -시프트 | line diff | 평가 |
|------|--------------|--------------|-----------|------|
| 21_언어_기출 | 3077 | **0** | 66 | 의도 (Task #540 + Task #544) |
| synam-001 (음수 ls 57건) | 0 | **0** | 0 | 무변경 ✓ |
| exam_math | 349 | **0** | 0 | 의도 (Task #540) |
| exam_eng | 0 | **0** | 0 | 무변경 ✓ |
| exam_kor | 0 | **0** | 58 | 의도 (paragraph border 정정) |
| exam_science | 0 | **0** | 0 | 무변경 ✓ |

**텍스트 음의 시프트 0건 — 회귀 없음**.

### 3.2 셀 내부 paragraph border / wrap=Square 호스트

- 셀 내부 (`cell_ctx.is_some()`) 케이스: paragraph_layout.rs 의 push 분기에서
  `cell_ctx.is_none()` 가드 그대로 보존. 영향 없음.
- wrap=Square 호스트 (`border_box_override`) 케이스: `paragraph_layout.rs:2698`
  분기에서 override 좌표 그대로 사용. 기존 로직 보존.

광범위 검증에서 회귀 없음 → 셀 내부 / wrap host 케이스도 영향 없음 확인.

## 4. 메모리 룰 적용

### 4.1 [feedback_pdf_not_authoritative]

본 task 정정의 정합 기준: 한컴 2010 PDF (보조 ref). 한컴 2020 / 한컴독스 환경
검증은 작업지시자 입력 시 가능. 현재 핵심 fix 가 PDF 와 일치하지만, 다른 환경
검증 권고.

### 4.2 [feedback_essential_fix_regression_risk]

paragraph border 좌표 본질 정정 (col_area 전체 width + IR vpos 기반 top). 광범위
샘플 검증 (synam-001 음수 ls 57건 무변경 + 6 샘플 텍스트 음의 시프트 0건) 으로
회귀 위험 완화.

### 4.3 [feedback_rule_not_heuristic]

paragraph margin 은 텍스트 inset 으로만 사용 (룰), 박스 outline 은 col_area 전체
(룰). 분기/허용오차/fallback 미도입. PDF 일치를 룰 검증 기준으로 채택.

## 5. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/working/task_m100_544_stage3.md` | 본 보고서 |
| `mydocs/report/task_m100_544_report.md` | 최종 결과 보고서 (별도) |

## 6. 다음 단계

1. 최종 결과 보고서 작성
2. orders 갱신
3. local/task544 → local/devel → devel merge + push
4. PR #538 업데이트 (Task #544 추가)

`closes #544` 는 최종 commit 메시지에 포함.
