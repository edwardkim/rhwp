---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 10 — p67 각주 실제 조판 높이와 예약 높이 불일치

## 재현 기준

Stage 9 commit `c7e24d929`의 실제 HWP 결과는 224쪽이다. HWP PDF p66의 표 23 0–4행·각주
76·77 소유권은 복원됐지만, p67 visual sweep은 `frame_overflow_pixels` 35px 후보를 기록한다.
기준 원본과 PDF는 [증적 보관 목록](../../pdf/pr3740/README.md), p66–p67 PNG는
[Stage 9 visual sweep](task_m100_3738_stage9_visual_sweep.md)에 보관한다.

## 원인 가설과 코드 증거

p67 render tree에서 `FootnoteArea`의 예약 bbox는 `y=669.5, height=369.8`이고 body bottom은
`1039.3px`다. 그러나 같은 FootnoteArea의 실제 마지막 `TextLine`은 `y=1096.3px`까지 그려져
footer(`y=1039.3px`)와 겹친다. 기준 PDF p67은 footnote 78이 약 `462.37pt = 616.49px`에서
시작하며 footer와 겹치지 않는다.

`src/renderer/layout/picture_footnote.rs`의 `estimate_footnote_area_height`는 원본
`Paragraph.line_segs`의 저장 높이만 합산한다. 반면 `layout_footnote_area`는 같은 각주를
`compose_paragraph`로 현재 area 폭과 marker를 반영해 다시 줄바꿈한다. p67의 긴 URL·영문 각주는
실제 composed lines가 저장 line segments보다 많아진다. 따라서 paginator/`PageLayoutInfo`의
`footnote_area` 예약은 작고, renderer는 더 긴 footnote text를 footer 아래까지 그린다.

## 기각한 수정과 다음 분석

1. `estimate_footnote_note_height`와 renderer-side `estimate_footnote_area_height` 모두를
   `compose_paragraph` line height·trailing spacing 합계로 바꿔 보았다. 코드 경로 가설은 맞았지만
   실제 HWP는 **224→226쪽**으로 악화했고, Stage 9 focused test의 `<=224` 회귀도 실패했다.
2. 따라서 renderer의 실제 footnote height를 paginator에 단순 전파하는 것은 해법이 아니다.
   paginator가 table fragment를 먼저 결정한 뒤 footnote area가 body를 줄이는 순서와, 한컴의
   p67 footnote 시작 anchor를 함께 모델링해야 한다. 이 후보 코드는 커밋하지 않고 되돌렸다.
3. 다음 Stage는 동일 p67의 `PageContent.current_height`, `current_footnote_height`, renderer의
   `FootnoteArea` 시작 y를 시점별로 대조해 footer collision과 전체 page-count 증가를 동시에
   막는 ownership/anchor 경계를 찾는다.
