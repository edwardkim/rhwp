# 최종 보고서 — Task M100 #2026: 1차 리팩토링 라운드 5 (typeset_section_endnotes 해체)

- 이슈: #2026 (계획 #1883 v2, umbrella #1582, 선행 #1904·#1925·#2001·#2003)
- 기간: 2026-07-07 / 거버넌스: v2 전 조항 + r4 스캔 사각지대 보정 절차 적용

## 1. 결과 요약 (공식, 스냅샷 `2026-07-07-r5/`)

| 함수 | 이전 | 이후 |
|---|---|---|
| `typeset_section_endnotes` | **CC 179 (전체 1위) · 5,539줄** | **CC 6 · 141줄** (오케스트레이터) |
| `prepare_endnote_emit` (신규) | — | CC 22 · 519줄 (경계 미만!) |
| `typeset_endnote_paragraphs` (신규) | — | CC 153 · 4,397줄 (다음 분해 대상) |
| 전체 최대 CC | 179 | **169** (`layout_partial_table`) |

**전체 1위 함수 5라운드 연속 해소**: 282→104, 288→226, 234→76, 226→146, **179→6**.
행동 회귀 통산 **0건** (테스트 2,924/0 · issue_1116 13/13 · OVR 추가 변동 0 · 매 추출 전수).

## 2. 산출물 — #1904 라운드 1 이연 설계의 완결

- **`EndnoteFlowState`** (Copy 7필드): 미주-간 흐름 캐리(vpos_offset/prev_en_bottom ×2/
  emitted_count/last_render_idx/rewind·overestimate) — 라운드 1이 "의존 32/mut 9"로
  이연했던 설계가 재실측(32/**8** — 오탐 1 보정)으로 확정·구현됨.
- **`EndnoteEmitVars`** (8필드) + **`EndnotePrepCarry`** (6필드): 미주-당 읽기 묶음 /
  프리앰블 캐리. `pre_emitted` HashSet은 실측(루프 후 미사용)으로 함수 로컬 흡수.
- 최종 구조: `resolve → prepare_endnote_emit(구분자·플래그) → typeset_endnote_paragraphs
  (en_para 조판) → 꼬리 스왑` — 명시적 파이프라인.

커밋: `bc1da05e`(수행계획) → `fe68f41a`(분석·설계) → `4bf7ed83`(추출 1) → `14cdfacb`(추출 2).

## 3. 재평가

- CC>25: 86 → **86** (변동 0 — 1위 해소분과 신규 body 진입이 상쇄; 과도기 누적은 본체
  분해가 진행되며 회수 예정).
- 전체 최대 169 = `layout_partial_table` — **r4의 163에서 +6 성장** (07-06~07 PR 유입,
  #2016이 table_partial 접촉). 기능 유입에 의한 재성장 추세가 표 계열로 확산 — 감시 지속.
- `typeset_endnote_paragraphs`(153)가 전체 2위 — 내부는 en_para-당 조판 단계(분할/앵커/
  vpos 보정)의 연쇄로, **다음 라운드 후보 1순위**.

## 4. 다음 라운드 후보

1. `layout_partial_table`(169, 현 1위) — 표 계열 첫 진입 + #2016 유입분 흡수
2. `typeset_endnote_paragraphs`(153) 내부 분해 — EndnoteFlowState 기반 후속
3. `layout_composed_paragraph`(146) 잔여 / `typeset_block_table`(127)
4. 도구: 의존 스캐너의 오탐 2종(클로저 파라미터, 필드 리터럴) 자동 배제 개선

## 5. 산출물 위치

계획 `plans/task_m100_2026{,_impl}.md` / 단계 보고 `working/task_m100_2026_stage{2,3}.md` /
스냅샷 `mydocs/metrics/2026-07-07-r5/`
