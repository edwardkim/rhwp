# Task #1037 Stage 4 진단 보고서 — D 시도 (page break + line_seg 합성) negative result

**Issue**: [#1037 HWP5 변환본 paragraph height 과대 측정](https://github.com/edwardkim/rhwp/issues/1037)
**Branch**: `local/task1037`
**작업 내용**: 작업지시자 옵션 D 선택 — page break vpos==0 휴리스틱 + p23 overflow line_seg 합성 옵션 B' 정량 평가

---

## 1. Stage 4-A — page break vpos==0 휴리스틱 (HWP5 변환본)

### 1.1 가설

한컴 변환기가 HWP3 → HWP5 변환 시 page_break_before 정보를 100% 손실 (Stage 3 fix 보고에서 진단). vpos==0 paragraph 가 페이지 시작 paragraph 신호인지 검증.

### 1.2 진단 (`tests/diag_1037_stage4.rs`)

HWP3 native [쪽나누기] paragraph ↔ HWP5 변환본 vpos==0 paragraph 교차 검증:

| 항목 | 결과 |
|------|------|
| HWP3 native [쪽나누기] paragraph | **57 개** |
| HWP5 변환본 vpos==0 paragraph | 22 개 |
| 공통 (true positive) | 18 개 |
| HWP3 만 (false negative — 휴리스틱 누락) | **39 개** |
| HWP5 만 (false positive — 휴리스틱 오감지) | 4 개 |
| Precision (TP / TP+FP) | 81.8% |
| **Recall (TP / TP+FN)** | **31.6%** (매우 낮음) |

### 1.3 결론 — 폐기

휴리스틱 적용 시 예상 효과:
- 18 paragraph: 정합 회복 ✓
- 39 paragraph: 여전히 미정합 (휴리스틱 누락)
- 4 paragraph: 잘못된 page break → over-split 회귀 위험

→ 한컴 변환기가 vpos 정보도 page_break_before 와 무관하게 인코딩. **vpos==0 휴리스틱 부적합**.

---

## 2. Stage 4-B — p23 외곽선 overflow line_seg 합성 옵션 B' (사전 평가)

### 2.1 Task #1010 Stage 2 회귀 데이터 재검토 (commit `f4710615`)

옵션 A line_seg 합성 시도 결과:
- 페이지 수: **64 → 88 (+24 over-split, 회귀)**
- max overflow: 158 → 669 px (악화)
- **결정적**: cross-correlation 으로 **overflow 80% 가 missing paragraph 와 무관**

### 2.2 옵션 B' (더 보수적 합성) 위험 분석

Task #1010 Stage 2 의 root cause:
- composer fallback hardcoded line_height=400 은 measure/render 정합 유지 위한 sentinel
- 합성으로 donor line_height (~1000) 주입 시 paragraph 키 2.5× → 페이지 split 회귀

옵션 B' (FullParagraph 만 합성, PartialParagraph 제외 등) 도 동일 양상 재현 위험:
- 합성된 paragraph 가 measure 와 render 양쪽 정합성 유지 어려움
- root cause 무관 단언 (cross-correlation) 으로 시도 가치 낮음

### 2.3 결론 — 폐기

Task #1010 Stage 2 의 정량 입증 + cross-correlation 결과로 **line_seg 합성 옵션 B' 시도 가치 낮음**. 다른 root cause 조사 필요 (별도 task).

---

## 3. D 시도 종합

| 항목 | 정량 결과 | 결정 |
|------|----------|------|
| Stage 4-A (page break vpos==0) | Recall 31.6%, FP 4개 | 폐기 |
| Stage 4-B (line_seg 합성 옵션 B') | Task #1010 Stage 2 회귀 + root cause 무관 | 폐기 |

**결론**: D 옵션 두 가지 모두 본 task scope 내 안전한 fix 불가능. 정량 데이터로 부적합 단언.

---

## 4. 최종 처리 방향 (재확인)

옵션 A — 본 task Stage 3 까지로 종료 + 잔존 이슈 별도 등록:

1. **HWP5 변환본 page_break_before 정보 100% 손실**: 한컴 변환기 quirk. text pattern / head_type / vpos==0 휴리스틱 모두 false negative 위험 큼. 깊은 휴리스틱 설계 + 광범위 검증 필요 → 별도 task 권고.

2. **HWP5 변환본 p23 외곽선 overflow**: Task #1010 Stage 1 cross-correlation 으로 **root cause 가 line_seg missing 이 아님** 단언. 다른 root cause (예: composer fallback line_height 자체, paragraph height measure/render 미정합) 조사 필요 → 별도 task 권고.

---

## 5. 다음 단계 (Stage 5)

- Task #1037 종료 결정 (Stage 3 dialog fix 까지 핵심 효과 보존)
- 최종 보고서 갱신 (Stage 4 negative result 추가)
- 잔존 이슈 2 개 별도 등록
- PR 생성

`feedback_visual_judgment_authority` 정합 — 정량 데이터 기반 결정 권한 작업지시자.
