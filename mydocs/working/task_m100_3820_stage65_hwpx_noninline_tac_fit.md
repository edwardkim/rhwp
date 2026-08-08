---
kind: analysis
status: active
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 65 — HWPX non-inline `treatAsChar` table fit

## Stage 64에서 확정한 PDF 계약

대상은 `samples/2025 행정업무운영 편람(최종).hwpx`와 한컴 2024 기준 PDF
`pdf/2025 행정업무운영 편람(최종)-2024.pdf`의 p144다. `section3.xml`의
`id=1723619577`(section 3, paragraph index 71) 3×1 표는 PDF p144 안에서 완료돼야
한다. p145로 옮겨서는 안 되는 표 안 text anchor는
`기안문에 작성한 붙임 문서를 첨부`다.

입력의 raw 속성은 `treatAsChar=1`, `flowWithText=0`, `pageBreak=NONE`이다.
따라서 이름과 달리 inline 글자처럼취급 표가 아니라 HWPX stored-layout의 block table이다.
PDF owner가 직접 정답지이고, HWPX source page count 387 및 기존 #3930의 p145 assertion은
이 계약보다 우선하지 않는다.

## 경로와 수치

`src/renderer/typeset.rs`는 `uses_tac_table_flow()`에서 HWPX lineage의 inline TAC을
`treat_as_char && flow_with_text`로 이미 정의한다. 그러나 declared whole-fit gate는 raw
`!table.common.treat_as_char`로 비-inline block table도 제외한다.

해당 표의 stage64 diagnostic은 다음과 같다.

| 항목 | 값 |
| --- | ---: |
| 시작 current height | 23.2px |
| page available | 740.8px |
| 실제 measured/effective height | 712.3px |
| fragment scan first part | 715.2px |
| declared total (host spacing 포함) | 720.3px |
| generic row-cut 마지막 row | 853.6px |

즉 measured placement는 735.5px로 실제 잔여 영역에 들어가지만, raw declared total에는
host spacing 8px가 합산되어 2.7px만 초과한다. raw TAC exclusion이 split path를 택하고,
그 path의 generic row-cut이 저장 row보다 167.7px 크게 계산해 p145 continuation을 만든다.

## 최소 보정 가설

HWPX stored-layout이고 `treatAsChar=1 && flowWithText=0`, `pageBreak=None`, table
footnote 없음, 실제 `effective_height`가 현재 page에 fit하는 경우만 non-inline block
table로 취급한다. 이 경우에는 declared whole-fit을 허용하되, layout advance도 실제
effective height로 제한해 host-spacing 반올림 때문에 다음 content를 footer 아래로 밀지
않는다.

이 보정은 native HWP5 및 진짜 inline TAC(`flowWithText=1`)을 건드리지 않는다. 모든
`treatAsChar` 표의 정책을 바꾸거나 전역 pagination tolerance를 늘리지 않는다.

## 검증 계획

1. helper 수준에서 HWPX non-inline TAC의 fit predicate를 구현한다.
2. 현행 direct HWPX p144/p145 source tree를 PDF owner 계약으로 고정한다. 기존 #3930
   save-equivalence assertion은 HWP round-trip 계약과 분리한다.
3. focused test, p143–p146 SVG/PDF visual sweep, HWPX page count 변화를 확인한다.
4. overflow-cell baseline과 76076 p33–p34를 우선 회귀 확인한다. 확장 release-test는
   코드 Stage가 끝난 뒤 최종 gate로 실행한다.

## 중단 조건

보정 후 p144 table 하단이 physical body를 넘거나 후속 owner가 겹치면 이 가설을 폐기하고,
`MeasuredTable` row-height/host-spacing 합산 경로를 별도 Stage로 분석한다.
