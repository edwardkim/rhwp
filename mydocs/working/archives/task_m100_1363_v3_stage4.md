# Stage 4 (v3) Part A — simulate↔render 정합: 발산 카탈로그 + 1차 정합

break 게이트 재설계의 선결인 **simulate 를 render 와 정합**을 착수했다. 렌더 측 계측을 추가해
per-para 발산을 정량화하고, 지배적 발산(leading 컨트롤-줄)을 정합했다. 비단조 cascade 로 전-문서
동시 green 은 미달 — Part A 는 다회차 alignment 가 필요함을 실증.

## 1. 계측 — `EN_RENDER` (렌더 상대-프레임 per-item)
`layout.rs build_single_column` 미주 단 루프에 `EN_RENDER pi= y_in_rel= y_out_rel= dy= col_h=`
로그 추가(`RHWP_EN_SSOT_DEBUG=1` 게이트). simulate 의 `EN_MEASURE` 와 동일 상대 프레임이라
para 단위 직접 비교 가능.

## 2. 발산 카탈로그 (sep20/20 단 935..963, render bottom=1065.5 / col_h=1001.6, ~64px overflow)

| 발산 | 예시 | 크기 | 원인 | 상태 |
|------|------|------|------|------|
| **(a) leading 컨트롤-줄** | pi=936 measured 127.7 ↔ render 101.3 | **+26px** | 렌더 full-para 경로는 `text_start_line`(수식 객체마커 ￼ 줄)을 건너뜀, scratch 는 0 부터 | **정합 완료** |
| (b) trailing-ls 잔차 | pi=936 정합후 107.3 ↔ render 101.3 | +6px | 다음 para 의 vpos forward-jump 가 trailing 을 덮는지 여부 | 잔존 |
| (c) item-type 불일치 | pi=936 simulate Partial{0,N}(127.7) ↔ render Full(101.3) | +26px | simulate 의 tentative 분할 hypothesis 가 최종 render 와 다름 | 잔존 |
| (d) vpos forward-jump | pi=937 render y_in=145.7(저장 vpos 점프) ↔ sim 125.3 | ~20px | 렌더는 저장 vpos 로 forward-jump, sim 의 vpos_adjust 는 다른 보정 | 잔존 |

→ 누적 결과 sim 이 render 보다 ~55px 낮음(under-count) → fit 게이트가 961 overflow 를 fit 으로 오판.

## 3. 1차 정합 — `text_start_line` 스킵 (정합 (a))
`measure_endnote_para_advance`: FullParagraph 측정 시 렌더 `has_real_text` 경로처럼 leading
컨트롤-전용 줄을 건너뛰고 `text_start_line` 부터 측정. 객체-전용 para(TAC 그림)는 0 유지(렌더 동일).
Partial 은 항목 지정 줄 그대로(렌더 partial 경로 동일).

- pi=936 measured 127.7 → **107.3**(render 101.3 에 근접, 잔차 +6=trailing).

## 4. overflow 결과 — 비단조 cascade 재확인

| 문서 | 정합 전 A3 | **정합 (a) 후 A3** |
|------|-----------|------------------|
| sep20/20 | 23.5px | 23.5px (불변 — 잔여 발산 (b)(c)(d) 지배) |
| 3-09_2022 | (fail) | 183.4px (악화) |
| 3-11_2022 | (fail) | **0.0px** (해소) |

→ **개별 정합은 옳으나(render 일치) cascade 로 문서별 명암**. 전-문서 동시 green 은 **모든 발산
((a)~(d)) 동시 정합 + 게이트 재설계(Part B)** 후에야 가능. 단일 정합으로는 비단조 reshuffle.

## 5. 검증
- **기본 B 무회귀**: `issue_1082` 5/5(정합은 `measure_endnote_para_advance` = A3 전용, B 무영향).
- A3 = 실험 opt-in. fit-parity 미달(정합 진행 중).

## 6. 상태 + 다음
- **정합 (a) 커밋**: render 충실 측정으로의 옳은 groundwork. `EN_RENDER` 계측 상주(gated).
- **다음(Part A 계속)**: 발산 (c) item-type(simulate tentative Partial↔render Full)·(d)
  vpos forward-jump 를 렌더 dispatch 와 일치시켜 sim==render bottom 달성 → 그 위에서 Part B
  게이트 단일화. 비단조라 **전 exam(1082/1139/1189/1284) 동시 green** 을 최종 게이트로, 다회차.
- 측정 인프라(Stage 1-2)·정합 (a) 견고, B 무회귀 유지.
