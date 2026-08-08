---
kind: analysis
status: completed
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

## 구현 결과

whole-fit 보정만으로는 p144의 render tree owner는 맞았지만, 하위 caption 세 줄이
physical body 밖으로 각각 13.7px, 6.3px, 21.0px clip됐다. 따라서 중단 조건에 따라
`MeasuredTable`을 바꾸지 않고 같은 outer cell의 layout을 다시 조사했다.

원인은 `src/renderer/layout/table_layout.rs`의 Task #362 일반 차단이었다. nested table이
있으면 모든 stored `LineSeg.vpos`를 쓰지 않아 Center 셀의 문단이 순차 flow로 다시
쌓였고, p144 outer cell의 inner 2×3 caption table이 약 170px 아래로 밀렸다. 대상 HWPX
outer cell은 다음의 독립 근거를 모두 만족한다.

| 확인 항목 | 값 |
| --- | --- |
| outer table | `treatAsChar=1`, `flowWithText=0`, `pageBreak=NONE` |
| nested-cell stored extent / 자체 측정 | 676.1px / 676.1px |
| stored line-height 합 | 467.6px |
| nested/wrap physical bottom | 676.1px 이하 |
| 문단 anchor | 모든 문단의 연속 `vpos` 사다리 보유 |

따라서 일반 nested-cell의 cumulative/reset `vpos` 차단은 유지하고, HWPX stored-layout의
non-inline block-TAC에만 위 형상 검사를 통과했을 때 문단 위치 anchor를 복원했다. extent가
자체 측정과 같은 경우에도 전체 높이는 바꾸지 않고 좌표만 복원한다. 이로써 p144의 image
caption이 PDF와 같이 쪽 안에서 끝나며, p145에는 앞 table의 content가 남지 않는다.

`tests/issue_3930_hwpx_hwp_save_layout.rs`는 PDF p144 owner뿐 아니라 새 `DocumentCore`
렌더에서 해당 쪽의 `overflow_cell_lines == 0`도 고정한다. tree에만 text가 남고 실제 paint가
footer 밖으로 잘리는 우회 회귀를 막는다.

## 검증 결과

- `cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout -- --nocapture` — 통과.
- `cargo test --profile release-test --test issue_1891 -- --nocapture` — 4/4 통과. Task #362의
  일반 HWPX nested-table clip guard가 유지됨을 확인했다.
- `cargo test --profile release-test --test overflow_cell_baseline -- --nocapture` — 통과,
  678 fixtures(3 skip), nonzero 17 documents, total 691 lines. baseline을 변경하지 않았다.
- p143–p146 180 DPI direct PDF sweep — SVG 386/386, requested raster 4/4. p144의 하위
  예시/caption은 body 안에 있고 p145 owner도 PDF와 일치한다. 근거 PNG는
  `mydocs/pr/assets/task_m100_3820_stage65_hwpx_noninline_tac_fit/review_144.png`,
  `review_145.png` 및 `summary.json`에 보존했다. font/ink raster 차이는 overlay의
  pixel proxy와 분리해 page-owner 및 physical clip을 직접 판정했다.
