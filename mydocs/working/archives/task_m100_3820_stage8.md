---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820 Stage 8 — p118→p119 그림 55 앞 본문 fragment 분석

## 범위와 정답지

이번 단계는 #3820의 사용자 쪽 p118→p119 경계만 다룬다. 입력 HWP와 한컴 2020 기준 PDF는
Stage 7과 동일한 개인정보 제거 문서이며, raster 차이가 아니라 본문 `pi=1275`의 physical-page
owner를 PDF와 일치시키는 것이 목적이다.

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 기준 PDF: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 기준 renderer: `target/task-3820-3821-fidelity/release-test/rhwp`

## 재현과 source 계약

최신 binary의 selected visual sweep에서 p118은 `line_order_overlap`, p119는
`column_text_flow_collapse`로 flag된다. PDF는 p118에서 `pi=1275`의 앞 9줄로 끝나고,
p119 상단에서 같은 문단의 뒤 2줄을 먼저 그린 뒤 그림 55(`pi=1276`)를 배치한다. 현재 rhwp는
`pi=1275`의 11줄을 p118에 모두 배치해 p119가 그림 55로 바로 시작한다.

원본 `LINE_SEG`는 이를 직접 나타낸다.

| `pi=1275` line | stored `vpos` | physical owner |
| --- | ---: | --- |
| 0..8 | 52721..68721 HU | p118 |
| 9..10 | 0, 2000 HU | p119 |

뒤 문단 `pi=1276`은 visible text가 없는 단일 `treat_as_char=true`, `TopAndBottom` 그림이며,
첫 line의 `vpos=4000 HU`, 높이 `30373 HU`와 caption을 가진다. 따라서 `pi=1275`의 내부
`68721 → 0` reset은 일반 줄 간 coordinate drift가 아니라 그림 55 직전 physical page boundary다.

## 보정 경계

일반 native-HWP5 text reset을 전역 page break로 승격하지 않는다. 보정 후보는 다음을 모두
만족해야 한다.

1. native HWP5, 단일 column, 기존 각주가 없는 body page의 visible-text 문단이다.
2. 문단 내부의 non-synthetic line에서 양수 tail 좌표가 body height의 70% 이상에 있고 다음 줄이
   정확히 `vpos=0`으로 reset된다.
3. reset 뒤의 즉시 다음 문단은 text 없는 단일 `treat_as_char TopAndBottom` 그림/도형이며, source
   상단이 `vpos=0..8000 HU`이고 그림 줄 높이가 body height의 15% 이상이다.
4. 현재 flow도 page tail(본문 height의 60% 이상)에 있어야 한다.

이 좁은 source+successor 계약에서만 reset line을 existing line-split loop의
`forced_page_break_line`으로 전달한다. 그러면 앞 9줄은 p118의 `PartialParagraph`, 뒤 2줄은
p119의 continuation이 되고 그림 55는 그 뒤 normal flow로 배치된다. 표, Square 그림, 각주,
일반 `TopAndBottom` float와 arbitrary reset은 범위 밖이다.

## 수용 기준

1. p118에는 `pi=1275` lines `0..9`만 있고 p119에는 lines `9..11`과 그림 55(`pi=1276`)가 있다.
2. p118/p119 기준 PDF direct raster와 layout owner가 일치하고 기존 p94/p106/p107/p108/p156/
   p168~170 contracts가 유지된다.
3. focused Rust regression과 selected visual sweep을 다시 실행한다.

## 구현과 검증 결과

`native_hwp5_text_reset_before_large_tac_topbottom_picture_break_line`은 위 네 가지 조건을
모두 확인한 뒤에만 reset line을 typeset의 `forced_page_break_line`으로 제공한다. 결과적으로
p118에는 `PartialParagraph pi=1275 lines=0..9`, p119에는 `lines=9..11` 뒤
`Shape pi=1276 ci=0`이 기록된다. 이전처럼 p118이 993.4px로 body를 넘는 대신, 보정 후
940.1px로 source tail 안에 끝난다.

새 integration regression `native_hwp5_text_tail_before_figure_55_keeps_the_pdf_page_owner`는
render tree에서 p118의 `pi=1275` line index `0..8`, p119의 `9..10`, p119 그림 55 단일
Image owner를 고정한다. 같은 fixture의 footnote, RowBreak, Square 그림 경계를 포함한 focused
test target은 20 passed/0 failed다.

selected sweep은 p118/p119/p127의 requested/completed/missing을 **3/3/0**으로 완료했고,
p118과 p119 자동 flag는 모두 0건이다. direct review와 증적은
[Stage 8 visual sweep](task_m100_3820_stage8_visual_sweep.md)에 보관한다. p127의 PDF 대비
Square-wrap 여백 차이는 이 보정 범위에 포함되지 않으며 다음 stage의 잔여 결함으로 유지한다.
