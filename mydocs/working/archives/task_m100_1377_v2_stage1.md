# Task #1377 v2 Stage 1 — phantom between-notes trailing 정합 구현 + fidelity 달성

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 1 — 판별 게이트 + 치환값 구현, sep2020 fidelity 격리 검증
- **결과**: pi=1128 phantom 발산(+20.5px) **완전 해소**. 전체 회귀 가드는 Stage 2.

## 1. 구현 (`paragraph_layout.rs`, +34줄)

- **헬퍼 추가**: `para_is_treat_as_char_equation_only(para)` (빈 텍스트 + TAC 수식 only).
- **치환 override** (per-line advance 직전): 미주 흐름 빈 수식 spacer 의 line_spacing 이
  between-notes 마커일 때 정상 base-Percent line-spacing 으로 치환.
  - **게이트 (AND)**: `btw_hu>0` + `is_endnote_virtual_para` + `para_is_treat_as_char_equation_only`
    + `|ls_hu − btw_hu| ≤ 200` + `Percent` 모드.
  - **치환값**: `base_fs × (ls_val − 100)/100` (= 150% 에서 base_fs 12px × 0.5 = 6px = 452HU).
    `base_fs` 는 `paragraph_active_text_style(styles, para, 0)` (런 없는 빈 para 도 char-shape 폰트).
- **계획서 대비 조정**: 게이트에서 "same-endnote successor" 조건 **제거**. probe 결과 pi=1128 은
  `succ=false`(노트 마지막 para)였고, `eqonly + ls≈marker` 만으로 충분·정확한 판별이 확인됨.

## 2. 정밀 진단 데이터 (probe, 코드 무수정 도구로 재확인)

| pi | 성격 | ls_hu | base_fs | repl_ls |
|----|------|-------|---------|---------|
| 1127 | 텍스트 | 452 | 12 | (게이트 미충족: ls≠marker) |
| **1128** | **빈 TAC수식 only** | **1984(=btw)** | **12** | **6.0px(452HU)** ← 치환 |
| 1129 | 텍스트+TAC그림 | 452 | 12 | (미충족) |
| 1131 | TAC그래프 | 452 | 12 | (미충족) |

→ 이웃은 `ls_hu=452 ≠ btw_hu=1984` 라 게이트 자동 제외. pi=1128 만 치환.

## 3. fidelity 검증 (sep2020 EN_RENDER, rel)

| pi | clean | v2 | typeset(EN_ACC) | 판정 |
|----|------|------|------|------|
| 1128 advance | 54.1 | **33.6** | 33.6 | ✅ 정합 |
| 1129 시작 | 72.1 | **51.7** | 51.7 | ✅ typeset top 정합 |
| 1130 advance | 36.1 | 36.1 | 36.1 | ✅ |
| 1131 bottom | 445.3 | **424.9** | 418.9 | ◐ +6px 잔차 |

- pi=1128 phantom(+20.5px) **완전 해소**, pi=1129 가 정확히 typeset top(51.7)에 안착.
- **잔차 +6px = pi=1131(TAC 그래프) 자체 trailing**(render 315.2 vs typeset TACstack 309.2).
  phantom between-notes 와 무관한 **별개 발산**(TAC 그래프 trailing line_spacing) → v2 범위 밖.
  clean(+26px) 대비 dominant +20px 제거, 잔차는 별개 소발산.

## 4. 코드 상태 / 다음

- probe 전량 제거, 빌드 경고 0. diff = paragraph_layout.rs +34줄 단일.
- **Stage 2**: `cargo test` 전체 회귀 가드 (issue_1139 19건 + height_cursor 26 + issue_1082 4 + golden).
  회귀 0 이 채택 조건.
