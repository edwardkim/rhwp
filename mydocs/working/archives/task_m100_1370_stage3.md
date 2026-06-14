# Task #1370 Stage 3 단계별 보고서 — 2023_sep 잔여 cascade 진단 (음성 결과 3건)

- **이슈**: #1370 (M100) / 브랜치 `local/task1370`
- **단계**: Stage 3 — 잔여 7건(2023_sep 중심) cascade 진단·해소 시도
- **결과**: 근본 규명 완료. 3개 레버 모두 **무효 또는 비단조 회귀** → 코드 Stage 2 클린 상태 유지.

## 1. 잔여 7건 근본 — 조기 advance(under-fill)

Stage 2(snap-off, over-fill 해소)와 **반대 방향**. 2023_sep 은 단을 **일찍 비우고 advance**한다.

실증 (2023_sep page15):
- A3 page15 단1 = pi=798 에서 종료, used=**892.2px** (available 1001.6, **~109px 여유**) 인데 advance.
- B page15 = pi=810(split)까지 **12 para 더** 채움 → page16 단0 가 pi=810 부터 시작, 문23 제목(812)
  이 상단 y≈147. A3 는 page16 단0 가 pi≈795 부터라 812 가 y≈799 로 침하(#11 실패).

**트리거**: `advance_for_fit`(typeset.rs:3381) = `current_height + en_fit > available`.
- pi=799 는 **빈("(빈)") para 인데 vpos forward-jump**(696864..702270 = 5406HU)로 `en_fit=115.3px`
  로 과대추정(saved-vpos delta). 892 + 115.3 > 1001.6 → 조기 advance.
- 그러나 **정확 sim(EN_COLSIM)은 pi=799 를 18px 로 올바르게 compact 렌더**(items18→19: 791.9→809.9).
  hancom 도 forward-jump 빈 para 를 흡수(gap)해 compact 배치.

→ **2022 는 exact-snap 이 과대(over-fill), 2023 은 saved-delta en_fit 이 과대(조기 advance)**.
hancom 은 두 추정 중 **작은 값**에 가깝다.

## 2. 시도한 레버 3건 — 전 exam 동시 측정 (음성)

| # | 레버 | A3 issue_1139 | 1082 | 판정 |
|---|------|--------------|------|------|
| L1 | rewind acc override(2684) A3→compact(acc_legacy) | 7 failed (불변) | 5/5 | **무효** (used 961→943, 분배 불변) |
| L2 | `advance_for_fit` overflow 를 정확 sim 으로 전역 교체(A3) | **23 failed** (+16) | 5/5 | **비단조 회귀** |
| L3 | L2 를 빈 para(`!visible_text`)에 한정 | **15 failed** (+8) | 5/5 | **비단조 회귀** |

`advance_for_fit` 을 정확 sim 으로 바꾸면 pi=799 조기 advance 는 막지만, 다른 단에서 **필요한
advance 를 억제**해 over-fill 회귀(L2 16건, L3 8건). en_fit 기반 advance 는 대다수 케이스에 필요 —
para 별 높이 추정이 전 게이트에 정밀 결합돼 있어 한 범주만 바꿔도 광범위 ripple.

## 3. 결론 — 비단조 핵심 잔여 (메모리 `tech_endnote_overflow_nonmonotonic_gate` 정합)

잔여 7건(2023_sep ×6 + 2024_between20 page18 ×1)은 **forward-jump 빈 para 의 높이 추정**이라는
단일 근본을 갖지만, 그 높이를 hancom-compact 로 고치는 **모든 단순 레버가 타 단의 advance 결정을
비단조로 흔든다**. #1363 v3 가 "잔여(별도 후속)"로 둔 정확한 이유.

원칙적 해법은 forward-jump 빈 para 의 break-결정 높이를 **min(saved-delta, 정확 sim)** 으로 보되,
그 적용 조건을 전 exam(1082/1139/1189/1209/1284) 동시 green 으로 만족시키는 **다조건 게이트 모델**을
다회차 탐색하는 것 — 본질적으로 큰 탐색 공간이다.

## 4. 권고

Stage 2 의 **6/13(무회귀)** 을 확정 인도하고, 잔여 7건(2023_sep cascade)을 다음 중 하나로:
- (A) **후속 타스크 분리** — 부분 green 인도 + 본 진단을 시드로 다회차 탐색. (권장)
- (B) 본 타스크에서 다조건 게이트 모델 다회차 탐색 계속(고비용·고위험).

## 5. 코드 상태

src/renderer/typeset.rs 는 **Stage 2 커밋 상태 그대로**(L1~L3 전부 revert). 본 단계는 진단 문서만.
