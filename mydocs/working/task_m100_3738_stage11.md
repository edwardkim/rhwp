---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 11 — p67 각주 reservation/paint 정합

## 출발점

Stage 9 commit `c7e24d929`로 HWP 225→224쪽, p66 표 23 0–4행 및 각주 76·77 ownership은
한컴 PDF와 같아졌다. 그러나 PDF는 215쪽이고 p67에는 footer 아래까지 그려지는 footnote candidate가
남는다. Stage 10의 “모든 재조판 높이를 paginator에 전파” 가설은 226쪽으로 악화돼
`094d0c70d`에 기각 기록만 남겼다.

## 다음 판정 축

한컴 PDF p67은 table-cell anchor 78–82와 이후 본문 anchor 83–85를 가진다. rhwp는 이 모두를
p67에 모아 `FootnoteArea` 실제 paint가 footer를 넘긴다. 따라서 다음 분석은 저장 line height를
paginator에 일괄 확대하는 것이 아니라 다음을 측정한다.

1. PDF p67–p68 텍스트층에서 각주 78–85의 실제 page ownership과 첫 baseline y를 표로 만든다.
2. `TypesetState`의 table continuation 종료 시점과 후속 본문 footnote 등록 시점의
   `current_height`·`current_footnote_height`·page index를 기록해, ownership 재배치가 필요한지
   먼저 배제한다.
3. renderer `FootnoteArea`의 계산 height와 실제 마지막 line bottom을 대조해, pagination을
   바꾸지 않고 paint reservation만 정확히 맞출 수 있는지 확인한다.
4. 수정 뒤 p66–p67 144 DPI sweep과 page-count를 다시 기준값으로 남긴다. 이 단계에서 다음
   독립 분기가 확인되면 해당 분기만 별도 Stage로 분리한다.

원본 HWP·HWPX·PDF와 Stage 9 PNG는 [증적 보관 목록](../../pdf/pr3740/README.md)과
[Stage 9 visual sweep](task_m100_3738_stage9_visual_sweep.md)을 재사용한다.

## 진행 중인 실측

- `pdftotext -f 67 -l 68 -layout`로 기준 PDF를 다시 확인했다. p67에는 각주 **78–85 모두**가
  있고 p68은 86부터 시작한다. 즉 p67의 78–85 ownership 자체는 rhwp와 같으며, Stage 11의
  핵심은 note를 p68로 임의 이동하는 것이 아니다.
- `RHWP_DIAG_FN=1 RHWP_DIAG_AVAIL=1`으로 HWP typeset의 각주 등록과 예약 가용 높이 로그를
  `/private/tmp/rhwp-stage11-p67-diag.log`에 확보했다. p67 table continuation와 이어지는
  body footnote registration의 page/current-height 경계를 해당 trace에서 분리 중이다.
- `FootnoteArea`의 기존 reservation은 `Paragraph.line_segs[].line_height`만 합산했지만,
  실제 `layout_footnote_area`는 마지막 문단의 마지막 줄을 제외하고 각 composed line의
  `line_spacing`도 누적했다. p67에서는 reservation `y=669.5, h=369.8px`에 비해 실제
  마지막 각주 line bottom이 `1108.3px`이어서 footer top `1039.3px`를 69px 넘었다.

## 수정과 결과

`src/renderer/layout/picture_footnote.rs`의 renderer-side
`estimate_footnote_area_height`만 actual paint와 동일하게 composed line height + trailing
line-spacing(각주 마지막 문단의 마지막 줄 제외)을 계산하도록 보정했다. paginator의
`estimate_footnote_note_height`는 바꾸지 않았다. 따라서 Stage 10에서 관측한 224→226쪽
재분할은 일으키지 않는다.

수정 후 p67 `FootnoteArea`는 `y=600.6, h=438.7px`이고 실제 하단은 footer top
`1039.3px`에서 끝난다. 각주 78 첫 text line `y=620.0px`은 기준 PDF의 약 `616.49px`와
3.51px 차이다. p67의 78–85 ownership과 p66 표 23 fragment ownership은 유지됐다.

## 검증과 잔여

- `cargo fmt --check`, `git diff --check`
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment --test issue_3738_hwp_caption_cell_alignment` — 2 passed
- HWP p66–p67, 144 DPI sweep — SVG/render tree 224쪽, raster 2/2 완료, structural 후보 0건

회귀는 p67의 actual footnote subtree bottom이 footer top보다 1px 이상 아래로 내려가지 않는지를
직접 고정한다. 상세 PNG·지표는 [Stage 11 visual sweep](task_m100_3738_stage11_visual_sweep.md)에
기록했다.

HWP/HWPX 224쪽과 기준 PDF 215쪽의 전체 9쪽 차이는 여전히 남는다. 이 커밋은 footer collision
한 건만 해결하며, 다음 Stage에서 p68 이후의 최초 remaining pagination 분기를 새로 분석한다.
