# Stage 2 (v2) — 미주 다단 누적 SSOT 시뮬레이션 배선 (A2)

Stage 1 결론(vpos_adjust 는 예측 불가 → 렌더 y 시뮬레이션 필요)을 구현. 마침 typeset 에
**컬럼 렌더 시뮬레이션이 이미 부분 존재**(predicted_y, 질문제목 overflow 판정용)함을 발견,
이를 정식 메서드로 일반화해 **미주 누적을 시뮬레이션 bottom 으로 스냅**(A2).

## 1. 발견 — 기존 인프라
- typeset 상태에 **vpos 스냅 필드**(`vpos_page_base/lazy_base/col_anchor`, Task #1027 Stage D)
  + 단단 본문용 `vpos_snap_current_height`(col_count==1, "다단은 Stage E") 이미 존재.
- 미주 루프에 **컬럼 시뮬레이션**(typeset.rs ~2942 `predicted_y`)이 이미 있으나 질문제목
  overflow 판정에만 국소 사용. 누적(`current_height += en_advance`)은 여전히 근사.

## 2. 구현 (`src/renderer/typeset.rs`)
- `EnSsotLevel::A2` + `RHWP_EN_SSOT=A2` (기본 B 불변, opt-in).
- **`simulate_endnote_column_bottom_y`**: `st.current_items` 를 렌더러 `build_single_column`
  동일 경로(`HeightCursor::vpos_adjust` + line/total advances)로 재생해 단 bottom y 산출.
- A2 게이트: para 항목 push 후 `current_height = 시뮬 bottom` 스냅 → compute_en_metrics
  saved-delta 근사를 렌더 실측과 정합.

## 3. 측정 — 누적 정합 성공, fit/split 미정합

| 대상 | DEFAULT(B) | **A2** | 평가 |
|------|-----------|--------|------|
| p17 pi=894 "C×C" | 단0 split 0..3(C×C 좌단) | **pi=894 전체 단1 이동(C×C 우단 ✓)** | 누적 정합 → 목표 방향 |
| p17 단1 used | 945.1 | 1087.9 (**+86 overflow**) | fit/split 미정합 부작용 |
| p21 단1 used | 1022.9(과대) | **994.0(렌더 근접)** | 누적 정합 ✓ |
| p21 pi=1127 | p22 단0 | p22 단0(미이동) | split 미정합으로 잔존 |

**핵심**: 누적(`current_height`)은 이제 렌더 실측에 정합(p21 1022.9→994, p17 pi=894 우단 이동).
그러나 **fit/split 결정(`split_endnote_to_fit`/컬럼 break)은 여전히 compute_en_metrics 의
`en_fit` 사용** → 누적과 불일치 → p17 단1 overflow, pi=1127 미이동.

## 4. 검증
- **DEFAULT(B): 전체 cargo test 2137 pass / 0 fail** (A2 완전 opt-in, 기본 불변).
- **A2: 3 fail** — 전부 issue_1082 overflow 가드(sep2020/3-11'22/3-09'22). 그 외 1792 pass.
  회귀가 **overflow 가드에 격리**됨 = fit/split lag 의 직접 결과(질문흐름 등 광범위 회귀 없음).

## 5. 결론 + 다음 (Stage 3)
누적 SSOT 만으로는 불충분 — **fit/split 결정도 시뮬레이션을 써야** 정합. Stage 3:
- `split_endnote_to_fit`·컬럼 break·`en_fit` 을 시뮬 bottom 기반으로 전환(A2 경로 내).
- 목표: A2 에서 issue_1082 3-overflow 해소 + p17 C×C 우단 단1 무overflow + p21 pi=1127 p21 유지.
- 매 단계 DEFAULT 2137 유지 + A2 측정 + 전체 cargo test.

> A2 는 opt-in 실험 경로. 기본 B 는 PR #1368 그대로(무회귀). Stage 3 정합 후 승격 검토.
