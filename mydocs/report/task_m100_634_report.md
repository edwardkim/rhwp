# Task #634 최종 보고서

**제목**: 한컴 호환 — 쪽번호 표시 동작 검증 + 회귀 방지 테스트
**브랜치**: `local/task634`
**이슈**: https://github.com/edwardkim/rhwp/issues/634
**Milestone**: M100 (v1.0.0)
**상태**: **완료 (Stage 4/4, 가설 정정 후 close)**
**완료 시점**: 2026-05-06

---

## 1. 배경

작업지시자 보고: `samples/aift.pdf` (한컴 PDF) 페이지 2 와 rhwp SVG 페이지 2 비교 시
한컴은 쪽번호 미표시, rhwp 는 "- 2 -" 표시 차이 발견.

## 2. 가설 시행 착오

### 2.1 가설 H1'' (Stage 0~3) — **잘못됨**

> 한컴은 첫 NewNumber Page 발화 페이지부터 쪽번호 표시.

Stage 1~3 에서 이 가설로 5건 통합 테스트 + Stage 2 fix (PageNumberAssigner.numbering_started)
구현. 검증 시 두 샘플 (aift, 국립국어원) 일치하는 듯 보였으나 **landscape 2-up
한컴 PDF 의 페이지 매핑을 잘못 해석**한 결과.

### 2.2 가설 정정 (Stage 4)

작업지시자가 새 한컴 PDF (1-up portrait) 추가 후 정밀 측정:

| rhwp page | 한컴 표시 | 메커니즘 |
|-----------|----------|---------|
| 1 (cover disclaimer) | **표시** | PageNumberPos 등록 후 표시 |
| 2 (사업계획서 표지)   | 미표시 | 미해결 → #637 |
| 3 (요약문)           | 미표시 | 미해결 → #637 |
| 4 (목차)             | 미표시 | PageHide on para 2.34 ✓ |
| 5 (별첨 목차)         | 미표시 | PageHide on para 2.54 ✓ |
| 6 (본문 시작)         | **표시** | 정상 |
| 7+ (NewNumber 발화 후) | **표시** | 정상 |

**가설 H1'' (NewNumber 게이팅) 은 한컴 동작이 아님**. 페이지 1, 6 표시는 NewNumber
무관. Stage 2 fix revert.

## 3. 작업 결과

### 3.1 코드 변경

Stage 2 fix (8 src 파일 + 9개 LOC) 모두 **revert**. 최종 코드 변경: **0** (페이지 2, 3
미해결로 정정 보류, 별도 issue #637 분리).

### 3.2 회귀 방지 테스트 (8건, Stage 4)

`src/renderer/layout/integration_tests.rs` 에 신규 추가:

| 테스트 | 검증 내용 | 결과 |
|--------|----------|------|
| `test_634_aift_page1_shows_page_number` | aift p1 표시 | PASS |
| `test_634_aift_page4_pagehide_no_page_number` | aift p4 PageHide 미표시 | PASS |
| `test_634_aift_page5_pagehide_no_page_number` | aift p5 PageHide 미표시 | PASS |
| `test_634_aift_page6_shows_page_number` | aift p6 표시 | PASS |
| `test_634_aift_page7_shows_page_number` | aift p7 NewNumber 후 표시 | PASS |
| `test_634_gukrip_page1_pagehide_no_page_number` | 국립국어원 p1 PageHide 미표시 | PASS |
| `test_634_gukrip_page3_shows_page_number` | 국립국어원 p3 표시 | PASS |
| `test_634_no_newnumber_doc_shows_page_numbers_from_page1` | hwp3-sample p1 표시 | PASS |

페이지 2, 3 미표시 메커니즘은 별도 issue **#637** 로 분리.

### 3.3 검증

```
test result: ok. 1127 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

기존 1119 + Task #634 신규 8 = 1127. **회귀 0**.

## 4. 메모리 룰 준수

- **[feedback_pdf_not_authoritative]**: PDF 측정도 정확한 page 매핑 (1-up vs 2-up,
  XObject vs direct text) 정밀 분석 필요. 첫 가설은 잘못된 매핑으로 도출.
- **[feedback_rule_not_heuristic]**: 가설 H1'' 가 룰 같았으나 실제로는 한컴 동작 미일치.
  명시적 룰 (PageHide) 만 처리하고 미해결 동작은 별도 분석.
- **[feedback_essential_fix_regression_risk]**: Stage 2 fix 가 페이지 1, 6 까지 미표시
  회귀 발생. revert 로 회귀 0 복귀. **잘못된 가설로 정정 시도하면 더 큰 회귀 발생 가능**.

## 5. 학습한 교훈

1. **landscape 2-up PDF 의 페이지 매핑 신중히** — 한컴 PDF 가 1-up 인지 2-up 인지 먼저
   확인. 첫 PDF 측정 시 landscape 였고 페이지 1 = (rhwp p1+p2 합본) 으로 추정했으나
   실제 매핑 모호. 1-up 출력으로 확실히 매핑한 후 가설 도출.
2. **footer 텍스트 op 측정에 cm/Tm 분리 필수** — Hancom PDF 는 cm 변환 + 작은 Tm Y 좌표
   조합. 단순 Tm.y 검사로는 footer 검출 불가. q/Q 스택 + cm 누적 필요.
3. **별도 메커니즘 분리** — 검증된 케이스 (페이지 1, 4, 5, 6, 7) 만 정정. 메커니즘 미확인
   케이스 (페이지 2, 3) 는 별도 issue 로 분리하여 회귀 위험 회피.

## 6. 산출물

| 파일 | 설명 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | Task #634 회귀 방지 테스트 8건 |
| `mydocs/plans/task_m100_634.md` | 수행 계획서 |
| `mydocs/plans/task_m100_634_impl.md` | 구현 계획서 (Stage 2 fix 시도, revert 됨) |
| `mydocs/working/task_m100_634_stage{0,1,2,3,4}.md` | 단계별 보고서 (Stage 4 = 가설 정정) |
| `mydocs/report/task_m100_634_report.md` | 본 최종 보고서 |
| `mydocs/orders/20260506.md` | 오늘 할일 항목 |

## 7. 결론

**가설 H1'' 잘못됨 → Stage 2 fix revert. Task #634 의 효용**:
- 8건 회귀 방지 통합 테스트 (한컴 일치 케이스 검증)
- 페이지 2, 3 미표시 메커니즘 별도 issue (#637) 로 분리

**closes #634**.

---

후속 PR 시: `local/task634` → `local/devel` merge → `devel` push.
