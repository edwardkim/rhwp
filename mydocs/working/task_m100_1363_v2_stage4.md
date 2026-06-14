# Stage 4 (v2) — 렌더-정합 시뮬레이션 돌파 (두 배치 해결, 재튜닝 잔여)

Stage 3 음성 결과(점진 fit/split 불가) 후, **시뮬레이션 부정확성**이 근본 원인임을 규명하고
시뮬을 렌더-정합화하여 **p17 C×C·p21 pi=1127 두 배치를 A2 에서 실제 해결**했다.

## 1. 돌파구 — 렌더러는 저장 line_segs 로 그린다
결정적 발견: 렌더러는 미주 텍스트/수식 para 를 **hancom 저장 `line_segs`**(vpos 레이아웃)로
그린다 — `format_paragraph` 의 **reflow(total_height)가 아님**. 수식 다줄 para 는 reflow 가
저장 span 보다 큼(pi=1126: 237 vs 185.8). Stage 2 시뮬이 total_height 를 써서 단을 과대
계상 → fit 결정 오류.
계상 → fit 결정 오류.

## 2. 시뮬 advance 를 항목 유형별 렌더 높이로 (`simulate_endnote_column_bottom_y`)
| para 유형 | advance | 근거 |
|-----------|---------|------|
| 텍스트/수식 | **저장 line_segs vpos 범위** | 렌더러 정합(reflow 아님) |
| TAC 그림/도형 | total_height | 개체 높이가 line_segs 에 없음(pi=1131 빈텍스트+309 그림) |
| 내부 vpos rewind | line_advances_sum | 렌더 순차 적층(Divergence A, pi=522: saved 32.5 vs 실제 183) |
| 표 | line_segs span / total_height | 표 높이 |

+ fit 게이트(`a2_overflow_with_para`)·단일줄 fit-or-advance 를 렌더-정합 시뮬로 구동.

## 3. 결과 — sep20/20 완전 해결 ✓
| 대상 | DEFAULT(B) | **A2** |
|------|-----------|--------|
| p17 pi=894 "C×C" | 단0(좌단 하단) | **단1 split 2..5 → C×C 우단 ✓** |
| p21 pi=1127 | p22 단0 | **p21 단1 ✓** |
| 전 페이지 overflow | p22 단0(부분) | **overflow 없음 ✓** |
| issue_1082 (5 exam) | 5 pass | **5 pass ✓**(대상 포함) |

**작업지시자가 지적한 두 배치가 A2 에서 실제 해결됨.** `RHWP_EN_SSOT=A2` 로 확인 가능.

## 4. 잔여 — 전 exam 재튜닝 (7건)
A2 전체 cargo test: **7 fail**(그 외 전부 pass). 전부 **타 미주 문서**의 한컴-검증 배치 테스트:
- issue_1139(2022 p17 pi=931 split, 2023 p12/13), issue_1189(2023 p19), issue_1284(2023 p19·
  2024-between20 p18/21/22).
- 시뮬이 sep20/20 수식엔 정합하나, 타 문서의 일부 para(예: 2022 pi=931)에서 한컴 배치와
  미세 차이 → 골든 테스트 회귀. 계획서가 예고한 **"전 exam 재튜닝"** 영역.

## 5. 상태 + 다음
- **DEFAULT(B) 무회귀**(A2 완전 opt-in). 두 배치는 A2 에서 실증 해결.
- 승격(기본화) 전제: 7건 재튜닝 — 각 문서의 한컴 배치를 시뮬이 재현하도록 항목 유형별 높이
  규칙 정밀화. 다회차 잔여.
- 코드: A2 시뮬 렌더-정합화 커밋. fit 게이트·rewind/그림/표 처리 포함.

> 결론: 후보 A(시뮬) 접근이 **두 배치를 실제로 해결할 수 있음을 입증**. 전면 승격은 7건
> 재튜닝 완료 후. 점진 게이트(Stage 3)와 달리 시뮬-정합은 수렴(회귀 3→0 후 타 문서 7건).
