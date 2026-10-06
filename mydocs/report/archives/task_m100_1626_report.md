# 최종 결과보고서 — Task #1626 (잔여 +1쪽 조사 — tolerance 재도입 net-negative 확정)

**제목**: 잔여 +1쪽 (#1608 tolerance 제거 후 native) 조사
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1626 · **브랜치**: `local/task1626` (base: `local/task1624`)

## 1. 잔여 +1쪽 3건 — 이질적 2 클래스

| 케이스 | 한글/rhwp | 클래스 |
|--------|------|------|
| 36382819 | 3/4 | **tac 표(8x3) 3.1px 페이지 초과** (page2 used 933.6 > body 930.5) cascade |
| 36389312 | 1/2 | **footer Page+Bottom**(vert=쪽 valign=Bottom) 선언높이 overshoot (body+footer > page, 한글은 1쪽 fit) |
| 36398366 | 1/2 | **footer Page+Bottom** 동일 |

"#1608 native tolerance" 단일 클래스가 아니라 tac 오버플로 1 + footer overshoot 2 의 혼합.

## 2. tolerance 재도입 실험 — **단조 net-negative 확정**

universal last-line tolerance(`RHWP_EXP_TOL`) 별 통제셋 일치:
| tolerance | 일치 | Δ |
|-----------|------|---|
| 0px(현재) | **75** | — |
| 3px | 74 | −1 |
| 5px | 73 | −2 |
| 8px | 72 | −3 |
| 12px | 71 | −4 |

**어떤 양수 tolerance도 일치를 떨어뜨림** — tolerance 는 available 높이↑ → 콘텐츠 조밀 → 페이지↓
로, +1(3건)을 고치는 것보다 −1(12건)을 더 많이 악화. **#1608 의 tolerance 제거가 옳았고
재도입은 잘못된 방향**임이 데이터로 확정.

## 3. footer overshoot 2건 (36389312·36398366)
footer Page+Bottom 에서 body+footer(선언) > page → rhwp push, 한글은 1쪽 fit(선언보다 작게
렌더/overlap 추정). #1611 declared-height 와 #1624 gap-guard 로도 미해소 — #1616 에서 확정한
**footer push/keep 기하 규칙 부재(razor-thin)** 의 +1 방향. 깔끔한 판별자 없음.

## 4. 판정 — 코드 변경 없음
- tolerance 재도입: net-negative(실험 확정) → 불가.
- footer overshoot: razor-thin 규칙 부재(#1616) → 깔끔한 fix 없음.
- 36382819 tac 3.1px 오버플로: tolerance 류 보정 필요하나 net-negative.

→ **잔여 +1 3건은 단일 net-positive fix 없음.** 양방향 razor-thin 한계의 +1 측면으로 확정,
알려진 한계 보존. 통제셋 81.5%(75/92) 유지.

## 5. 산출물
- 분석: `output/poc/defect_classes.py`, 본 보고서
- 코드 변경: **없음** (tolerance 실험 코드 revert)
