# Task #1377 Stage 2 — 발산 근원 최종 국소화 (blank-line 전진) + 시도 음성

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 2 — TAC 그림 발산의 정확한 코드 경로 규명 + 수정 시도
- **결과**: 근원을 **빈 para 의 line_spacing 전진**까지 국소화. 3개 lever 시도 전부 **no-op**(경로 불일치).
  코드 클린(시도 전부 revert).

## 1. 최종 국소화 (sep2020 p22 단0)

| pi | 내용 | render 전진 | typeset acc | 발산 |
|----|------|------------|-------------|------|
| 1127 | "(ⅰ)~(ⅲ)에서" | — | — | 0 |
| **1128** | **"(빈)"** | **54.1px** | 33.7px | **+20.4px** |
| 1129 | "문29)" + TAC 그림 | 22(자체) | 21.9 | 0(자체) |
| 1131 | TAC 그래프 | — | — | +6 |

**핵심**: pi=1128(빈 para)의 **렌더 전진 54px ≈ file saved-vpos 갭**(1264088→1266610 = 2522HU ≈ 53.4px).
typeset/hancom 은 33.7px 로 compact. 즉 **빈 para 의 blank-line line_spacing 을 렌더가 file 값(54px)대로
전진하나 hancom 은 줄인다.** pi=1129 의 vpos_adjust 는 no-op(y_before==y_after) — 발산은 pi=1128 전진.

→ 코드 경로: `paragraph_layout.rs` 의 line 전진(`line_height + line_spacing`). 게이트/vpos_adjust/title-gap 아님.

## 2. 시도한 lever 3건 (전부 no-op — 경로 불일치)

| lever | 위치 | 결과 |
|-------|------|------|
| TAC-그림 title-gap compaction(#1355 OR 조건) | layout.rs:3310 | no-op (`prev_endnote_title_gap_px>0` 게이트 미도달) |
| vpos_adjust 직후 그림+textless+forward clamp | layout.rs:3211 | no-op (pi=1129 vpos_adjust 가 이미 no-op) |
| 빈 para controls.is_empty clamp(#1375 v2) | layout.rs:3219 | no-op (빈 para 가 controls 보유; 전진은 별 경로) |

→ 셋 다 **pi=1129 의 vpos/title 경로**를 짚었으나 발산은 **pi=1128 의 blank-line 전진**. 수정점은
line-layout 의 blank-line height/spacing 산출(broad·delicate, 전 문서 빈 줄 영향).

## 3. 현황 / 권고

- **국소화 성과**: #1363~#1375 5개 타스크가 못 잡던 render↔typeset 발산을 **빈 para blank-line
  line_spacing 전진**까지 정확히 추적(이 자체가 큰 진전).
- **미해결**: 실제 수정은 blank-line line_spacing 을 hancom compact 로 정합 — line-layout 의 broad 경로라
  전 문서 빈 줄 렌더 영향, 골든 SVG 회귀 위험 큼. 좁은 endnote-blank 한정 조건 + 골든 가드 필수.
- **권고**: 이 정밀 국소화를 시드로 **별도 집중 작업**(line-layout blank-line 전진의 endnote 한정 compact).
  본 세션은 #1375 의 p22 overflow 무회귀 부분해소(tolerance 24→6, 커밋 `821a8b32`)까지 확정.

## 4. 추가 시도 (line-layout 직접) — 음성

`EN_RENDER`(layout.rs:3486) 계측으로 per-para 렌더 전진 확정:
| pi | render dy | typeset adv | diff |
|----|-----------|-------------|------|
| 1127 | 18.0 | 18.0 | 0 |
| **1128(빈)** | **54.1** | 33.6 | **+20.5** |
| 1129(그림제목) | 22.0 | 22.0 | 0 |
| 1131(그래프) | 315.2 | 309.2 | +6.0 |

**lever 4**: `layout_composed_paragraph` 의 endnote 빈 줄 `line_spacing_px.min(line_height)` 캡
(paragraph_layout.rs:1741) → **no-op(pi=1128 dy 54.1 불변) + issue_1139 2건 회귀**. 즉 pi=1128 의 54px
는 **line_spacing 이 아니다** — 캡은 다른 빈 줄만 깬다.

→ pi=1128 54px 의 출처는 vpos_adjust(no-op)·line_spacing(no-op)·게이트(no-op) 모두 아님. 남은 의심:
**pi=1129 TAC 그림(ci=1) reserved 높이가 pi=1128 영역에 귀속**되거나 spacer-fill 경로. 4+ 시도로 미규명.

## 5. 코드 상태 / 정직한 한계

src 클린(시도 전부 revert). 유효 수정 = #1375 `821a8b32`(tolerance) + #1377 Stage1/2 진단.

**정직한 한계**: 발산을 pi=1128(빈 para) 54px 전진까지 정밀 국소화했으나, 그 54px 를 만드는 **정확한
코드 경로를 4+ 시도로도 못 짚음**(vpos/line_spacing/게이트 모두 아님). 다음 의심(그림 reserved 귀속)은
shape 배치 코드의 깊은 추적이 필요. 본 진단을 시드로 **렌더러 아키텍처 컨텍스트가 더 있는 집중 세션**에서
shape-reserved↔blank-para 상호작용을 파는 것을 권고. 무리한 블라인드 수정은 회귀(이번 2건)를 반복.
