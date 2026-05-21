# Stage 3 findings — #1046: 사후 reflow(A)는 본 문서 overflow를 줄이지 못함

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 작성일: 2026-05-21
- 결론: **승인된 접근(A) 사후 reflow는 이 문서에서 overflow를 줄이지 못하며 오히려 늘린다.**
  근본 원인은 페이지네이터↔렌더러 측정 드리프트(보고서 task_m100_993 §4, #1022 잔여)이며,
  이는 항목 이월로 해소되지 않는다(드리프트 보존). → 방향 재결정 필요.

## 1. 구현한 것 (Stage 2~3)

- typeset: `force_break_before` hint → 해당 para 앞 강제 페이지나눔 (단위테스트 통과).
- layout: `LayoutOverflow`에 `section_index`/`is_first_in_column` 기록.
- DocumentCore `paginate()`: 1차 배치 → 전 페이지 layout 측정 → first=false overflow를
  `(구역,para)` hint 누적 → 전 구역 재배치 반복(최대 6회). `RHWP_REFLOW_MIN_PX` 임계값,
  `RHWP_DISABLE_REFLOW` 게이트.

## 2. 측정 결과 (임계값 스윕)

| RHWP_REFLOW_MIN_PX | overflow 건수 | 페이지 수 |
|---|---|---|
| (reflow 끄기 = baseline) | **16** | 185 |
| 1000 (사실상 무동작) | 16 | 185 |
| 100 (pi=567만) | 19 | 186 |
| 30 | 19 | 186 |
| 20 (pi=567만) | 19 | 186 |
| 15 (242·781·567) | 18 | 187 |
| 10 | 20 | 189 |
| 5 | 21 | 191 |
| 0 (전부) | 23 | 195 |

**어떤 임계값도 baseline 16보다 좋지 않다.** 항목을 옮길수록 overflow가 늘고 페이지만 증가.

## 3. 왜 (A)가 실패하는가

### (a) 드리프트 보존 (conservation of drift)
근본 원인은 "몇 항목이 잘못 배치됨"이 아니라 **거의 모든 표-하단 페이지에 퍼진 ~2~19px
측정 드리프트**(페이지네이터 cut/추정 높이 < 렌더러 실측). 페이지마다 평균 ~5px씩 과밀하다.
항목을 다음 페이지로 옮기면:
- 출발 페이지는 줄지만,
- 도착 페이지는 그 항목 + 자기 콘텐츠로 다시 과밀 → 도착 페이지 마지막 항목이 새로 넘침.

즉 overflow가 **이동·증식**할 뿐 사라지지 않는다. threshold=20에서 oversized 항목 1개
(pi=567)만 옮겨도 16→19 (+3) 가 이를 입증한다.

### (b) is_first 가드의 한계 — 숨은 page-larger
pi=567(843px 표 + 누적 1797px)은 위에 작은 제목 pi=566이 있어 `first=false`지만 **본질적
page-larger**다. 이월하면 새 페이지에서 분할되어 두 청크가 모두 넘쳐(92·93쪽) overflow가
오히려 +2. "단의 첫 항목" 신호만으로는 이런 숨은 page-larger를 거르지 못한다(항목 자체
높이 > 본문 높이 판정이 필요).

## 4. 진단 정합
task_m100_993 §4 / #1022 §4 가 이미 지목: `MeasuredCell`(HeightMeasurer) ↔ `cell_units`
줄높이가 같은 셀에 대해 다른 px 합 산출 → 페이지네이터·렌더러 측정 공간 불일치. 이 드리프트가
존재하는 한, 페이지 경계 재배치(=reflow)는 드리프트를 옮길 뿐이다. 해소는 **측정 통일(B)**
로 페이지네이터 cut 높이 = 렌더러 height 가 되어야 가능하다(LAYOUT_OVERFLOW→0).

## 5. 권고 (작업지시자 결정 필요)
- **(B) 측정 통일**(권장, 근본): 페이지네이터 cut 높이 ↔ 렌더러 HeightMeasurer bit 정합.
  #1022 본래 의도. reflow 훅(Stage2)은 통일 이후에도 "본문보다 큰 단일 항목" 한정으로 유용.
- **(대안) 렌더 클램프**: overflow 시 다음 페이지 이월 대신 본문 하단에서 클립(시각적 잘림).
  단 사용자 지시("다음으로 넘긴다")와 다르고 내용 손실.
- **현 reflow 코드 처리**: 기본 비활성(`RHWP_DISABLE_REFLOW` 불필요하게) 또는 "본문보다 큰
  단일 항목"에만 적용하도록 축소 후 (B)와 결합.

## 6. 코드 상태
Stage 2 커밋(98d2df5a)은 무동작 메커니즘이라 안전. Stage 3 reflow 루프는 **미커밋**
(regression 이라 커밋 보류). 방향 재결정 후 정리.
