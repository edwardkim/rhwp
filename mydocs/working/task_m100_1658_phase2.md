# Phase 2 보고 — #1658 decoupled 2-pass: 진짜 fixpoint 확정 + 전략 전환 (소스 무회귀)

- 브랜치: local/task1658 / 일자: 2026-06-29 / 결과: 구현+계측으로 난점·ROI 확정. 동작 fix 미landing(소스=upstream).

## 1. 구현 (decoupled 2-pass + 안정 인덱스)
- `page_seq`(실제 페이지 나눔마다 +1, ensure_page 무관 안정 키) 도입 → reserve 맵 키.
- pass0(예약 없음, 하단 표 out-of-flow) → 페이지 확정 → pass1(per-page reserve 적용).
- reserve 에 4px 허용오차(한글 rounding 관용 모사) 부여.

## 2. 계측으로 확정한 두 난점 (page_seq 디버그, PC셧다운)

### (a) px-level close-call
- 콘텐츠 442.5 + 하단표 549.6 = **992.1 vs body 990.2 = 1.9px 초과**.
- 한글은 1쪽에 수용(rounding/overlap 관용). rhwp 는 reserve(990−549.6=440.7) + typeset_paragraph
  자체 safety(−4) 로 **이중 보수화** → 콘텐츠가 못 들어가 pi6 가 page2 로 → 2쪽.
- 4px 허용오차도 자체 safety 와 상쇄 → 미해결. px 튜닝은 취약.

### (b) 페이지 배정 fixpoint (근본)
- pass0 **out-of-flow** → 하단 표 페이지 **과소 배정**(콘텐츠만으론 1쪽 → reserve 로 1쪽 →
  한글이 2쪽인 문서 **−1 회귀**). 소형 게이트 75→63(−1: 12→24).
- pass0 **flow** → 과대 배정(현행 over 그대로).
- **어느 pass0 모드도 fixpoint 를 풀지 못함** — 하단 표의 올바른 페이지는 reserve 가 활성인
  상태의 flow 결과인데, 그 reserve 가 다시 페이지에 의존(순환).

## 3. 전략적 발견 — ROI

| 게이트 | 베이스 | Phase 2 변경 후 |
|--------|--------|----------------|
| 소형 controlset(난케이스) | 75 | **63 (회귀)** |
| 대형 오라클(랜덤452) | 441 | **441 (불변)** |

- 하단 표 문제는 **controlset(큐레이트된 희귀 케이스)에 집중**, **스케일에선 무영향**.
- 대형 오라클 over 지배항은 **`law 별표 .hwp` 표 행분할**(Δ최대+8) — 하단 표와 **다른 메커니즘**.
- ∴ 하단 표 fix 는 **고난도·저ROI**. 스케일 개선의 우선순위 아님.

## 4. 결론·권고

- 하단 표 페이지 배정 = **px-level close-call + 배정 fixpoint** 의 진짜 연구난제. 안정 단일/2-pass
  reserve 로는 controlset 회귀 없이 해결 불가(Phase 0~2, 8종 시도로 확정). 소스 무회귀 유지.
- **전략 전환 권고**: 다음 작업은 **(B) 법령 표 행분할(대형 오라클 over 지배)** 또는
  **(A) 하단 표를 px-tolerant 모델 + 정확 배정**(저ROI) 중 택일.
  - **권고: B(법령 표 행분할)** — 스케일 빈도↑, 별도 메커니즘이라 fixpoint 무관일 가능성. 또는
  - 현 시점 페이지수 정합 97.6% 를 수용하고, 시각 충실도(셀/표 내부)로 우선순위 이동.

## 5. 상태
- 소스 무회귀(typeset.rs=upstream). 양 게이트 베이스라인(75/441).
- Phase 0(오라클·설계)·1(메커니즘)·2(fixpoint·ROI) 누적 기록 = 향후 작업 토대.
