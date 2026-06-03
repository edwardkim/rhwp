# Task #291 Stage 3 — 회귀 검증

## 5샘플 byte-diff 스캔

| 샘플 | 변경/전체 | align 분포 | 평가 |
|------|-----------|------------|------|
| **KTX.hwp** | 1/1 | TAC + Right | ✅ 의도 (이슈 목표) |
| **exam_math.hwp** | 0/20 | - | ✅ 무영향 |
| **21_언어_기출_편집가능본.hwp** | 0/19 | - | ✅ 무영향 |
| **aift.hwp** | 18/74 | TAC + Center/Right | ✅ 모두 의도된 개선 |
| **biz_plan.hwp** | 1/6 | TAC + Center | ✅ 의도된 개선 |

### aift.hwp 18페이지 변경 페이지 분석

| 페이지 | section.pi | align |
|--------|------------|-------|
| 6 | 2.75 | Center |
| 11 | 2.126 | Center |
| 15 | 2.145 | Center |
| 27 | 2.394 | Right |
| 32 | 2.444 | Center |
| 43 | 2.566 | Right |
| 50 | 2.694 | Center |
| 51 | 2.715 | Center |
| 52 | 2.719 | Center |
| 54 | 2.751 | Center |
| 56 | 2.775 | Right |
| 57 | 2.786 | Right |
| 59 | 2.802 | Right |
| 60 | 2.809 | Right |
| 61 | 2.816 | Right |
| 62 | 2.828 | Right |
| 63 | 2.843 | Center |
| 64 | 2.849 | Right |

→ **모든 변경 페이지가 align=Center 또는 align=Right 인 TAC 표**. 이전에는 좌측에 붙어있던 것이 이제 의도대로 정렬됨.

### biz_plan.hwp 1페이지 변경

- pi=2 TAC 표, align=Center
- 좌측 x: 68.03 → 70.01 (1.98px 이동, 미세)

## 한컴 PDF 비교 (aift 6페이지 예시)

aift.hwp 의 한컴 PDF 가 코드베이스에 없어 직접 비교는 불가하나, **수정 후 좌표가 align=Center 정의에 정확히 일치** (body 중앙) 함을 수치로 확인:

- aift 6페이지 pi=2.75 표:
  - body_area: x=75.6, w=642.5
  - 표 폭: 610.8
  - 기대 중앙 = 75.6 + (642.5 - 610.8)/2 = **91.45**
  - 실측: x=**91.47** (오차 0.02px)

## 종합 회귀 결과

- **회귀 0건**
- **개선 20건** (KTX 1 + aift 18 + biz_plan 1)
- 모든 변경이 ParaShape `align=Right/Center` 패턴으로 설명됨

## 브라우저 시각 검증

- WASM Docker 빌드: 11:59 재생성
- rhwp-studio 에서 KTX.hwp 1페이지 시각 확인
- 작업지시자 판정: **회귀 없음** ✅

## Stage 3 결론

- 정량 회귀: byte-diff 의 모든 변경이 align 패턴으로 설명 → 회귀 0
- 정성 회귀: 작업지시자 브라우저 직접 확인 → 회귀 0
- 본 PR 의 효과: align=Right/Center TAC 표가 한컴 기대 위치로 정렬됨 (KTX 외 다수 개선 포함)

다음 단계 (Stage 4): 최종 보고서 + 트러블슈팅 등록
