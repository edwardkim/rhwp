# Stage 177: declared object와 행 frame 계약

## 계기

두 canonical #3820 fixture는 모두 한컴 PDF와 같은 383 physical page로 렌더되어야 한다.

- `samples/2025 행정업무운영 편람(최종).hwp`
- `samples/2025 행정업무운영 편람(최종).hwpx`

Stage 166 이후 두 파일 모두 section 2, paragraph 216의 physical page 47에서 처음
달라졌다. 36행 `Square`/`RowBreak` 표가 전체 `Table`로 출력되었지만, 383쪽 기준선은
rows 0부터 5까지를 `PartialTable`로 출력한다.

## 원인

Stage 166은 declared/measured drift 상한을 셀별 stored `lineSeg` predicate로 대체했다.
이 predicate는 각 셀의 텍스트가 선언된 셀 box 안에 들어감을 올바르게 증명하지만, 표
control의 declared object height가 모든 행을 포함함까지는 증명하지 않는다.

paragraph 216에서 표 object는 약 160px로 선언됐지만, stored cell row geometry는 약
1,200px다. 따라서 object frame이 stale/minimum frame인데도 셀 predicate가 통과했다.
이후 declared whole-fit 경로가 `RowBreak` scanner를 막아 이후 모든 page owner가 밀렸다.

## 보정

`table_declared_object_covers_cell_row_frames`는 이제 declared table object height가 각
non-rowspan 행의 최대 declared cell box와 declared cell spacing을 포함하도록 요구한다.
기존의 셀별 stored-line frame predicate는 계속 필요하지만, 이제 그것만으로는 충분하지
않다.

이는 measured-height ratio가 아닌 source geometry다. object frame이 declared row frame을
포함하지 않는 표는 일반 `RowBreak` scan을 사용하고, 완전한 object frame을 가진 표만
기존 stored fragment guard를 모두 충족할 때 declared whole-fit을 사용할 수 있다.

## 검증 목표

- Native verifier: HWP와 HWPX 모두 383쪽을 보고한다.
- Browser E2E: 두 원본을 독립적으로 업로드해 383쪽으로 렌더하고, canonical p285 HWP와
  p144 HWPX의 텍스트 owner를 유지한다.
