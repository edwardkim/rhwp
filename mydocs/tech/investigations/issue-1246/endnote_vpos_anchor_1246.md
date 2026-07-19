---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-1246/README.md
last_verified: 2026-07-16
---

# 조사 보고서 — Task #1246 Stage 1: render/pagination 미주 vpos 좌표 정합

작성일: 2026-06-02
대상: #1246 (render 미주 좌표 정합) — #1238(미주 between-notes margin) 흡수
선행: `tech/between_notes_multiline_1238.md §7`, `working/task_m100_1238_stage2.md`

## 1. 핵심 발견 — 수행계획서 프레이밍 정정

수행계획서(`plans/task_m100_1246.md`)는 "render 가 누적 incoming y 로 앵커 → 절대 vpos 앵커로
전환"으로 프레이밍했으나, 코드 정독 결과 **부정확**했다. 실제:

- **render 는 이미 vpos→y 매핑을 수행**한다. `build_single_column` 이 `HeightCursor`
  (`src/renderer/height_cursor.rs`, Task #1027 Stage C)를 통해 항목마다 `vpos_adjust()` 로
  `end_y = col_anchor_y + (vpos_end − base)` 를 계산한다(page_base/lazy_base 경로).
- 따라서 "누적 y vs 절대 vpos" 라는 단순 대립이 아니다. **render·pagination 모두 vpos 기반**이되,
  **서로 다른 코드 경로**로 계산해 어긋난다.

## 2. 진짜 구조 — 두 개의 분리된 vpos 측정기

| | 미주 문단 위치 산정 | 비고 |
|--|--|--|
| **render** | `HeightCursor.vpos_adjust` (`height_cursor.rs`) | compact endnote(`suppress_large_forward_jump`) 특례 **다수 보유** |
| **pagination** | `typeset.rs` `compute_en_metrics` | vpos-delta advance + capped/rewind 특례, **별도 구현** |

- Stage C 주석(`height_cursor.rs` L8): *"Stage D 에서 페이지네이터(typeset)가 동일 커서로
  height-only 패스를 수행하여 두 측정 공간을 일치시킨다."* → **Stage D 미완**. 즉 pagination 이
  HeightCursor 를 쓰지 않고 자체 `compute_en_metrics` 로 측정 → **두 공간 불일치 = drift 원천**.

## 3. HeightCursor 는 이미 between-notes 를 처리한다

`vpos_adjust` 의 compact endnote(`suppress_large_forward_jump=true`) 분기는 미주 사이 gap 을
이미 광범위하게 다룬다(단위테스트로 고정):

| 케이스 | 동작 | 테스트 |
|--------|------|--------|
| `compact_endnote_question_title_caps_large_forward_gap` | 큰 forward 점프를 `stored_gap+40` 으로 cap | L712 (`1984`=7mm 사용) |
| `..preserves_spacing_on_stale_forward_jump` | stale 절대 vpos 버리고 직전 ls(1984)만 보존 | L734 |
| `..after_empty_spacer_keeps_stored_gap_only` | 빈 spacer 뒤 +40 완충 생략 | L755 |
| `..after_tall_line_uses_content_bottom_gap` | 큰 수식 뒤 제목은 내용 바닥+10 | L776 |
| `compact_endnote_safe_vpos_backtrack` (L362) | 단 중간 backtrack 허용(내용 바닥 위) | **= #1209 문12 케이스** |
| `compact_endnote_deep_backtrack`/`title_tail_backtrack` | 하단부 제한적 backtrack(overflow 완화) | L335/L346 |

→ #1238 의 render-클램프가 깨뜨린 `issue_1209_2022_sep_page10_question12` 는 바로
`compact_endnote_safe_vpos_backtrack`(L362). **render 클램프는 이 분기를 우회·침범**했다.

## 4. 무엇이 빠졌나 — 문22(다줄-prev gap=0)

HeightCursor 는 **forward 점프 cap / backtrack** 은 풍부하나, **"stored vpos 가 between_notes
보다 작은 gap(특히 0)을 주는 경우 min-gap 으로 끌어올리는"** 케이스가 없다. 문22(다줄 풀이
다음 제목, 직전 다줄 last seg trailing=0 → vpos gap≈0)가 정확히 이 사각지대다.

- `vpos_end` 산정(L237-243): curr_first_vpos 가 prev seg.vpos 보다 크면 그 값 사용 → gap 이
  stored vpos 그대로. stored gap=0 이면 end_y≈y_offset → 제목이 직전 줄에 붙음(문22 버그).
- forward-cap/ backtrack 분기는 gap 이 **과도할 때** 줄이는 방향만 다룸 → **부족할 때 늘리는
  로직 부재**.

## 5. 수정 위치 결론 (Stage 2 방향)

**전면 좌표 재작성 불필요.** 수정 위치는 `HeightCursor.vpos_adjust` 의 compact endnote 분기:

1. **min-gap 케이스 추가**: compact endnote 에서 새 미주 첫 문단(제목)이 forward 흐름이고
   `end_y − prev_content_bottom_y < between_notes_px` 면 `end_y = prev_content_bottom_y +
   between_notes_px` 로 끌어올린다. **단**, backtrack(`safe_vpos_backtrack`/`deep_backtrack`)·
   rewind·stale-forward 케이스는 **제외**(기존 분기 우선) → #1209 문12 무회귀.
2. **pagination 정합**: render 에서 min-gap 으로 end_y 를 늘리면 `shift_vpos_base_for_rendered_delta`
   (L419)로 base 를 이동해 후속 줄이 gap 을 복원하지 않게 함(기존 패턴). 단 pagination 의
   `compute_en_metrics` 도 같은 gap 을 예약해야 overflow(pi=475) 가 없다 → **Stage D 부분 정합**
   (between-notes 한정) 또는 compute_en_metrics 에 동일 min-gap 반영.

`between_notes_px` 는 HeightCursor 가 알아야 하므로 셋업 시 주입(현재 미보유 — Stage 2 에서 추가).

## 6. 잔여 리스크 / Stage 2 게이트

| 위험 | 방어 |
|------|------|
| backtrack/forward-cap 케이스 침범 (문12 등) | min-gap 은 **forward & gap 부족 & 비-backtrack** 에만. 기존 분기 우선순위 유지 |
| pi=475 overflow (pagination 미예약) | render min-gap + compute_en_metrics 동일 예약 동시 적용, 전체 cargo test 게이트 |
| HeightCursor 단위테스트(28+개) 회귀 | 신규 케이스는 기존 테스트 입력에 영향 없도록 좁은 조건. 신규 단위테스트 추가 |
| 문29/30 late-tail, 2024 미주모양 | 골든 + issue_1139/1189/1209 전체 통과 |

## 7. Stage 2 착수 전 — 구현계획서 필요

본 조사로 수정 위치(`HeightCursor.vpos_adjust` compact endnote min-gap + pagination 정합)와
회귀 게이트가 특정되었다. **수행계획서의 "전면 절대 vpos 앵커" 단계 구성은 본 발견에 맞게
구현계획서에서 재구성**한다(국소 HeightCursor 수정 중심). 구현계획서 승인 후 Stage 2 착수.
