# Phase 1 보고 — #1658 vpos-snap 이식 시도·핵심 진실 확정 (소스 무회귀)

- 브랜치: local/task1658 / 일자: 2026-06-29 / 결과: 실험으로 핵심 메커니즘 확정, 동작 fix 미landing(소스=upstream).

## 1. 실험 A — 비-TAC 블록 표 직후 vpos_page_base 유지
- 변경: 블록 표 직후 base 무효화를 건너뜀(다음 문단이 저장 vpos 스냅하도록).
- 결과: **양 게이트 불변**(소형 75, 대형 441). TypesetEngine vpos_snap 은 `!has_table` 에만 호출되고
  HeightCursor lazy 경로가 동일 결과 → 효과 없음. **되돌림.**

## 2. 계측으로 확정한 핵심 진실 (RHWP_TYPESET_DRIFT, PC셧다운)

| 문단 | 저장 vpos(한글) | rhwp cur_h | 차이 |
|------|----------------|-----------|------|
| pi2 | 286px | 533.7 | **+248** |
| pi5 | 473px | 720.3 | **+247** |

- 차이 ~247px = **pi1(페이지 하단 표 높이)**. rhwp 가 하단 표를 flow 에 누적 → cur_h 가 정확히 표
  높이만큼 과대. 한글 저장 vpos 는 하단 표를 flow 에서 제외(하단 고정).

## 3. 결정적 발견 — Paginator 의 스냅은 over 를 못 고친다
- Paginator snap(engine.rs:626)은 `if vpos_h > current_height { current_height = vpos_h }` —
  **증가 전용(max)**. PC 는 cur_h(533) > 저장(286) 로 **감소**가 필요 → **스냅으로 안 고쳐짐**.
- ∴ Paginator 가 PC=1 을 맞히는 건 스냅이 아니라 **하단 표를 애초에 flow 에 안 넣기**(out-of-flow).
- 이는 #1653 pattern-B out-of-flow 와 동일 → **단일 하단표 문서가 −1 로 회귀**하는 그 문제로 회귀.
  (Paginator 전역 44/92 의 −1 폭증과 정합.)

## 4. 종합 — 안정 해법의 형태 확정
- vpos-snap(증가 전용) 이식만으로는 블록 표 over 해소 불가(이번 Phase 1 으로 확정).
- 유일하게 한글에 맞는 모델 = **per-page 하단 reserve + 올바른 페이지 배정**:
  하단 표를 flow 에서 빼되(over 해소), 그 높이를 *해당 페이지 하단*에 예약(단일표 문서 −1 방지).
- 6종 실패의 불안정 원인 = **하단 표의 reserve 가 자기 자신의 앵커를 밀어내는 결합(진동)**.

## 5. 다음(Phase 2) — decoupled 2-pass 알고리즘 (구체화)
오실레이션을 깨는 핵심: **하단 표의 자기 reserve 가 자기 앵커의 페이지 결정에 영향 주지 않게 한다.**
1. Pass 0: 하단 표 out-of-flow(reserve 0)로 전체 조판 → 각 하단 표 앵커 페이지 `P_T` 확정.
2. per-page reserve 맵 = Σ(하단 표 높이) by `P_T`.
3. Pass 1: reserve 적용하되 **하단 표 앵커는 pass0 의 `P_T` 에 고정**(pass1 reserve 로 재이동 금지).
   다른 flow 콘텐츠만 reserve 로 제약 → 진동 차단.
   - 구현 키: pass1 에서 하단 표 만날 때 `st.pages.len()` 이 아닌 **고정 `P_T`** 로 기록·검증.
     앵커 para 가 reserve 때문에 다른 페이지로 밀리면 강제로 `P_T` 로 당김(또는 reserve 를 P_T 에만).
- 게이트: 소형(75)·대형(441) 양쪽 무회귀 + PC/관악 해소 확인하며 점진.

## 6. 상태
- 소스 무회귀(typeset.rs=upstream). 양 게이트 베이스라인(75 / 441) 유지.
- Phase 1 = 메커니즘·해법형태 확정. 동작 fix 는 Phase 2(decoupled 2-pass)에서.
